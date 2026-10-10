//! Exact display conversion for the native Hyper-V VRAM byte contract.
//! This requests values; it makes no physical reservation/enforcement claim.
use hyper_gpu_support::model::Allocation;

const BYTES_PER_GB: u64 = 1_000_000_000;
const BYTES_PER_STEP: u64 = BYTES_PER_GB / 10;
// Every selectable integer must survive Slint's f32 Slider representation.
const MAX_EXACT_STEP: u64 = 1 << 24;

pub(super) struct MemorySlider {
    pub minimum: i32,
    pub maximum: i32,
    pub position: i32,
    pub label: String,
    pub range_start: String,
    pub range_end: String,
    bounds: Allocation,
    limits: Allocation,
}

pub(super) fn format_gb(bytes: u64) -> String {
    let whole = bytes / BYTES_PER_GB;
    let fraction = bytes % BYTES_PER_GB;
    if fraction == 0 {
        format!("{whole} GB")
    } else {
        let digits = format!("{fraction:09}");
        format!("{whole}.{} GB", digits.trim_end_matches('0'))
    }
}

/// Incomplete/invalid raw edits are never replaced by moving the slider.
pub(super) fn request(values: &[String]) -> Result<Option<Allocation>, String> {
    if values.len() != 3 {
        return Err("VRAM fields are unavailable.".into());
    }
    if values.iter().all(|v| v.trim().is_empty()) {
        return Ok(None);
    }
    let parsed = values
        .iter()
        .map(|v| v.trim().parse::<u64>())
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| "Complete or clear all three VRAM fields to use the slider.")?;
    Ok(Some(Allocation {
        minimum: parsed[0],
        optimal: parsed[1],
        maximum: parsed[2],
    }))
}

impl MemorySlider {
    /// Call only for native VRAM bytes, never arbitrary historical provider units.
    pub fn new(limits: &Allocation, current: Option<&Allocation>) -> Result<Self, String> {
        limits.validate(limits)?;
        let bounds = current.unwrap_or(limits);
        bounds
            .validate(limits)
            .map_err(|_| "Correct the VRAM range in Advanced Allocation to use the slider.")?;
        let lower = bounds.minimum.div_ceil(BYTES_PER_STEP).max(1);
        let upper = bounds.maximum / BYTES_PER_STEP;
        if upper > MAX_EXACT_STEP {
            return Err(
                "VRAM bounds exceed exact slider precision. Use Advanced Allocation.".into(),
            );
        }
        if lower >= upper {
            return Err(
                "This VRAM range has no usable 0.1 GB slider interval. Use Advanced Allocation."
                    .into(),
            );
        }
        Ok(Self {
            minimum: lower as i32,
            maximum: upper as i32,
            position: (bounds.optimal / BYTES_PER_STEP).clamp(lower, upper) as i32,
            label: current.map_or_else(|| "Not specified".into(), |v| format_gb(v.optimal)),
            range_start: format_gb(lower * BYTES_PER_STEP),
            range_end: format_gb(upper * BYTES_PER_STEP),
            bounds: bounds.clone(),
            limits: limits.clone(),
        })
    }

    pub fn select(&self, position: i32) -> Result<Allocation, String> {
        if !(self.minimum..=self.maximum).contains(&position) {
            return Err("The selected memory value is outside the current VRAM range.".into());
        }
        let optimal = u64::try_from(position)
            .map_err(|_| "Invalid memory value")?
            .checked_mul(BYTES_PER_STEP)
            .ok_or("Memory conversion overflow")?;
        let result = Allocation {
            optimal,
            ..self.bounds.clone()
        };
        result.validate(&self.limits)?;
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn limits() -> Allocation {
        Allocation {
            minimum: 1,
            optimal: 4_000_000_000,
            maximum: 8_000_000_000,
        }
    }

    #[test]
    fn slider_preserves_bounds_and_writes_exact_decimal_bytes() {
        let current = Allocation {
            minimum: 2_000_000_001,
            optimal: 4_000_000_001,
            maximum: 6_099_999_999,
        };
        let slider = MemorySlider::new(&limits(), Some(&current)).unwrap();
        assert_eq!((slider.minimum, slider.maximum), (21, 60));
        assert_eq!(slider.label, "4.000000001 GB");
        let selected = slider.select(55).unwrap();
        assert_eq!(selected.minimum, current.minimum);
        assert_eq!(selected.maximum, current.maximum);
        assert_eq!(selected.optimal, 5_500_000_000);
        assert!(slider.select(61).is_err());
        assert!(slider.select(-1).is_err());
    }

    #[test]
    fn unspecified_request_is_not_invented_until_selection() {
        let limits = limits();
        let slider = MemorySlider::new(&limits, None).unwrap();
        assert_eq!(slider.label, "Not specified");
        assert_eq!(
            slider.select(80).unwrap(),
            Allocation {
                optimal: 8_000_000_000,
                ..limits
            }
        );
        assert_eq!(
            request(&[String::new(), String::new(), String::new()]).unwrap(),
            None
        );
        assert!(request(&["1".into(), "".into(), "8".into()]).is_err());
    }

    #[test]
    fn invalid_narrow_and_imprecise_ranges_stay_unavailable() {
        assert!(
            MemorySlider::new(
                &Allocation {
                    minimum: 9,
                    optimal: 8,
                    maximum: 10
                },
                None
            )
            .is_err()
        );
        assert!(
            MemorySlider::new(
                &Allocation {
                    minimum: 0,
                    optimal: 1,
                    maximum: 99_999_999
                },
                None
            )
            .is_err()
        );
        assert!(
            MemorySlider::new(
                &Allocation {
                    minimum: 0,
                    optimal: 1,
                    maximum: u64::MAX
                },
                None
            )
            .is_err()
        );
        assert!(
            MemorySlider::new(
                &limits(),
                Some(&Allocation {
                    minimum: 1,
                    optimal: 9_000_000_000,
                    maximum: 9_000_000_000
                })
            )
            .is_err()
        );
        assert_eq!(format_gb(u64::MAX), "18446744073.709551615 GB");
    }
}
