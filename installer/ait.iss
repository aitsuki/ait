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
ArchitecturesInstallIn64BitMode=x64
PrivilegesRequired=lowest
UninstallDisplayIcon={app}\{#MyAppExeName}

[Files]
Source: "{#MyAppSourceExe}"; DestDir: "{app}"; DestName: "{#MyAppExeName}"; Flags: ignoreversion

[Icons]
Name: "{group}\ait"; Filename: "{app}\{#MyAppExeName}"

[Run]
Filename: "{app}\{#MyAppExeName}"; Description: "Launch ait"; Flags: nowait postinstall skipifsilent
