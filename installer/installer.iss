; OpenCrypt InnoSetup Installer
#define MyAppName "OpenCrypt"
#define MyAppVersion "0.2.1"
#define MyAppPublisher "OpenCrypt"
#define MyAppURL "https://github.com/Arean-Max/OpenCrypt"
#define MyAppExeName "OpenCrypt.exe"

[Setup]
AppId={{8A2C5B1E-3D4F-4A6B-9C8D-1E2F3A4B5C6D}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
DefaultDirName={localappdata}\Programs\{#MyAppName}
DefaultGroupName={#MyAppName}
OutputDir=..\Release
OutputBaseFilename=OpenCrypt_Setup_v{#MyAppVersion}
Compression=lzma2
SolidCompression=yes
UninstallDisplayIcon={app}\{#MyAppExeName}
PrivilegesRequired=lowest
DisableProgramGroupPage=yes
SetupIconFile=..\assets\shield.ico
ChangesEnvironment=yes

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"
Name: "russian"; MessagesFile: "compiler:Languages\Russian.isl"

[Tasks]
Name: "addtopath"; Description: "Add OpenCrypt to PATH (enables the `opc` command)"; Flags: checkedonce

[Files]
Source: "..\Release\{#MyAppExeName}"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\Release\opc.exe"; DestDir: "{app}"; Flags: ignoreversion
; DLL is bundled inside the EXEs via PyInstaller --add-data

[Icons]
Name: "{group}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"
Name: "{group}\Uninstall {#MyAppName}"; Filename: "{uninstallexe}"
Name: "{commondesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"

[Run]
Filename: "{app}\{#MyAppExeName}"; Description: "Launch {#MyAppName}"; Flags: postinstall nowait skipifsilent

[UninstallRun]
Filename: "{app}\{#MyAppExeName}"; Parameters: "--unregister"; Flags: runhidden

[UninstallDelete]
Type: dirifempty; Name: "{app}"

[Code]
const
  EnvKey = 'Environment';

procedure EnvAddPath(InstallPath: string);
var
  Paths: string;
begin
  if not RegQueryStringValue(HKCU, EnvKey, 'Path', Paths) then
    Paths := '';
  if Pos(';' + LowerCase(InstallPath), ';' + LowerCase(Paths)) = 0 then
  begin
    if Paths <> '' then
      Paths := Paths + ';';
    Paths := Paths + InstallPath;
    RegWriteExpandStringValue(HKCU, EnvKey, 'Path', Paths);
  end;
end;

procedure EnvRemovePath(InstallPath: string);
var
  Paths: string;
  P: Integer;
begin
  if not RegQueryStringValue(HKCU, EnvKey, 'Path', Paths) then
    Exit;
  P := Pos(';' + LowerCase(InstallPath), ';' + LowerCase(Paths));
  if P = 0 then
    Exit;
  Delete(Paths, P - 1, Length(InstallPath) + 1);
  RegWriteExpandStringValue(HKCU, EnvKey, 'Path', Paths);
end;

procedure CurStepChanged(CurStep: TSetupStep);
begin
  if (CurStep = ssPostInstall) and WizardIsTaskSelected('addtopath') then
    EnvAddPath(ExpandConstant('{app}'));
end;

procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
begin
  if CurUninstallStep = usPostUninstall then
    EnvRemovePath(ExpandConstant('{app}'));
end;
