# The packages of the Linux distributions

The packages of the distributions are not built by the project. They are built by the
distributions, from the sources of the release, with their own key and their own rules. This
guide answers two questions: **which packages exist**, and **what a distributor needs** to
build one.

## Which packages exist

| Family | The package of the distribution | The tool that builds it | The command |
|---|---|---|---|
| Debian, Ubuntu | `deb` | `dpkg-buildpackage` with the `dh` build system of debhelper | `dpkg-buildpackage -b -us -uc` |
| Fedora, RHEL, openSUSE | `rpm` | `rpmbuild` | `rpmbuild -ba phocommander.spec` |
| Arch | `pacman` package | `makepkg` | `makepkg -si` |
| Alpine | `apk` | `abuild` | `abuild -r` |

The two forms that do not need a distributor are the Flatpak
([`linux-flatpak.md`](linux-flatpak.md)) and the AppImage
([`linux-appimage.md`](linux-appimage.md)). They are built by the project and they work on every
system that has the corresponding tool.

## What the project hands over

The release carries the sources of the program and the files that describe it. A distributor
needs exactly these:

| File | What it is |
|---|---|
| The archive of the sources | The repository at the tag of the release, with `Cargo.lock`. |
| `assets/io.github.mazer666.phocommander.desktop` | The desktop entry. |
| `assets/phocommander.svg`, `assets/phocommander.png` | The icon. |
| `assets/io.github.mazer666.phocommander.metainfo.xml` | The description for the software centre. |
| `LICENSE-MIT`, `LICENSE-APACHE` | The licences, `MIT OR Apache-2.0`. |
| The sum of the archive | The checksum that the package manager of the distribution checks. |

The binary itself is built by the distributor, on the architecture of the distribution, from the
sources of the release. The project does not hand over a binary that was built somewhere else.

## The file list of a package

Every distribution installs the same files:

```text
/usr/bin/phocommander-desktop
/usr/share/applications/io.github.mazer666.phocommander.desktop
/usr/share/icons/hicolor/scalable/apps/io.github.mazer666.phocommander.svg
/usr/share/icons/hicolor/512x512/apps/io.github.mazer666.phocommander.png
/usr/share/doc/phocommander/README.md        (the name of the distribution)
```

And installs nothing else:

| Not installed | Why |
|---|---|
| No service, no daemon, no socket for the system | The program is a program that the user starts. Nothing starts when the user logs in. |
| No autostart entry in `~/.config/autostart` or in `/etc/xdg/autostart` | The program does not start itself. |
| No file in `/etc` | The configuration belongs to the user and lives in `~/.config/phocommander`. |
| No policy of polkit, no helper with higher rights | The program never asks for administrator rights. |
| No update service | An update is a download that the user starts (FR-02). |
| No telemetry client | Nothing is collected (FR-03). |

Because there is no service and no autostart entry, the removal of the package is complete:
the package manager removes the files, and the configuration of the user stays in
`~/.config/phocommander` — it is the data of the user and not part of the package.

## The recipes

The four recipes below build the packages from the sources of the release. They are complete
and they are meant to be copied into the tree of the distribution.

### Debian and Ubuntu

```text
debian/control
debian/changelog
debian/rules
debian/copyright
debian/install
```

`debian/control`:

```text
Source: phocommander
Section: utils
Priority: optional
Maintainer: Phocommander <packages@phocommander.invalid>
Build-Depends: debhelper-compat (= 13), cargo, rustc
Standards-Version: 4.6.2
Homepage: https://github.com/mazer666/phocommander
Rules-Requires-Root: no

Package: phocommander
Architecture: any
Depends: ${shlibs:Depends}, ${misc:Depends}
Description: Two-pane file manager with a built-in viewer
 Phocommander shows two directory panels side by side and works on them:
 copying, moving, renaming and deleting files. The viewer for images, texts
 and archives is part of the program.
 .
 The program is free, needs no account, opens no network connection and
 collects nothing.
```

`debian/rules`:

```make
#!/usr/bin/make -f

%:
	dh $@ --buildsystem=cargo
```

`debian/install`:

```text
target/release/phocommander-desktop usr/bin
assets/io.github.mazer666.phocommander.desktop usr/share/applications
assets/phocommander.svg usr/share/icons/hicolor/scalable/apps
assets/phocommander.png usr/share/icons/hicolor/512x512/apps
```

The build and the installation:

```bash
dpkg-buildpackage -b -us -uc
sudo apt install ./phocommander_0.1.0-1_amd64.deb
sudo apt remove phocommander
```

### Fedora, RHEL and openSUSE

`phocommander.spec`:

