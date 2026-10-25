# The AppImage of Phocommander

The AppImage is the form of the application for the Linux systems that have no package manager
for the program: the user downloads one file, makes it executable and starts it. Nothing is
installed into the system, and nothing is left behind when the file is deleted.

## What is built, and from what

The build of the AppImage is two steps. The first builds the program, the second packs it.

```bash
./scripts/cargo.sh build --release -p phocommander-desktop
./scripts/build-appimage.sh target/release/phocommander-desktop
```

The script [`../../scripts/build-appimage.sh`](../../scripts/build-appimage.sh) assembles the
`AppDir` — the directory that the packer expects — and calls the `appimagetool` of the
AppImageKit. The result is

```text
target/appimage/Phocommander-0.1.0-x86_64.AppImage
```

The two tools that the script needs are `appimagetool` and `desktop-file-validate`; the second
one belongs to the package `desktop-file-utils` of the distribution. The script says which one
is missing.

### The layout that is packed

```text
Phocommander.AppDir/
├── AppRun                                  the entry point of the image
├── io.github.mazer666.phocommander.desktop the desktop entry
├── io.github.mazer666.phocommander.png     the icon
├── .DirIcon                                the icon of the directory
└── usr/
    ├── bin/phocommander-desktop            the program
    └── share/
        ├── applications/io.github.mazer666.phocommander.desktop
        ├── icons/hicolor/256x256/apps/io.github.mazer666.phocommander.png
        ├── icons/hicolor/scalable/apps/io.github.mazer666.phocommander.svg
        └── metainfo/io.github.mazer666.phocommander.appdata.xml
```

Three files in the root of the `AppDir` are mandatory: the entry point `AppRun`, the desktop
entry and the icon whose name is the value of `Icon=` in the desktop entry. Without them the
packer stops with `Desktop file not found`.

The `AppRun` is a small shell script. It determines the path of the program from its own
location, because the path inside the mounted image is not the path of the build directory:

```sh
#!/bin/sh
HERE="$(dirname "$(readlink -f "$0")")"
exec "${HERE}/usr/bin/phocommander-desktop" "$@"
```

## How the user installs the package, and how they remove it

There is no installation. The user downloads the file, makes it executable and starts it:

```bash
chmod +x Phocommander-0.1.0-x86_64.AppImage
./Phocommander-0.1.0-x86_64.AppImage
```

Most desktop systems offer the file in the menu of the system as well, because the image
carries the desktop entry and the icon. That is what the *AppImage Integration* of the
distribution does; it is a part of the system, not of the package.

The removal is the deletion of the file. Nothing was installed into the system, so nothing has
to be uninstalled.

The configuration of the application stays where the program keeps it — in
`~/.config/phocommander`, unless the directory holds the marker file `portable.txt`, in which
case the program keeps its configuration beside itself. The marker file and the portable mode
are described in [`windows-portable.md`](windows-portable.md); the mode is decided by the
program and not by the package, and the same program runs in the AppImage.

## What the package contains, and what it does not

| In the package | |
|---|---|
| `usr/bin/phocommander-desktop` | The program. |
| `usr/share/applications/….desktop` | The desktop entry. |
| `usr/share/icons/…` | The icon, in two forms. |
| `usr/share/metainfo/…` | The description for the software centres. |

| Not in the package | Why |
|---|---|
| No system library of the distribution | The image carries the program; the system libraries of the host are used, and the program is built against them. |
| No service, no autostart entry, no daemon | Nothing starts when the user logs in. |
| No update check | The program opens no connection; the update is a download that the user starts. |
| No telemetry, no crash reporter | Nothing is collected (FR-03). |
| No activation, no account | The program is free and needs no preparation. |

## The signature

The AppImage of the project is **not signed by the project**. The checksum of the file is
enough, and it is published with the release:

```bash
sha256sum Phocommander-0.1.0-x86_64.AppImage
```

The user compares the sum with the one on the release page before starting the file. The
signature of a distribution has a different job: the maintainer of a distribution signs the
package they build from the release, and the package manager of the system checks that
signature against the key of the distribution. That is the chain of trust of the distribution,
and it does not need a second chain from the project.

The file itself carries no signature, and that is deliberate. An AppImage runs from a directory
the user owns; a signature inside the file would only say that the file was not changed since
it was signed — which the checksum already says, in a way that every system can check without
an extra tool.
