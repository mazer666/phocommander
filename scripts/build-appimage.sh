#!/usr/bin/env bash
#
# Das AppImage von Phocommander.
#
# WHAT: legt das AppDir der Anwendung an und packt es mit dem appimagetool
#       in eine einzige Datei.
# WHY:  das AppImage ist die Form, in der die Anwendung ohne Installation
#       auf einem fremden System laeuft. Die Form ist im Wesentlichen ein
#       Verzeichnis mit drei Pflichtdateien — dem Eintrag `AppRun`, der
#       Schreibtischdatei und dem Symbol — das der appimagetool zu einer
#       Datei zusammenpackt. Das Skript baut genau dieses Verzeichnis auf,
#       damit die drei Linux-Formen (Flatpak, AppImage, Pakete der
#       Distributionen) nicht drei verschiedene Kopien der Dateiliste
#       fuehren.
#
#       Das Skript ruft das Werkzeug `cargo` nicht auf. Der Befehl, der das
#       Programm baut, steht in der Anleitung; das Skript nimmt die fertige
#       Datei entgegen. So laesst sich das AppImage auch auf einer Maschine
#       bauen, auf der kein Rust-Werkzeug installiert ist.
#
#       Die Voraussetzungen sind das `appimagetool` des AppImageKit und das
#       `desktop-file-validate`, das der appimagetool braucht, um die
#       Schreibtischdatei zu pruefen. Fehlt eines von beiden, bricht der
#       Packschritt ab — das Skript sagt dann, welches fehlt.
#
# EXAMPLE:
#     ./scripts/cargo.sh build --release -p phocommander-desktop
#     ./scripts/build-appimage.sh target/release/phocommander-desktop
#
# Exit code 0 — das AppImage liegt in dem Ausgabeverzeichnis.
# Exit code 1 — eine Voraussetzung fehlt (Programm, Symbol, Werkzeug).

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

# ---------------------------------------------------------------------------
# Die Kennungen. Sie stehen an genau einer Stelle, weil Flatpak, AppImage
# und die Pakete der Distributionen dieselben Namen benutzen.
# ---------------------------------------------------------------------------
APP_ID="io.github.mazer666.phocommander"
BINARY_NAME="phocommander-desktop"
APPDIR_NAME="Phocommander.AppDir"

# Das Programm, das gepackt wird. Das Argument überschreibt den Weg, damit
# das Skript auch mit einem Programm aus einem anderen Bau funktioniert.
BINARY="${1:-target/release/${BINARY_NAME}}"
OUTPUT_DIR="${2:-target/appimage}"

if [ ! -f "$BINARY" ]; then
    echo "FEHLER: das Programm '${BINARY}' gibt es nicht." >&2
    echo "       Erst bauen: ./scripts/cargo.sh build --release -p ${BINARY_NAME}" >&2
    exit 1
fi

if [ ! -f "assets/phocommander.png" ]; then
    echo "FEHLER: das Symbol 'assets/phocommander.png' gibt es nicht." >&2
    exit 1
fi

for werkzeug in appimagetool desktop-file-validate; do
    if ! command -v "$werkzeug" > /dev/null 2>&1; then
        echo "FEHLER: das Werkzeug '${werkzeug}' ist nicht installiert." >&2
        case "$werkzeug" in
            appimagetool)
                echo "       Es gehört zum AppImageKit: https://github.com/AppImage/AppImageKit/releases" >&2
                ;;
            desktop-file-validate)
                echo "       Es gehört zum Paket 'desktop-file-utils' der Distribution." >&2
                ;;
        esac
        exit 1
    fi
done

# Die Version steht im Manifest des Workspace. Das Skript liest sie dort,
# damit die Version im Dateinamen nicht von der Version im Programm
# abweichen kann.
VERSION="$(sed -n 's/^version = "\(.*\)"$/\1/p' Cargo.toml | head -n 1)"
if [ -z "$VERSION" ]; then
    echo "FEHLER: die Version wurde im Manifest des Workspace nicht gefunden." >&2
    exit 1
fi

APPDIR="${OUTPUT_DIR}/${APPDIR_NAME}"
APPIMAGE="${OUTPUT_DIR}/Phocommander-${VERSION}-$(uname -m).AppImage"

