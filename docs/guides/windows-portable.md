# The portable variant of Phocommander

The portable variant runs from a USB stick, on a machine that belongs to somebody else, without
an installation. It is the same executable as the installed program: the mode is decided by the
presence of a marker file beside the executable.

The requirements that describe it are FR-22m to FR-22q of the specification and section 7.5 of
[`../REQUIREMENTS.md`](../REQUIREMENTS.md).

## What is built, and from what

The variant is **not a second build**. It is the executable of the desktop build, plus the file
`portable.txt` beside it. The code that decides the mode is in the platform crate:
[`../../platform/phocommander-platform/src/portable.rs`](../../platform/phocommander-platform/src/portable.rs).
The name of the marker file is a constant of that module, `PORTABLE_MARKER_FILE_NAME`, and the
name of the configuration directory beside the executable is `PORTABLE_CONFIG_DIRECTORY_NAME`.

The build is the build of the desktop program:

```bash
./scripts/cargo.sh build --release -p phocommander-desktop --target x86_64-pc-windows-msvc
```

The directory that becomes the portable copy:

```text
Phocommander-0.1.0-x64-portable/
├── phocommander-desktop.exe     the program
└── portable.txt                  the marker of the portable mode
```

The marker is an empty file. Its name is what matters, not its content — a user who copies the
directory to a stick should not have to know anything about it.

```powershell
$portable = "Phocommander-0.1.0-x64-portable"
New-Item -ItemType Directory -Force -Path $portable | Out-Null
Copy-Item "target\x86_64-pc-windows-msvc\release\phocommander-desktop.exe" $portable
New-Item -ItemType File -Force -Path "$portable\portable.txt" | Out-Null
Compress-Archive -Path $portable -DestinationPath "Phocommander-0.1.0-x64-portable.zip"
```

The checksum of the archive is published with the release:

```bash
sha256sum Phocommander-0.1.0-x64-portable.zip
```

## How the user installs the variant, and how they remove it

There is no installation. The user unpacks the archive wherever they want — on a stick, in a
folder of the desktop, in a network drive — and starts the program.

```powershell
Expand-Archive -Path Phocommander-0.1.0-x64-portable.zip -DestinationPath D:\Werkzeug
D:\Werkzeug\Phocommander-0.1.0-x64-portable\phocommander-desktop.exe
```

The program writes its configuration into the directory `Phocommander` beside the executable:

```text
D:\Werkzeug\Phocommander-0.1.0-x64-portable\
├── phocommander-desktop.exe
├── portable.txt
└── Phocommander\               the configuration, written by the program
    ├── config.toml
    └── keymap.toml
```

The removal is the deletion of the directory. Nothing else was written: no entry in the
registry, no file in the profile of the user, no entry in the list of the installed programs.
The two exceptions are the ones the user causes: the documents they open, and the trash of the
system when they choose to delete rather than to move to the trash.

## What the variant contains, and what it does not

| In the variant | |
|---|---|
| `phocommander-desktop.exe` | The program — the same file as in the MSIX package, the same signed file. |
| `portable.txt` | The marker. Its presence is the whole declaration. |
| `Phocommander/` | The configuration, written by the program at run time, not by the build. |

| Not in the variant | Why |
|---|---|
| No activation, no licence file | The program is free and has no licence server. A copy on a stick needs no preparation beyond copying the directory. |
| No installer, no registry entry | The program installs nothing and changes no configuration of the machine. |
| No service, no scheduled task | Nothing starts when the user logs in. |
| No update check, no network call at the start | The program opens no socket when it starts (FR-22q). |
| No second binary | The installed and the portable variant share one executable; only the marker decides the mode (FR-22p). |

The variant has every function of the installed program. It is the same code path — the mode
decides only where the configuration lives.

## The signature

The executable of the portable variant is the **signed** executable of the release, the same file
that the MSIX package carries. The signature is what a machine with an application control
policy looks at, and a portable copy is exactly the case that such a policy watches: an
executable of an unknown source on a removable drive.

```powershell
signtool verify /pa /v "phocommander-desktop.exe"
```

The signing happens once, on the executable of the release. The MSIX package and the portable
archive carry the same signed file; the package adds the manifest and the logos around it, the
portable archive adds the marker file.

A portable copy that is not signed still runs on a machine without a policy. On a machine with
one it is blocked, and that is the correct answer: the policy asks for a signature, and the
signature is there.
