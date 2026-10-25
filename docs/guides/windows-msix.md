# The MSIX package of Phocommander

The MSIX package is the installed form of the application on Windows. It is the form that the
Windows operating system itself installs, updates and removes, and the form that a machine with
an application control admits — because the package is signed.

## What is built, and from what

The package carries the executable of the desktop build. The build of the program is the same
build that every platform uses; the package adds nothing to the program and takes nothing away
from it.

```bash
./scripts/cargo.sh build --release -p phocommander-desktop --target x86_64-pc-windows-msvc
```

The result is `target/x86_64-pc-windows-msvc/release/phocommander-desktop.exe`.

The layout of the package is a directory. The tool that packs it is `MakeAppx.exe` from the
Windows SDK; the tool that signs it is `SignTool.exe` from the same SDK.

```text
Phocommander.msix-layout/
├── AppxManifest.xml                    the manifest of the package
├── phocommander-desktop.exe            the program
└── Assets/
    ├── Square44x44Logo.png             the icon in the menu and in the task bar
    ├── Square150x150Logo.png           the icon of the tile
    ├── StoreLogo.png                   the icon in the software centre
    └── Wide310x150Logo.png             the banner of the tile
```

The manifest is [`../../assets/AppxManifest.xml`](../../assets/AppxManifest.xml), the logos are
in [`../../assets/msix/`](../../assets/msix). The layout is assembled like this:

```powershell
$layout = "Phocommander.msix-layout"
New-Item -ItemType Directory -Force -Path "$layout\Assets" | Out-Null
Copy-Item "target\x86_64-pc-windows-msvc\release\phocommander-desktop.exe" "$layout\"
Copy-Item "assets\AppxManifest.xml" "$layout\"
Copy-Item "assets\msix\*.png" "$layout\Assets\"
```

The packer turns the layout into the package:

```powershell
makeappx pack /d $layout /p "Phocommander.msix" /o
```

The `/o` says that an existing file of the same name is overwritten. The packer checks the
manifest against the files of the layout while it packs, and it refuses a package whose logo or
whose program is missing.

The version of the manifest and the version of the build belong together. The manifest of the
repository carries the version of the workspace; a release raises both.

## The long path manifest

The executable in the package carries an application manifest that declares the program as
`longPathAware`, so that paths longer than `MAX_PATH` work. The declaration is not made by the
packaging — it is made by the build.

The manifest is the file
[`../../apps/phocommander-desktop/windows.manifest`](../../apps/phocommander-desktop/windows.manifest),
and the build script of the application
([`../../apps/phocommander-desktop/build.rs`](../../apps/phocommander-desktop/build.rs)) writes
it into the executable as the resource 1 of the type 24 (`RT_MANIFEST`). The script needs
nothing but the compiler and the linker, so the build of the Windows port does not depend on
the tools of the Windows SDK.

The same manifest declares that the program asks for no administrator rights:

```xml
<requestedExecutionLevel level="asInvoker" uiAccess="false"/>
```

`asInvoker` means: the program runs with the rights of the user who starts it. A machine with
an application control policy watches for the other branch — "run as administrator" — and this
program never takes it.

The package cannot be built without the manifest, because the requirement FR-28 asks the
program to report a path that is too long in plain language, and the program can only reach
such a path when the operating system lets it.

**The long paths of the system also have to be switched on.** The declaration of the program
is only one of the two gates; the setting `LongPathsEnabled` of the system is the other one.
The application does not change that setting — it is a change of the configuration of the
machine, which the product forbids (NFR-28).

## The signature

A package can only be installed when it is signed. The signature binds two things: the
publisher of the package and the content of the package.

**The publisher of the manifest must be the subject of the certificate, character by character.**
The manifest of the repository carries

```xml
<Identity Name="Phocommander" Publisher="CN=Phocommander, O=Phocommander" Version="0.1.0.0" ProcessorArchitecture="x64"/>
```

When the owner holds a certificate with a different subject, this one attribute in
[`../../assets/AppxManifest.xml`](../../assets/AppxManifest.xml) is changed to the subject of the
certificate — and to nothing else.

The signing itself:

```powershell
signtool sign /fd SHA256 /td SHA256 /tr http://timestamp.digicert.com /a `
    /f "C:\certs\phocommander.pfx" /p "%PASSWORD%" "Phocommander.msix"
```

The timestamp server proves that the signature was made while the certificate was valid, so
that the package stays valid after the certificate expires.

The signature is verified before the package is published (NFR-26):

```powershell
signtool verify /pa /v "Phocommander.msix"
```

The exit code of the command decides whether the release happens. A package that is published
without a verified signature is a defect, not a release.

For a test on a machine of the development, a self-signed certificate is enough. It has to be
trusted in the store *Trusted People* of that machine, and the package is installed with the
publisher of that certificate.

## How the user installs the package, and how they remove it

```powershell
Add-AppxPackage -Path .\Phocommander.msix
```

The package appears in the menu of the system under the name of the manifest, and it runs from
the protected directory of the package. The application asks for no administrator rights, and
the installation needs none either.

The removal takes the identity of the installed package, which the system prints:

```powershell
Get-AppxPackage -Name Phocommander
Remove-AppxPackage -Package <the full name that the command above printed>
```

The removal takes the program and the files of the package away. The configuration of the
application stays in the profile of the user, in `%APPDATA%\Phocommander` — it is the user's
data and not part of the package.

## What the package contains, and what it does not

| In the package | |
|---|---|
| `phocommander-desktop.exe` | The program. The signed executable of the desktop build. |
| `AppxManifest.xml` | The identity, the capability and the logos. |
| `Assets/*.png` | The four logos of the manifest. |

| Not in the package | Why |
|---|---|
| No service, no scheduled task, no driver | The program never asks for administrator rights, and nothing starts when the user logs in. |
| No registry entry | The program does not change the configuration of the machine. |
| No update check | An update is a download that the user starts (FR-02). |
| No telemetry, no crash reporter | Nothing is collected (FR-03). |
| No licence file, no activation | The program is free and has no licence server (section 7.5 of the specification). |
| No second copy of the program | The portable variant is the same executable with a marker file beside it, not a second build. |

The capability of the package is exactly one:

```xml
<Capabilities>
  <rescap:Capability Name="runFullTrust" />
</Capabilities>
```

`runFullTrust` is what a desktop program needs, and it is the only thing the package asks for.
It grants no file system access on its own: the program runs with the rights of the user who
starts it, and it reaches the files that the user opens.

## The portable variant

The Windows build ships in a second form, the portable variant: the same executable with the
file `portable.txt` beside it. The description is in
[`windows-portable.md`](windows-portable.md).
