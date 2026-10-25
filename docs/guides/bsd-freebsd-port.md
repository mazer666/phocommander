# The port of Phocommander for FreeBSD

The program is installed on FreeBSD by a port — a directory of the ports tree with a
`Makefile`, a `distinfo`, a `pkg-descr` and a `pkg-plist`. The port builds the program from the
sources of the release with the Rust compiler of the tree and installs the files of the
application into the system.

This guide describes what the port looks like. The port itself is not part of this repository:
a port is submitted to the ports tree by its own maintainer, and the ports tree is a project of
its own.

## The place of the port in the tree

```text
graphics/phocommander/
├── Makefile        the build of the port
├── distinfo        the names and the sums of the fetched files
├── pkg-descr       the description for the software centre of the system
└── pkg-plist       the list of the installed files
```

`graphics` is the category of the program: it is a program that works with files and with
pictures.

## The Makefile

```make
PORTNAME=	phocommander
DISTVERSION=	0.1.0
CATEGORIES=	graphics

MAINTAINER=	ports@phocommander.invalid
COMMENT=	Two-pane file manager with a built-in viewer
WWW=		https://github.com/mazer666/phocommander

LICENSE=	MIT APACHE20
LICENSE_COMB=	dual
LICENSE_FILE_MIT=	${WRKSRC}/LICENSE-MIT
LICENSE_FILE_APACHE20=	${WRKSRC}/LICENSE-APACHE

# The window of the program is drawn with winit. The libraries of the display
# server are loaded at run time, so they are not NEEDED at build time.
LIB_DEPENDS=	libwayland-client.so:graphics/wayland \
		libxkbcommon.so:x11/libxkbcommon

USES=		cargo desktop-file-utils xorg
USE_XORG=	x11 xcb xcursor xi
USE_GITHUB=	yes
GH_ACCOUNT=	mazer666
GH_TAGNAME=	v${DISTVERSION}

# The workspace of the project holds more than one program. The port builds
# the desktop program and nothing else.
CARGO_BUILD_ARGS=	-p phocommander-desktop

# The framework of the ports would install the program with "cargo install".
# The port installs the files itself, because the desktop entry, the icon and
# the description belong to the program as well.
CARGO_INSTALL=	no

APP_ID=		io.github.mazer666.phocommander

do-install:
	${INSTALL_PROGRAM} ${CARGO_TARGET_DIR}/release/phocommander-desktop \
		${STAGEDIR}${PREFIX}/bin
	${INSTALL_DATA} ${WRKSRC}/assets/${APP_ID}.desktop \
		${STAGEDIR}${PREFIX}/share/applications
	@${MKDIR} ${STAGEDIR}${PREFIX}/share/metainfo
	${INSTALL_DATA} ${WRKSRC}/assets/${APP_ID}.metainfo.xml \
		${STAGEDIR}${PREFIX}/share/metainfo/${APP_ID}.metainfo.xml
	@${MKDIR} ${STAGEDIR}${PREFIX}/share/icons/hicolor/scalable/apps
	@${MKDIR} ${STAGEDIR}${PREFIX}/share/icons/hicolor/512x512/apps
	${INSTALL_DATA} ${WRKSRC}/assets/phocommander.svg \
		${STAGEDIR}${PREFIX}/share/icons/hicolor/scalable/apps/${APP_ID}.svg
	${INSTALL_DATA} ${WRKSRC}/assets/phocommander.png \
		${STAGEDIR}${PREFIX}/share/icons/hicolor/512x512/apps/${APP_ID}.png

.include <bsd.port.mk>
```

`USES=cargo` brings the Rust compiler of the tree (`lang/rust`) and the rules of the build. The
list of the crates that the program needs is not written by hand; the framework generates it:

```bash
cd /usr/ports/graphics/phocommander
make cargo-crates
```

The result is the long list `CARGO_CRATES` in the `Makefile`, one crate per line, with the
version. The framework fetches those crates, checks their sums and builds the program against
them, so that the build of the port does not depend on the state of the registry of crates.

`USES=desktop-file-utils` brings the tool that checks the desktop entry. `USE_XORG=x11 xcb
xcursor xi` names the parts of X11 that the window needs.

## The other files of the port

`pkg-descr` — the description that `pkg search` and the software centre of the system show:

```text
Phocommander is a two-pane file manager: two directory panels side by side, and
the operations that work on them — copying, moving, renaming, deleting. The
viewer for images, texts and archives is part of the program.

The program is free, needs no account, opens no network connection and collects
nothing.
```

`pkg-plist` — the list of the files that the port installs. The framework compares the list
with what is really installed, and a difference is an error of the build:

```text
bin/phocommander-desktop
share/applications/io.github.mazer666.phocommander.desktop
share/icons/hicolor/512x512/apps/io.github.mazer666.phocommander.png
share/icons/hicolor/scalable/apps/io.github.mazer666.phocommander.svg
share/metainfo/io.github.mazer666.phocommander.metainfo.xml
```

`distinfo` — the names and the sums of the fetched files, written by the framework:

```bash
make distinfo
make makesum
```

## How the user installs the package, and how they remove it

From the binary repository of the system:

```bash
sudo pkg install phocommander
```

From the ports tree, when the package is not in the repository yet:

```bash
cd /usr/ports/graphics/phocommander
sudo make install
```

The removal:

```bash
sudo pkg remove phocommander
```

or, from the ports tree:

```bash
sudo make deinstall
```

The configuration of the application stays in the profile of the user, in
`~/.config/phocommander` — it is the data of the user and not part of the package.

## What the package contains, and what it does not

| In the package | |
|---|---|
| `bin/phocommander-desktop` | The program. |
| `share/applications/io.github.mazer666.phocommander.desktop` | The desktop entry: the name in the menu of the system. |
| `share/icons/hicolor/…/io.github.mazer666.phocommander.*` | The icon. |
| `share/metainfo/io.github.mazer666.phocommander.metainfo.xml` | The description for the software centre. |

| Not in the package | Why |
|---|---|
| No service, no daemon, no `/etc/rc.d` script | The program is a program that the user starts. Nothing starts when the user logs in. |
| No file in `/etc` | The configuration belongs to the user and lives in `~/.config/phocommander`. |
| No setuid program, no helper with higher rights | The program never asks for administrator rights. |
| No update service | An update is a download that the user starts; the repository of the system offers the update. |
| No telemetry client | Nothing is collected (FR-03). |

## The signature

The port itself is not signed. The packages of the ports tree are signed by the tree, with the
key of the FreeBSD project, and the program `pkg` checks that signature before it installs
anything:

```bash
pkg check -a
```

The chain of trust is the one of the system: the user trusts the key of the ports tree, the
tree signs the package, and the package manager verifies the signature. The project does not
need a second signature of its own — the sums in the `distinfo` of the port already bind the
sources to the build.
