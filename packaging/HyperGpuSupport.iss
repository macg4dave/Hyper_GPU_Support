; Build only through scripts/build-installer.ps1; all payload paths are staged.
#if Ver < EncodeVer(6, 7, 3, 0) || Ver >= EncodeVer(6, 7, 4, 0)
  #error This installer requires pinned Inno Setup 6.7.3
#endif
#ifndef AppVersion
  #error AppVersion is required
#endif
#ifndef PayloadDir
  #error PayloadDir is required
#endif
#ifndef OutputDir
  #error OutputDir is required
#endif
#ifndef OutputName
  #error OutputName is required
#endif

[Setup]
AppId={{582BBF4B-C798-48B5-9F32-56F8064CA698}
AppName=Hyper GPU Support
AppVersion={#AppVersion}
AppVerName=Hyper GPU Support {#AppVersion}
; Match the CompanyName/LegalCopyright in build/windows_resources.rs.
AppPublisher=macg4dave
AppCopyright=Copyright (c) 2026 macg4dave
DefaultDirName={autopf}\HyperGpuSupport
DefaultGroupName=Hyper GPU Support
DisableDirPage=yes
DisableProgramGroupPage=yes
UsePreviousAppDir=no
PrivilegesRequired=admin
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
MinVersion=10.0.22000
OutputDir={#OutputDir}
OutputBaseFilename={#OutputName}
VersionInfoVersion={#FileVersion}
VersionInfoProductVersion={#FileVersion}
VersionInfoProductTextVersion={#AppVersion}
VersionInfoDescription=Hyper GPU Support Setup
VersionInfoProductName=Hyper GPU Support
LicenseFile={#PayloadDir}\LICENSE
InfoBeforeFile=reset-notice.txt
WizardStyle=modern
Compression=lzma2
SolidCompression=yes
CloseApplications=no
RestartApplications=no
SetupMutex=Global\HyperGpuSupportSetup
UninstallDisplayIcon={app}\hyper-gpu-support.exe
UninstallDisplayName=Hyper GPU Support
UninstallLogMode=append
ChangesEnvironment=no
#ifdef SignedBuild
SignedUninstaller=yes
SignedUninstallerDir={#UninstallerDir}
#else
SignedUninstaller=no
#endif

[Languages]
Name: "enGB"; MessagesFile: "compiler:Default.isl"

[LangOptions]
LanguageName=English (United Kingdom)
LanguageID=$0809

[Messages]
WizardLicense=Licence Agreement
LicenseLabel=Please read the following important information before continuing.
LicenseLabel3=Please read the following Licence Agreement. You must accept its terms before continuing with the installation.
LicenseAccepted=I &accept the agreement
LicenseNotAccepted=I &do not accept the agreement

[Files]
; Early extraction supports read-only preflight before any installed file changes.
Source: "{#PayloadDir}\*.exe"; Flags: dontcopy noencryption
Source: "{#PayloadDir}\SHA256SUMS.txt"; Flags: dontcopy noencryption
Source: "{#PayloadDir}\*.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#PayloadDir}\SHA256SUMS.txt"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#PayloadDir}\LICENSE"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#PayloadDir}\licenses\*"; DestDir: "{app}\licenses"; Flags: ignoreversion recursesubdirs createallsubdirs

[Icons]
Name: "{group}\Hyper GPU Support"; Filename: "{app}\hyper-gpu-support.exe"

[Code]
var
  PayloadExtracted: Boolean;
  MaintenanceFailed: Boolean;

function Quote(const Value: String): String;
begin
  Result := '"' + Value + '"';
end;

function HelperError(const Action, LogPath: String; ExitCode: Integer): String;
var
  Details: AnsiString;
begin
  Result := Action + ' failed (exit ' + IntToStr(ExitCode) + ').';
  if LoadStringFromFile(LogPath, Details) then
    Result := Result + #13#10 + String(Details);
end;

function RunHelper(const Helper, Args, Action, LogPath: String;
  OriginalUser: Boolean): String;
var
  ExitCode: Integer;
  Started: Boolean;
begin
  ExitCode := -1;
  if OriginalUser then
    Started := ExecAsOriginalUser(Helper, Args + ' --log ' + Quote(LogPath), '',
      SW_HIDE, ewWaitUntilTerminated, ExitCode)
  else
    Started := Exec(Helper, Args + ' --log ' + Quote(LogPath), '',
      SW_HIDE, ewWaitUntilTerminated, ExitCode);
  if not Started then
    Result := Action + ' could not start: ' + SysErrorMessage(ExitCode)
  else if ExitCode <> 0 then
    Result := HelperError(Action, LogPath, ExitCode)
  else
    Result := '';
end;

function PrepareToInstall(var NeedsRestart: Boolean): String;
var
  Helper, LogPath: String;
begin
  if CompareText(ExpandConstant('{app}'),
    ExpandConstant('{autopf}\HyperGpuSupport')) <> 0 then begin
    Result := 'Hyper GPU Support must be installed in its standard Program Files folder.';
    Exit;
  end;
  if not PayloadExtracted then begin
    ExtractTemporaryFile('hyper-gpu-support.exe');
    ExtractTemporaryFile('hyper-gpu-runner.exe');
    ExtractTemporaryFile('hyper-gpu-guest.exe');
    ExtractTemporaryFile('d3d11-probe.exe');
    ExtractTemporaryFile('app-install.exe');
    ExtractTemporaryFile('SHA256SUMS.txt');
    PayloadExtracted := True;
  end;
  Helper := ExpandConstant('{tmp}\app-install.exe');
  LogPath := ExpandConstant('{tmp}\user-check.log');
  Result := RunHelper(Helper, '--package-check-user',
    'Checking the initiating user', LogPath, True);
  if Result <> '' then Exit;
  LogPath := ExpandConstant('{tmp}\package-check.log');
  Result := RunHelper(Helper, '--package-check --release-dir ' +
    Quote(ExpandConstant('{tmp}')), 'Checking installation safety', LogPath, False);
end;

procedure CurStepChanged(CurStep: TSetupStep);
var
  Helper, LogPath, Failure: String;
begin
  Helper := ExpandConstant('{tmp}\app-install.exe');
  if CurStep = ssInstall then begin
    LogPath := ExpandConstant('{tmp}\package-reset.log');
    Failure := RunHelper(Helper, '--package-reset --release-dir ' +
      Quote(ExpandConstant('{tmp}')), 'Archiving and resetting protected state',
      LogPath, False);
    if Failure <> '' then RaiseException(Failure);
  end;
  if CurStep = ssPostInstall then begin
    { Inno catches ssPostInstall exceptions and still displays its finished page.
      Keep failure set through every exit, including an unexpected exception. }
    MaintenanceFailed := True;
    LogPath := ExpandConstant('{tmp}\package-verify.log');
    Failure := RunHelper(Helper, '--package-verify --release-dir ' +
      Quote(ExpandConstant('{app}')), 'Verifying installed files', LogPath, False);
    if Failure <> '' then RaiseException(Failure);
    LogPath := ExpandConstant('{tmp}\user-reset.log');
    Failure := RunHelper(Helper, '--package-reset-user',
      'Resetting the initiating user settings', LogPath, True);
    if Failure <> '' then RaiseException(Failure);
    MaintenanceFailed := False;
  end;
end;

function GetCustomSetupExitCode: Integer;
begin
  if MaintenanceFailed then
    Result := 20
  else
    Result := 0;
end;

procedure CurPageChanged(CurPageID: Integer);
begin
  if (CurPageID = wpFinished) and MaintenanceFailed then begin
    WizardForm.FinishedHeadingLabel.Caption := 'Installation checks failed';
    WizardForm.FinishedLabel.Caption :=
      'Application files were copied, but verification or resetting settings failed. ' +
      'Setup did not complete successfully. Review the error shown earlier and ' +
      'the setup log. Preserve any protected archive.';
  end;
end;

function InitializeUninstall: Boolean;
var
  Failure, LogPath: String;
begin
  LogPath := ExpandConstant('{tmp}\uninstall-check.log');
  Failure := RunHelper(ExpandConstant('{app}\app-install.exe'),
    '--package-check --release-dir ' + Quote(ExpandConstant('{app}')),
    'Checking uninstall safety', LogPath, False);
  Result := Failure = '';
  if not Result then MsgBox(Failure, mbError, MB_OK);
end;

procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
var
  Failure, LogPath: String;
begin
  if CurUninstallStep = usUninstall then begin
    LogPath := ExpandConstant('{tmp}\uninstall-reset.log');
    Failure := RunHelper(ExpandConstant('{app}\app-install.exe'),
      '--package-uninstall', 'Archiving and removing protected registration and state',
      LogPath, False);
    { Unlike ssPostInstall, Inno propagates usUninstall exceptions before removing
      any logged application files. Keep the existing installation on failure. }
    if Failure <> '' then RaiseException(Failure);
  end;
end;
