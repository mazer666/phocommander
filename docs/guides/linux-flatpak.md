# The Flatpak of Phocommander

The Flatpak is the form of the application for the Linux systems that carry a software centre
with the sandbox of Flatpak. The program runs in a sandbox that only sees what the manifest
allows it to see, and the manifest is the place where that is written down.

## What is built, and from what

The Flatpak is built by `flatpak-builder` from the manifest
[`../../assets/io.github.mazer666.phocommander.yml`](../../assets/io.github.mazer666.phocommander.yml).
The manifest builds the program from the working tree with the Rust toolchain of the SDK
extension `org.freedesktop.Sdk.Extension.rust-stable`, and it installs the program, the desktop
entry, the icon and the description into the package.

```bash
flatpak-builder --user --install --force-clean build \
    assets/io.github.mazer666.phocommander.yml
```

The result is a package in the local repository of the user, under the identifier
`io.github.mazer666.phocommander`.

For a release, the source of the manifest is the archive of the release instead of the working
tree. The line that changes is the `sources` list of the module:

```yaml
    sources:
      - type: archive
        url: https://github.com/mazer666/phocommander/archive/refs/tags/v0.1.0.tar.gz
        sha256: <the sum of the archive, published with the release>
```

The sum is computed once, on the machine that publishes the release:

```bash
sha256sum phocommander-0.1.0.tar.gz
```

A second source belongs to the manifest for the crates. `flatpak-builder-tools` writes the list
of the crates of `Cargo.lock` into a file, and the build then fetches the crates from that list
instead of from the network:

```bash
flatpak-builder-tools generate --output cargo-sources.json
```

With that file in the `sources` list, the build command changes from `cargo fetch` to
`cargo --offline fetch`, and the build of the Flatpak no longer depends on the network at all.

## The sandbox

The permissions of the sandbox are the `finish-args` of the manifest:

```yaml
finish-args:
  - --share=ipc
  - --socket=wayland
  - --socket=fallback-x11
  - --device=dri
  - --filesystem=home
```

| Permission | Why the program needs it |
|---|---|
| `--share=ipc` | The shared memory of X11, which the window of the program uses. |
| `--socket=wayland` | The display server of the system. |
| `--socket=fallback-x11` | The fallback to X11 for the systems that have no Wayland session. |
| `--device=dri` | The graphics card, for the drawing of the interface. |
| `--filesystem=home` | The files of the user — that is the work of a file manager — and the configuration of the program in `~/.config/phocommander`. |

**There is no `--share=network`.** The program opens no connection when it starts and sends
nothing anywhere (FR-02, FR-03). A permission that the program does not use would be a
permission that the user of the software centre cannot check, and the specification asks for no
permission that the program does not use (NFR-27).

The same holds for the rest: the manifest asks for no portal, no session bus, no camera, no
microphone, no location and no contacts.

## How the user installs the package, and how they remove it

From the software centre of the distribution, the package appears under the name of the
description file. From the command line:

```bash
flatpak install --user -r flathub io.github.mazer666.phocommander
```

A package that was built locally is installed with the identifier alone:

```bash
flatpak install --user io.github.mazer666.phocommander
```

The removal:

```bash
flatpak uninstall --user io.github.mazer666.phocommander
```

The removal takes the program away. The configuration of the user stays in
`~/.config/phocommander` — it is the data of the user and not part of the package. A user who
wants it gone as well deletes that directory.

## What the package contains, and what it does not

| In the package | |
|---|---|
| `/app/bin/phocommander-desktop` | The program. |
| `/app/share/applications/io.github.mazer666.phocommander.desktop` | The desktop entry, with the name in the menu of the system. |
| `/app/share/icons/hicolor/scalable/apps/io.github.mazer666.phocommander.svg` | The icon. |
| `/app/share/metainfo/io.github.mazer666.phocommander.metainfo.xml` | The description for the software centre. |

| Not in the package | Why |
|---|---|
| No network permission | The program opens no connection and sends nothing. |
| No service, no autostart entry, no system daemon | The program is a program that the user starts; nothing starts when the user logs in. |
| No update check | An update is a download that the user starts. The software centre of the distribution offers the update, and it asks first. |
| No telemetry, no crash reporter, no advertising identifier | Nothing is collected (FR-03). |
| No second copy of the program | The package carries the program of the desktop build, unchanged. |

## The signature

The Flatpak of the project is **not signed by the project**. A Flatpak is signed by the
repository that serves it, with the key of that repository; the software centre checks the
signature of the repository, not the signature of the application.

That is why the checksum of the release is enough for the project:

```bash
sha256sum phocommander-0.1.0.flatpak
```

The sum is published with the release, and the user verifies it before installing a package
that did not come from a software centre. For a package that goes into the software centre of
the distribution, the key of that centre signs it — the project hands over the package and the
sum, and the centre takes the responsibility for the key.
