//! Exact-SID Windows LSA adapter for the dedicated runner account rights.

use std::collections::BTreeSet;
use std::fmt;

use windows::Win32::Foundation::{HLOCAL, LocalFree, STATUS_OBJECT_NAME_NOT_FOUND};
use windows::Win32::Security::Authentication::Identity::{
    LSA_HANDLE, LSA_OBJECT_ATTRIBUTES, LSA_UNICODE_STRING, LsaAddAccountRights, LsaClose,
    LsaEnumerateAccountRights, LsaFreeMemory, LsaNtStatusToWinError, LsaOpenPolicy,
    LsaRemoveAccountRights, POLICY_CREATE_ACCOUNT, POLICY_LOOKUP_NAMES,
};
use windows::Win32::Security::Authorization::{ConvertSidToStringSidW, ConvertStringSidToSidW};
use windows::Win32::Security::PSID;
use windows::core::{PCWSTR, PWSTR};

use crate::account_rights::{RUNNER_RIGHTS, expected_rights, update_required};

/// Native account-right update failure.
#[derive(Debug)]
pub enum AccountRightsError {
    /// The supplied SID was not a canonical SID string.
    InvalidSid,
    /// A Windows LSA or allocation operation failed.
    Windows(std::io::Error),
    /// LSA reported a different final right set than the exact requested delta.
    Verification {
        /// Expected final rights.
        expected: BTreeSet<String>,
        /// Observed final rights.
        actual: BTreeSet<String>,
    },
}

impl fmt::Display for AccountRightsError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSid => formatter.write_str("runner account SID is invalid"),
            Self::Windows(error) => {
                write!(formatter, "Windows account-right operation failed: {error}")
            }
            Self::Verification { expected, actual } => {
                write!(
                    formatter,
                    "account-right verification mismatch: expected {expected:?}, actual {actual:?}"
                )
            }
        }
    }
}

impl std::error::Error for AccountRightsError {}

/// Add or remove only the fixed runner rights for one exact SID and verify that
/// every unrelated right remains unchanged.
///
/// # Errors
/// Rejects an invalid SID and preserves native LSA failures or a final-state
/// mismatch.
#[allow(unsafe_code)]
pub fn update_runner_rights(sid_text: &str, grant: bool) -> Result<(), AccountRightsError> {
    let sid = OwnedSid::parse(sid_text)?;
    let policy = Policy::open()?;
    let before = enumerate_rights(policy.0, sid.0)?;
    if !update_required(&before, grant) {
        return Ok(());
    }
    let expected = expected_rights(&before, grant);
    let buffers: Vec<Vec<u16>> = RUNNER_RIGHTS
        .iter()
        .map(|right| right.encode_utf16().collect())
        .collect();
    let rights: Vec<LSA_UNICODE_STRING> = buffers.iter().map(|value| lsa_string(value)).collect();
    // SAFETY: policy and SID are owned valid handles, while every LSA string
    // points into `buffers`, which remains alive and unmodified for this call.
    let status = unsafe {
        if grant {
            LsaAddAccountRights(policy.0, sid.0, &rights)
        } else {
            LsaRemoveAccountRights(policy.0, sid.0, false, Some(&rights))
        }
    };
    check_status(status.0)?;
    let actual = enumerate_rights(policy.0, sid.0)?;
    if actual != expected {
        return Err(AccountRightsError::Verification { expected, actual });
    }
    Ok(())
}

struct Policy(LSA_HANDLE);

impl Policy {
    #[allow(unsafe_code)]
    fn open() -> Result<Self, AccountRightsError> {
        let mut attributes = LSA_OBJECT_ATTRIBUTES {
            Length: u32::try_from(std::mem::size_of::<LSA_OBJECT_ATTRIBUTES>())
                .expect("LSA_OBJECT_ATTRIBUTES size fits u32"),
            ..Default::default()
        };
        let mut handle = LSA_HANDLE::default();
        // SAFETY: null system name selects the local policy; both output
        // structures are initialized and valid for the call.
        let status = unsafe {
            LsaOpenPolicy(
                None,
                &raw mut attributes,
                (POLICY_LOOKUP_NAMES | POLICY_CREATE_ACCOUNT) as u32,
                &raw mut handle,
            )
        };
        check_status(status.0)?;
        Ok(Self(handle))
    }
}

impl Drop for Policy {
    #[allow(unsafe_code)]
    fn drop(&mut self) {
        // SAFETY: handle is owned by this guard after successful open.
        let _ = unsafe { LsaClose(self.0) };
    }
}

struct OwnedSid(PSID);

