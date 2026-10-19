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
  { Inno's elevated temporary directory is read-only to the original user.
    Only machine helpers may create logs there. }
  if OriginalUser then
    Started := ExecAsOriginalUser(Helper, Args, '',
      SW_HIDE, ewWaitUntilTerminated, ExitCode)
  else
    Started := Exec(Helper, Args + ' --log ' + Quote(LogPath), '',
      SW_HIDE, ewWaitUntilTerminated, ExitCode);
  if not Started then
    Result := Action + ' could not start: ' + SysErrorMessage(ExitCode)
  else if ExitCode <> 0 then begin
    if OriginalUser then
      Result := Action + ' failed (exit ' + IntToStr(ExitCode) + '). ' +
        'Close Hyper GPU Support and start Setup normally from your Windows account.'
    else
      Result := HelperError(Action, LogPath, ExitCode);
  end
  else
    Result := '';
  Log(Action + ': exit ' + IntToStr(ExitCode));
end;

