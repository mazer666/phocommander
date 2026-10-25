# The package of Phocommander for NetBSD

The program is installed on NetBSD by a package of the pkgsrc tree — the same tree that builds
the packages of NetBSD itself. The package is a directory with a `Makefile`, a `distinfo`, a
`DESCR` and a `PLIST`.

This guide describes what the package looks like. The package itself is not part of this
repository: a package is submitted to the pkgsrc tree by its own maintainer, and the tree is a
project of its own.

## The place of the package in the tree

```text
graphics/phocommander/
├── Makefile            the build of the package
├── cargo-depends.mk    the list of the crates of the program
├── distinfo            the names and the sums of the fetched files
├── DESCR               the description for the software centre of the system
└── PLIST               the list of the installed files
```

The tree of pkgsrc is the same tree on every system: the package is built on NetBSD, on Linux
and on the other systems that use pkgsrc, and the result is a binary package for the system in
question.

## The Makefile

```make
DISTNAME=	phocommander-0.1.0
CATEGORIES=	graphics
MASTER_SITES=	${MASTER_SITE_GITHUB:=mazer666/}
GITHUB_PROJECT=	phocommander
GITHUB_TAG=	v${PKGVERSION_NOREV}

MAINTAINER=	pkgsrc-users@NetBSD.org
HOMEPAGE=	https://github.com/mazer666/phocommander
COMMENT=	Two-pane file manager with a built-in viewer
LICENSE=	mit apache-2.0

# The minimum version of the Rust compiler that the program needs.
RUST_REQ=	1.99.0

# The program compiles C and C++ code — the compression library and the
# library of the locales — so it needs a C and a C++ compiler.
USE_LANGUAGES=	c c++

# The list of the crates. The file is generated, see below.
.include "cargo-depends.mk"

INSTALLATION_DIRS=	bin share/applications share/metainfo \
			share/icons/hicolor/scalable/apps \
			share/icons/hicolor/512x512/apps

# The fragment of the tree builds the program with the Rust compiler and
# installs it. The package installs the files itself, because the desktop
# entry, the icon and the description belong to the program as well.
do-install:
	${INSTALL_PROGRAM} ${WRKSRC}/target/release/phocommander-desktop \
		${DESTDIR}${PREFIX}/bin/phocommander-desktop
	${INSTALL_DATA} ${WRKSRC}/assets/io.github.mazer666.phocommander.desktop \
		${DESTDIR}${PREFIX}/share/applications
	${INSTALL_DATA} ${WRKSRC}/assets/io.github.mazer666.phocommander.metainfo.xml \
		${DESTDIR}${PREFIX}/share/metainfo
	${INSTALL_DATA} ${WRKSRC}/assets/phocommander.svg \
		${DESTDIR}${PREFIX}/share/icons/hicolor/scalable/apps/io.github.mazer666.phocommander.svg
	${INSTALL_DATA} ${WRKSRC}/assets/phocommander.png \
		${DESTDIR}${PREFIX}/share/icons/hicolor/512x512/apps/io.github.mazer666.phocommander.png

.include "../../lang/rust/cargo.mk"
.include "../../mk/bsd.pkg.mk"
```

The fragment `lang/rust/cargo.mk` does three things: it brings the Rust compiler, it fetches
the crates of the program from the registry of crates and checks their sums, and it builds the
program with `--offline` — the build never reaches the network.

The list of the crates is not written by hand. The tree writes it from the `Cargo.lock` of the
program:

```bash
cd /usr/pkgsrc/graphics/phocommander
make extract
make print-cargo-depends > cargo-depends.mk
```

The result is one line per crate in `cargo-depends.mk`:

```make
CARGO_CRATE_DEPENDS+=	anyhow-1.0.100
CARGO_CRATE_DEPENDS+=	cap-std-4.0.3
CARGO_CRATE_DEPENDS+=	...
```

The sums of the fetched files are written into the `distinfo`:

```bash
make makesum
```

## The other files of the package

`DESCR` — the description that `pkg_search` and the software centre of the system show:

```text
Phocommander is a two-pane file manager: two directory panels side by side, and
the operations that work on them — copying, moving, renaming, deleting. The
viewer for images, texts and archives is part of the program.

The program is free, needs no account, opens no network connection and collects
nothing.
```

`PLIST` — the list of the files that the package installs:

```text
bin/phocommander-desktop
share/applications/io.github.mazer666.phocommander.desktop
share/icons/hicolor/512x512/apps/io.github.mazer666.phocommander.png
share/icons/hicolor/scalable/apps/io.github.mazer666.phocommander.svg
share/metainfo/io.github.mazer666.phocommander.metainfo.xml
```

## How the user installs the package, and how they remove it

The built package is installed with `pkg_add`:

```bash
sudo pkg_add phocommander-0.1.0.tgz
```

A user who works with the tree itself installs it from there:

```bash
cd /usr/pkgsrc/graphics/phocommander
sudo pkg_install
```

The removal:

```bash
sudo pkg_delete phocommander
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
| No service, no daemon, no script in `/etc/rc.d` | The program is a program that the user starts. Nothing starts when the user logs in. |
| No file in `/etc` | The configuration belongs to the user and lives in `~/.config/phocommander`. |
| No setuid program, no helper with higher rights | The program never asks for administrator rights. |
| No update service | An update is a download that the user starts. |
| No telemetry client | Nothing is collected (FR-03). |

## The signature

The package is not signed by the project. The packages of the pkgsrc tree are signed by the
tree with the key of the NetBSD project, and `pkg_add` checks that signature before it
installs anything.

What the project publishes beside the sources is the checksum of the archive of the release:

```bash
sha256sum phocommander-0.1.0.tar.gz
```

The sum binds the sources to the build, and the signature of the tree binds the package to the
tree. Together they are the chain of trust of the system, and a second signature of the
project would say nothing that this chain does not already say.