impl OwnedSid {
    #[allow(unsafe_code)]
    fn parse(value: &str) -> Result<Self, AccountRightsError> {
        if value.contains('\0') {
            return Err(AccountRightsError::InvalidSid);
        }
        let wide: Vec<u16> = value.encode_utf16().chain(std::iter::once(0)).collect();
        let mut sid = PSID::default();
        // SAFETY: input is NUL-terminated and SID receives LocalAlloc memory.
        unsafe { ConvertStringSidToSidW(PCWSTR(wide.as_ptr()), &mut sid) }
            .map_err(|_| AccountRightsError::InvalidSid)?;
        let owned = Self(sid);
        if canonical_sid(owned.0)? != value {
            return Err(AccountRightsError::InvalidSid);
        }
        Ok(owned)
    }
}

#[allow(unsafe_code)]
fn canonical_sid(sid: PSID) -> Result<String, AccountRightsError> {
    let mut text = PWSTR::null();
    // SAFETY: SID is valid and the output receives LocalAlloc memory.
    unsafe { ConvertSidToStringSidW(sid, &raw mut text) }
        .map_err(|error| AccountRightsError::Windows(std::io::Error::other(error)))?;
    let mut length = 0;
    // SAFETY: the API returned a NUL-terminated UTF-16 string.
    unsafe {
        while *text.0.add(length) != 0 {
            length += 1;
        }
    }
    // SAFETY: the preceding scan found the terminator within the API-owned string.
    let words = unsafe { std::slice::from_raw_parts(text.0, length) };
    let decoded = String::from_utf16(words).map_err(|error| {
        AccountRightsError::Windows(std::io::Error::new(std::io::ErrorKind::InvalidData, error))
    });
    // SAFETY: text was allocated by ConvertSidToStringSidW.
    let _ = unsafe { LocalFree(Some(HLOCAL(text.0.cast()))) };
    decoded
}

impl Drop for OwnedSid {
    #[allow(unsafe_code)]
    fn drop(&mut self) {
        // SAFETY: SID was allocated by ConvertStringSidToSidW.
        let _ = unsafe { LocalFree(Some(HLOCAL(self.0.0))) };
    }
}

#[allow(unsafe_code)]
fn enumerate_rights(policy: LSA_HANDLE, sid: PSID) -> Result<BTreeSet<String>, AccountRightsError> {
    let mut values = std::ptr::null_mut();
    let mut count = 0_u32;
    // SAFETY: policy/SID are valid and the output pointers are writable.
    let status = unsafe { LsaEnumerateAccountRights(policy, sid, &raw mut values, &raw mut count) };
    if status == STATUS_OBJECT_NAME_NOT_FOUND {
        return Ok(BTreeSet::new());
    }
    check_status(status.0)?;
    let mut rights = BTreeSet::new();
    let mut decoding_error = None;
    if count > 0 {
        if values.is_null() {
            return Err(AccountRightsError::Windows(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "LSA returned a null account-right buffer",
            )));
        }
        // SAFETY: successful LSA call returns `count` initialized entries.
        let slice = unsafe { std::slice::from_raw_parts(values, count as usize) };
        for value in slice {
            let length = usize::from(value.Length) / 2;
            // SAFETY: each LSA string buffer is valid for Length bytes until freed.
            let words = unsafe { std::slice::from_raw_parts(value.Buffer.0, length) };
            match String::from_utf16(words) {
                Ok(right) => {
                    rights.insert(right);
                }
                Err(error) => {
                    decoding_error = Some(AccountRightsError::Windows(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        error,
                    )));
                    break;
                }
            }
        }
    }
    if !values.is_null() {
        // SAFETY: values was allocated by LsaEnumerateAccountRights.
        let free_status = unsafe { LsaFreeMemory(Some(values.cast())) };
        check_status(free_status.0)?;
    }
    if let Some(error) = decoding_error {
        return Err(error);
    }
    Ok(rights)
}

fn lsa_string(value: &[u16]) -> LSA_UNICODE_STRING {
    let byte_length = u16::try_from(value.len() * 2).expect("fixed right name fits u16");
    LSA_UNICODE_STRING {
        Length: byte_length,
        MaximumLength: byte_length,
        Buffer: PWSTR(value.as_ptr().cast_mut()),
    }
}

#[allow(unsafe_code)]
fn check_status(status: i32) -> Result<(), AccountRightsError> {
    if status == 0 {
        return Ok(());
    }
    // SAFETY: conversion accepts any NTSTATUS value and returns a Win32 code.
    let code = unsafe { LsaNtStatusToWinError(windows::Win32::Foundation::NTSTATUS(status)) };
    Err(AccountRightsError::Windows(
        std::io::Error::from_raw_os_error(i32::try_from(code).unwrap_or(i32::MAX)),
    ))
}

#[cfg(test)]
mod tests {
    use super::{AccountRightsError, OwnedSid};

    #[test]
    fn sid_parser_requires_canonical_untruncated_text() {
        assert!(OwnedSid::parse("S-1-5-32-544").is_ok());
        assert!(matches!(
            OwnedSid::parse("BA"),
            Err(AccountRightsError::InvalidSid)
        ));
        assert!(matches!(
            OwnedSid::parse("S-1-5-32-544\0ignored"),
            Err(AccountRightsError::InvalidSid)
        ));
    }
}
