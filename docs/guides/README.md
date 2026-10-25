# The packaging of Phocommander

This directory holds the guides that describe how the application is packaged for the three
desktop platforms. A guide answers four questions for every form:

1. **What is built, and from what** — the command and the file.
2. **How the user installs the package, and how they remove it again.**
3. **Which files the package contains** — and which it does not.
4. **How the signature is handled.**

## The rules that hold for every form

The rules are not a checklist that each guide repeats. They are properties of the product, and
every package has to keep them.

| Rule | Where it comes from | What it means for a package |
|---|---|---|
| The application is free, and no feature is behind a payment | FR-01 | The package may be sold for nothing, and no file of the package asks for money. |
| The application starts without an account and without a network connection | FR-02 | The package contains no login, no licence file and no update check. A package that checks for a new version on start-up breaks this rule. |
| Nothing is collected | FR-03 | The package contains no telemetry client, no crash reporter and no advertising identifier. |
| The application never asks for administrator rights | section 9.4 of the specification | The package installs no service, no scheduled task, no driver and no shell extension. Nothing starts when the user logs in. |
| The application asks for no permission that it does not use | NFR-27 | The manifest of a package lists exactly the capabilities the program uses. The Flatpak manifest is the clearest example: it asks for the home directory, the display sockets and the graphics card, and for nothing else. |
| The application does not change the configuration of the machine | NFR-28 | The installation touches the directories of the package and nothing else. |

A package that breaks one of these rules is not a package of this product. The gates of the
repository cannot see a package, so the rules are written down here and in
[`../REQUIREMENTS.md`](../REQUIREMENTS.md).

## The one build that every package carries

Every form starts with the same program. The build produces one executable, and every package
carries that executable — unchanged, without a second build.

```bash
./scripts/cargo.sh build --release -p phocommander-desktop
```

The result is `target/release/phocommander-desktop`. The name of the binary is not changed by
any of the guides: the desktop entry, the Flatpak manifest and the packages of the distributions
all name `phocommander-desktop`, so that the name in the menu of the system and the name on the
disk cannot drift apart.

The version of a package is the version of the workspace, `0.1.0` at the time of writing. It
stands in `Cargo.toml` and in the manifest of the MSIX package; every other form takes it from
there.

## The forms

| Platform | Form | Guide |
|---|---|---|
| Windows | The MSIX package | [`windows-msix.md`](windows-msix.md) |
| Windows | The portable variant | [`windows-portable.md`](windows-portable.md) |
| Linux | The Flatpak | [`linux-flatpak.md`](linux-flatpak.md) |
| Linux | The AppImage | [`linux-appimage.md`](linux-appimage.md) |
| Linux | The packages of the distributions | [`linux-distribution-packages.md`](linux-distribution-packages.md) |
| FreeBSD | The port of the ports tree | [`bsd-freebsd-port.md`](bsd-freebsd-port.md) |
| NetBSD | The package of the pkgsrc tree | [`bsd-netbsd-pkgsrc.md`](bsd-netbsd-pkgsrc.md) |

## The files that the packages share

The files that describe the application live in [`../../assets/`](../../assets). They stand in
one place, so that the four Linux forms cannot drift apart.

| File | What it is |
|---|---|
| [`../../assets/phocommander.svg`](../../assets/phocommander.svg) | The icon in the vector form. |
| [`../../assets/phocommander.png`](../../assets/phocommander.png) | The icon as a picture of 512 × 512 pixels. |
| [`../../assets/io.github.mazer666.phocommander.desktop`](../../assets/io.github.mazer666.phocommander.desktop) | The desktop entry: the name in the menu of the system, the command and the icon. |
| [`../../assets/io.github.mazer666.phocommander.metainfo.xml`](../../assets/io.github.mazer666.phocommander.metainfo.xml) | The description of the application for the software centres. |
| [`../../assets/io.github.mazer666.phocommander.yml`](../../assets/io.github.mazer666.phocommander.yml) | The manifest of the Flatpak. |
| [`../../assets/AppxManifest.xml`](../../assets/AppxManifest.xml) | The manifest of the MSIX package. |
| [`../../assets/msix/`](../../assets/msix) | The logos of the MSIX package. |

The application identifier `io.github.mazer666.phocommander` is the name of the Flatpak, the
name of the desktop entry and the name of the icon in the icon theme. It is written down once,
in the files above.

## The signature

| Platform | What the signature is for | What happens without one |
|---|---|---|
| Windows | A signed binary is admitted by the application control of a managed machine, and a package can only be installed when it is signed. The signature is part of the release, and the release is only published when the signature has been verified (NFR-26). | The package is refused by Windows. The portable variant runs, but a machine with an application control policy blocks it. |
| Linux | The package is not signed by the project. The checksum of the release file is enough, because the software centre of the distribution checks the package against the list of the distribution. | Nothing changes; the checksum is published with the release. |
| BSD | The ports tree and the pkgsrc tree sign their own packages with the key of the tree. The port itself is not signed. | Nothing changes; the package of the tree carries the signature of the tree. |

The checksum of a release is a SHA-256 sum over the published file. The release page carries the
file and its sum; the sum is what the user verifies.

```bash
sha256sum Phocommander-0.1.0-x86_64.AppImage
```

## The builds of the platforms

The CI matrix builds the program for the desktop platforms. The matrix of the cross-compilation
and the toolchain that the BSD targets need are described in the workflow itself,
[`.github/workflows/ci.yml`](../../.github/workflows/ci.yml).