# ---------------------------------------------------------------------------
# Das AppDir. Es wird jedes Mal neu gebaut, damit kein Rest einer früheren
# Fassung in der Datei steckt.
# ---------------------------------------------------------------------------
rm -rf "$APPDIR"
mkdir -p "${APPDIR}/usr/bin" \
         "${APPDIR}/usr/share/applications" \
         "${APPDIR}/usr/share/icons/hicolor/256x256/apps" \
         "${APPDIR}/usr/share/icons/hicolor/scalable/apps" \
         "${APPDIR}/usr/share/metainfo"

install -m 0755 "$BINARY" "${APPDIR}/usr/bin/${BINARY_NAME}"
install -m 0644 "assets/${APP_ID}.desktop" "${APPDIR}/usr/share/applications/${APP_ID}.desktop"
install -m 0644 "assets/phocommander.png" "${APPDIR}/usr/share/icons/hicolor/256x256/apps/${APP_ID}.png"
install -m 0644 "assets/phocommander.svg" "${APPDIR}/usr/share/icons/hicolor/scalable/apps/${APP_ID}.svg"

# Die Beschreibung der Anwendung fuer die Software-Zentren.
#
# WHY: der appimagetool sucht die Datei unter dem Namen, der aus der
#       Schreibtischdatei folgt, und ersetzt dabei die Endung ".desktop"
#       durch ".appdata.xml". Die heute uebliche Endung ".metainfo.xml"
#       legt der Flatpak-Bauer aus. Damit beide Werkzeuge dieselbe Datei
#       finden, liegen zwei Namen derselben Datei im Verzeichnis.
install -m 0644 "assets/${APP_ID}.metainfo.xml" "${APPDIR}/usr/share/metainfo/${APP_ID}.appdata.xml"
install -m 0644 "assets/${APP_ID}.metainfo.xml" "${APPDIR}/usr/share/metainfo/${APP_ID}.metainfo.xml"

# Das Programm im Wurzelverzeichnis, der Eintrag und das Symbol.
#
# WHY: der appimagetool liest die Schreibtischdatei und das Symbol im
#       Wurzelverzeichnis des AppDir. Ohne diese beiden Dateien bricht der
#       Packschritt ab ("Desktop file not found"). Das Symbol muss eine
#       PNG-Datei sein, deren Name dem Eintrag "Icon=" entspricht.
cp "${APPDIR}/usr/share/applications/${APP_ID}.desktop" "${APPDIR}/${APP_ID}.desktop"
cp "${APPDIR}/usr/share/icons/hicolor/256x256/apps/${APP_ID}.png" "${APPDIR}/${APP_ID}.png"
ln -sf "${APP_ID}.png" "${APPDIR}/.DirIcon"

# Der Einstieg des AppImage.
#
# WHY: der Einstieg muss den Pfad des Programms kennen, und der Pfad ist im
#       gemounteten AppImage nicht der Pfad im Buildverzeichnis. Das Skript
#       bestimmt ihn selbst aus seinem eigenen Ort.
cat > "${APPDIR}/AppRun" <<'APPRUN'
#!/bin/sh
# The entry point of the AppImage of Phocommander.
#
# WHY: the program lives beside this script inside the mounted image, and the
#       path is only known at run time.
HERE="$(dirname "$(readlink -f "$0")")"
exec "${HERE}/usr/bin/phocommander-desktop" "$@"
APPRUN
chmod 0755 "${APPDIR}/AppRun"

# ---------------------------------------------------------------------------
# Das Packen. Das Werkzeug braucht die Architektur in der Umgebung, weil es
# sie in den Dateinamen der fertigen Datei schreibt, und die Version, weil es
# sie in die Schreibtischdatei schreibt.
# ---------------------------------------------------------------------------
cd "$OUTPUT_DIR"
ARCH="$(uname -m)" VERSION="$VERSION" appimagetool "$APPDIR_NAME" "$APPIMAGE"

if [ ! -f "$APPIMAGE" ]; then
    echo "FEHLER: der appimagetool hat keine Datei erzeugt." >&2
    exit 1
fi

echo
echo "Das AppImage liegt in: ${APPIMAGE}"
echo "Es traegt die Version ${VERSION} und braucht kein Programm, keine"
echo "Rechte und keine Installation."