```spec
Name:           phocommander
Version:        0.1.0
Release:        1%{?dist}
Summary:        Two-pane file manager with a built-in viewer
License:        MIT OR Apache-2.0
URL:            https://github.com/mazer666/phocommander
Source0:        %{name}-%{version}.tar.gz

BuildRequires:  cargo
BuildRequires:  rust
BuildRequires:  desktop-file-utils

%description
Phocommander shows two directory panels side by side and works on them:
copying, moving, renaming and deleting files. The viewer for images, texts
and archives is part of the program.

The program is free, needs no account, opens no network connection and
collects nothing.

%prep
%autosetup

%build
cargo build --release --locked

%install
install -Dm755 target/release/phocommander-desktop %{buildroot}%{_bindir}/phocommander-desktop
install -Dm644 assets/io.github.mazer666.phocommander.desktop %{buildroot}%{_datadir}/applications/io.github.mazer666.phocommander.desktop
install -Dm644 assets/phocommander.svg %{buildroot}%{_datadir}/icons/hicolor/scalable/apps/io.github.mazer666.phocommander.svg
install -Dm644 assets/phocommander.png %{buildroot}%{_datadir}/icons/hicolor/512x512/apps/io.github.mazer666.phocommander.png

%files
%license LICENSE-MIT LICENSE-APACHE
%doc README.md
%{_bindir}/phocommander-desktop
%{_datadir}/applications/io.github.mazer666.phocommander.desktop
%{_datadir}/icons/hicolor/scalable/apps/io.github.mazer666.phocommander.svg
%{_datadir}/icons/hicolor/512x512/apps/io.github.mazer666.phocommander.png

%changelog
```

The build and the installation:

```bash
rpmbuild -ba phocommander.spec
sudo dnf install phocommander-0.1.0-1.fc*.rpm
sudo dnf remove phocommander
```

### Arch

`PKGBUILD`:

```bash
pkgname=phocommander
pkgver=0.1.0
pkgrel=1
pkgdesc="Two-pane file manager with a built-in viewer"
arch=('x86_64' 'aarch64')
url="https://github.com/mazer666/phocommander"
license=('MIT' 'Apache')
depends=('xorg-server')
makedepends=('cargo' 'rust')
source=("https://github.com/mazer666/phocommander/archive/v$pkgver.tar.gz")
sha256sums=('...')

build() {
	cargo build --release --locked
}

package() {
	install -Dm755 target/release/phocommander-desktop "$pkgdir/usr/bin/phocommander-desktop"
	install -Dm644 assets/io.github.mazer666.phocommander.desktop "$pkgdir/usr/share/applications/io.github.mazer666.phocommander.desktop"
	install -Dm644 assets/phocommander.svg "$pkgdir/usr/share/icons/hicolor/scalable/apps/io.github.mazer666.phocommander.svg"
	install -Dm644 assets/phocommander.png "$pkgdir/usr/share/icons/hicolor/512x512/apps/io.github.mazer666.phocommander.png"
}
```

The build and the installation:

```bash
makepkg -si
sudo pacman -R phocommander
```

### Alpine

`APKBUILD`:

```bash
pkgname=phocommander
pkgver=0.1.0
pkgrel=0
pkgdesc="Two-pane file manager with a built-in viewer"
arch="x86_64 aarch64"
url="https://github.com/mazer666/phocommander"
license="MIT Apache-2.0"
depends="rust cargo"
makedepends=""

build() {
	cargo build --release --locked
}

package() {
	install -Dm755 target/release/phocommander-desktop "$pkgdir/usr/bin/phocommander-desktop"
	install -Dm644 assets/io.github.mazer666.phocommander.desktop "$pkgdir/usr/share/applications/io.github.mazer666.phocommander.desktop"
	install -Dm644 assets/phocommander.svg "$pkgdir/usr/share/icons/hicolor/scalable/apps/io.github.mazer666.phocommander.svg"
	install -Dm644 assets/phocommander.png "$pkgdir/usr/share/icons/hicolor/512x512/apps/io.github.mazer666.phocommander.png"
}
```

The build and the installation:

```bash
abuild -r
sudo apk add ./phocommander-0.1.0-r0.apk
sudo apk del phocommander
```

## The signature

The packages of the distributions are signed by the distribution, with the key of the
distribution. The project does not sign them and does not need to: the package manager of the
system checks the signature of the repository of the distribution against the key that is
already in the system, and that is the trust the user has bought with the distribution.

What the project publishes beside the sources is the checksum of the archive. It is the
provenance of the release that the distributor builds from:

```bash
sha256sum phocommander-0.1.0.tar.gz
```

A distributor who builds a package from the release verifies that sum before the build, and the
package that comes out of it carries the signature of the distribution. The chain from the
release to the installed program is complete without a second signature from the project.
