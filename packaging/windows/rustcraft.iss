; Installs a release bundle (the files of rustcraft-client-windows-x64.zip) for the
; current user, into a folder the client's updater can write to.
;
; usage: iscc /DAppVersion=<x.y.z> /DBundle=<bundle-dir> /DOutDir=<out-dir> /DOutName=<name> rustcraft.iss

[Setup]
AppId={{22F91093-7D94-4FA4-9045-1F5CBA3D82D5}
AppName=Rustcraft
AppVersion={#AppVersion}
AppPublisher=RustcraftMC
AppPublisherURL=https://github.com/RustcraftMC/Client
DefaultDirName={localappdata}\Programs\Rustcraft
DisableProgramGroupPage=yes
PrivilegesRequired=lowest
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
OutputDir={#OutDir}
OutputBaseFilename={#OutName}
SetupIconFile=..\..\rustcraft-client\assets\icon.ico
UninstallDisplayIcon={app}\rustcraft-client.exe
WizardStyle=modern
Compression=lzma2
SolidCompression=yes

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
Source: "{#Bundle}\*"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\Rustcraft"; Filename: "{app}\rustcraft-client.exe"
Name: "{autodesktop}\Rustcraft"; Filename: "{app}\rustcraft-client.exe"; Tasks: desktopicon

[Run]
Filename: "{app}\rustcraft-client.exe"; Description: "{cm:LaunchProgram,Rustcraft}"; Flags: nowait postinstall skipifsilent

; What the updater leaves behind; the data folder (%LOCALAPPDATA%\Rustcraft) stays.
[UninstallDelete]
Type: files; Name: "{app}\*.old"
Type: filesandordirs; Name: "{app}\.rustcraft-update"
