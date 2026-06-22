#define MyAppName "ait"
#define MyAppVersion GetEnv("AIT_VERSION")
#define MyAppExeName "ait.exe"
#define MyAppSourceExe GetEnv("AIT_SOURCE_EXE")
#define MyAppOutputDir GetEnv("AIT_OUTPUT_DIR")
#define MyAppOutputBase GetEnv("AIT_OUTPUT_BASE")

[Setup]
AppId={{8F5939A4-77A7-4EE9-9E1F-A01E7E728437}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher=ait
DefaultDirName={autopf}\ait
DefaultGroupName=ait
DisableProgramGroupPage=yes
OutputDir={#MyAppOutputDir}
OutputBaseFilename={#MyAppOutputBase}
Compression=lzma
SolidCompression=yes
WizardStyle=modern
ArchitecturesInstallIn64BitMode=x64compatible
PrivilegesRequired=lowest
CloseApplications=force
UninstallDisplayIcon={app}\{#MyAppExeName}

[Files]
Source: "{#MyAppSourceExe}"; DestDir: "{app}"; DestName: "{#MyAppExeName}"; Flags: ignoreversion

[Icons]
Name: "{group}\ait"; Filename: "{app}\{#MyAppExeName}"

[Run]
Filename: "{app}\{#MyAppExeName}"; Description: "Launch ait"; Flags: nowait postinstall skipifsilent

[Code]
const
  AitTrayWindowClassName = 'ait_tray_window';
  WM_TRAY_COMMAND = $8014;
  MENU_EXIT = 1004;
  CloseWaitAttempts = 50;
  CloseWaitDelayMs = 100;

function AitTrayWindowHandle: HWND;
begin
  Result := FindWindowByClassName(AitTrayWindowClassName);
end;

procedure RequestRunningAitExit;
var
  Window: HWND;
  Attempt: Integer;
begin
  Window := AitTrayWindowHandle;
  if Window = 0 then
    Exit;

  Log('Requesting running ait instance to exit through tray command.');
  if not PostMessage(Window, WM_TRAY_COMMAND, MENU_EXIT, 0) then
  begin
    Log('Failed to post tray exit command to running ait instance.');
    Exit;
  end;

  for Attempt := 1 to CloseWaitAttempts do
  begin
    Sleep(CloseWaitDelayMs);
    Window := AitTrayWindowHandle;
    if Window = 0 then
    begin
      Log('Running ait tray window closed.');
      Exit;
    end;
  end;

  Log('Timed out waiting for running ait tray window to close.');
end;

function PrepareToInstall(var NeedsRestart: Boolean): String;
begin
  RequestRunningAitExit;
  Result := '';
end;
