; Runs the production helper launcher without installing or registering anything.
[Setup]
AppName=Installer helper test
AppVersion=1.0
DefaultDirName={tmp}
CreateAppDir=no
Uninstallable=no
PrivilegesRequired=lowest
OutputDir={#FixtureDirectory}\compiled
OutputBaseFilename=helper-test

[Code]
#include "..\..\packaging\helpers.iss"

procedure Check(Condition: Boolean; const Message: String);
begin
  if not Condition then RaiseException(Message);
end;

function InitializeSetup: Boolean;
var
  Helper, ProtectedLog, WritableLog, Failure, Report: String;
begin
  Result := False;
  Report := ExpandConstant('{#FixtureDirectory}\result.txt');
  try
    Helper := ExpandConstant('{#FixtureDirectory}\helper.exe');
    ProtectedLog := ExpandConstant('{#FixtureDirectory}\protected\user.log');
    WritableLog := ExpandConstant('{#FixtureDirectory}\machine.log');
    Failure := RunHelper(Helper, '--package-check-user', 'Checking user settings', ProtectedLog, True);
    Check(Failure = '', 'Normal-user helper failed with a protected log folder: ' + Failure);
    Failure := RunHelper(Helper, '--package-reset-user', 'Resetting user settings', ProtectedLog, True);
    Check(Failure = '', 'Normal-user reset failed with a protected log folder: ' + Failure);
    Check(not FileExists(ProtectedLog), 'Normal-user helper wrote into the protected folder');
    Failure := RunHelper(Helper, '', 'Checking files', WritableLog, False);
    Check(Failure = '', 'Machine helper failed: ' + Failure);
    Check(FileExists(WritableLog), 'Machine helper did not write its diagnostic log');
    Failure := RunHelper(Helper, '--fail', 'Checking files', WritableLog, False);
    Check(Pos('fixture failure', Failure) > 0, 'Machine helper failure lost its diagnostic');
    Failure := RunHelper(Helper, '--fail', 'Checking user settings', ProtectedLog, True);
    Check(Failure <> '', 'Normal-user helper failure was reported as success');
    Failure := RunHelper(Helper, '--negative', 'Checking user settings', ProtectedLog, True);
    Check(Pos('exit -1', Failure) > 0, 'Normal-user helper exit -1 was lost');
    Failure := RunHelper(Helper, '--negative', 'Checking files', WritableLog, False);
    Check(Pos('exit -1', Failure) > 0, 'Machine helper exit -1 was lost');
    Failure := RunHelper(Helper + '.missing', '', 'Checking files', WritableLog, False);
    Check(Failure <> '', 'Missing helper was reported as success');
    Check(SaveStringToFile(Report, 'PASS', False), 'Cannot write test result');
  except
    SaveStringToFile(Report, 'FAIL: ' + GetExceptionMessage, False);
  end;
end;
