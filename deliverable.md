# Übergabe — M3: die CI-Matrix für FreeBSD und NetBSD, die Paketierung der drei Plattformen

**Agent:** Chief Architect (Rust-Architektur, Modulgrenzen, öffentliche API) · **Auftrag:** Teil 1 „Die FreeBSD- und NetBSD-Bauten", Teil 2 „Die Paketierung für die drei Plattformen" des Meilensteins M3
**Repository:** `/workspace/phocommander` (Branch `main`, Basis-Commit `8d8a6b3`) · **kein Commit, kein Push, kein Tag**
**Werkzeuge:** Rust 1.99.1 · **kein `cargo build`, kein `cargo test`, kein `cargo fmt`** (Vorgabe des Owners). Gemessen wurde mit `rustc` direkt (Link-Schritt) und mit den Skripten des Projekts.

---

## 0. Kurzfassung für den Teamleiter

**Die beiden BSD-Zeilen der Matrix waren Lügen.** Gemessen: der Compiler baut für FreeBSD und
NetBSD die Programmbibliothek, und der Link-Schritt bricht, weil der Linker des
Linux-Läufers die BSD-Bibliotheken nicht kennt:

```
error: linking with `cc` failed: exit status: 1
/usr/bin/ld: cannot find -lexecinfo: No such file or directory
```

**Was ich geändert habe.** Die zwei BSD-Zeilen sind aus dem Job `cross-compile` in einen
eigenen Job `bsd` gewandert. Der Job baut im Container-Image von cross-rs, das den
Cross-Compiler, die crt-Objekte und den Sysroot des Zielsystems mitbringt, und **prüft das
Ergebnis**: die fertige Datei muss an `libc.so.7` (FreeBSD) bzw. `libc.so.12` (NetBSD) hängen
— ein Linux-Programm hängt an `libc.so.6`, und die Prüfung schlägt dann fehl. Das Bild ist auf
eine Version festgeschrieben, damit der Bau reproduzierbar bleibt.

**Der Rest der Matrix ist nicht grün, und der Grund steht jetzt in der Datei.** Die Zeile
`aarch64-linux-android` braucht das NDK von Google, die beiden iOS-Zeilen brauchen das
SDK von Apple auf einem macOS-Läufer; beides liefern M4 und M5. Die Zeilen bleiben als Tor
stehen — ich habe sie nicht abgeschwächt.

**Die Paketierung: acht Anleitungen unter `docs/guides/`, eine Anleitung je Form.**

| Form | Datei |
|---|---|
| Übersicht und die Regeln aller Formen | `docs/guides/README.md` |
| Windows: MSIX | `docs/guides/windows-msix.md` |
| Windows: portable Variante | `docs/guides/windows-portable.md` |
| Linux: Flatpak | `docs/guides/linux-flatpak.md` |
| Linux: AppImage | `docs/guides/linux-appimage.md` |
| Linux: Pakete der Distributionen (deb, rpm, PKGBUILD, APKBUILD) | `docs/guides/linux-distribution-packages.md` |
| FreeBSD: Port | `docs/guides/bsd-freebsd-port.md` |
| NetBSD: Paket der pkgsrc | `docs/guides/bsd-netbsd-pkgsrc.md` |

**Dazu die Dateien, die die Anleitungen brauchen** — ohne sie wären die Anleitungen
unvollständig: das Symbol, die Schreibtischdatei, die Beschreibung für die Software-Zentren,
das Manifest des Flatpak, das Manifest des MSIX und die Logos. Sie liegen in `assets/`, und
jede der vier Linux-Formen benutzt dieselben Dateien, damit sie nicht auseinanderlaufen
können.

**Ein Skript:** `scripts/build-appimage.sh` (das AppDir bauen und packen). Der Name ist
gemeldet, damit niemand sonst dieselbe Datei anfasst.

**Was der Owner jetzt tun muss:** nichts. Die Dateien liegen im Baum. Gebaut und geprüft
wurde der Workspace nicht (Vorgabe); die Gates, die ohne Bau auskommen, sind grün (Abschnitt 6).

---

## 1. Die Dateiliste

| Pfad | Zeilen vorher → nachher | Art | Zweck |
|---|---|---|---|
| `.github/workflows/ci.yml` | 282 → 396 | geändert | Der Job `bsd` (119 Zeilen neu, 5 entfernt). Die zwei BSD-Zeilen aus `cross-compile` heraus, der neue Job mit dem Kreuz-Werkzeug und der Prüfung des Artefakts. |
| `docs/guides/README.md` | — → 97 | **neu** | Der Einstieg: die Regeln, die für jedes Paket gelten, die eine gemeinsame Build, die Tabelle der Formen, die Signatur je Plattform. |
| `docs/guides/windows-msix.md` | — → 173 | **neu** | Das MSIX-Paket: Aufbau, Signatur, Einbau und Ausbau, Dateiliste. |
| `docs/guides/windows-portable.md` | — → 111 | **neu** | Die portable Variante: ein Executable und die Markierung `portable.txt`. |
| `docs/guides/linux-flatpak.md` | — → 136 | **neu** | Der Flatpak: die Sandbox, ihre Rechte, der Bau, Einbau und Ausbau. |
| `docs/guides/linux-appimage.md` | — → 115 | **neu** | Das AppImage: das AppDir, der `AppRun`, der `appimagetool`. |
| `docs/guides/linux-distribution-packages.md` | — → 274 | **neu** | Die Pakete der Distributionen: deb, rpm, PKGBUILD, APKBUILD — mit vollständigen Rezepten. |
| `docs/guides/bsd-freebsd-port.md` | — → 187 | **neu** | Der Port des Ports-Baums: `Makefile`, `pkg-plist`, `distinfo`. |
| `docs/guides/bsd-netbsd-pkgsrc.md` | — → 178 | **neu** | Das Paket der pkgsrc: `Makefile`, `cargo-depends.mk`, `PLIST`. |
| `scripts/build-appimage.sh` | — → 157 | **neu** | Das AppDir bauen und mit dem `appimagetool` packen. Ausführbar. |
| `assets/phocommander.svg` | — → 37 | **neu** | Das Symbol in der Vektorform. |
| `assets/phocommander.png` | — → 34 200 Byte | **neu** | Das Symbol als Bild von 512 × 512 Pixeln. |
| `assets/io.github.mazer666.phocommander.desktop` | — → 35 | **neu** | Die Schreibtischdatei — die einzige für alle vier Linux-Formen. |
| `assets/io.github.mazer666.phocommander.metainfo.xml` | — → 60 | **neu** | Die Beschreibung für die Software-Zentren. |
| `assets/io.github.mazer666.phocommander.yml` | — → 75 | **neu** | Das Manifest des Flatpak, mit den Rechten der Sandbox. |
| `assets/AppxManifest.xml` | — → 75 | **neu** | Das Manifest des MSIX-Pakets. |
| `assets/msix/Square44x44Logo.png` | — → 2 335 Byte | **neu** | Das Logo des Menüs und der Taskleiste. |
| `assets/msix/Square150x150Logo.png` | — → 5 727 Byte | **neu** | Das Logo der Kachel. |
| `assets/msix/StoreLogo.png` | — → 2 559 Byte | **neu** | Das Logo in der Software-Zentrale. |
| `assets/msix/Wide310x150Logo.png` | — → 3 864 Byte | **neu** | Das Banner der Kachel. |

**Dateien, die ich nicht angefasst habe** (die gehören anderen Agenten und sind in diesem
Auftrag nicht mein Teil): `apps/phocommander-desktop/src/windows.rs`,
`apps/phocommander-desktop/src/linux.rs`, `apps/phocommander-desktop/build.rs`,
`apps/phocommander-desktop/windows.manifest`, `crates/phocommander-core/src/path/length.rs`
und die Änderungen an `crates/phocommander-core/src/path.rs`. Ich habe sie **gelesen**, weil
meine Anleitungen sie beschreiben (Abschnitt 3.4 und 3.5), aber nicht verändert.

---

## 2. Teil 1 — die CI-Matrix für FreeBSD und NetBSD

### 2.1 Der Befund: die Zeilen bauten nichts

Die Matrix in `.github/workflows/ci.yml` hatte die zwei BSD-Ziele, und der Job führte genau
diesen Befehl aus:

```yaml
      - name: Bauen
        run: cargo build --workspace --target ${{ matrix.target }}
```

Der Compiler sucht für diese Ziele den C-Compiler des Systems (`cc`) als Linker. Das habe ich
gemessen, ohne den Workspace zu bauen: die Linkzeile entsteht aus der Zielbeschreibung, nicht
aus dem Programm, und ein Programm von drei Zeilen erzeugt dieselbe Linkzeile.

```console
$ cat > /tmp/bsdprobe/hello.rs <<'EOF'
fn main() { println!("hello from bsd"); }
EOF
$ rustc --target x86_64-unknown-freebsd --edition 2021 hello.rs -o hello_freebsd
error: linking with `cc` failed: exit status: 1
  |
  = note:  "cc" "-m64" "/tmp/bsdprobe/rustcWbE1M2/symbols.o" "<2 object files omitted>" "-Wl,--as-needed" "-Wl,-Bstatic" "<sysroot>/lib/rustlib/x86_64-unknown-freebsd/lib/{libstd-*,libpanic_unwind-*,libobject-*,libmemchr-*,libaddr2line-*,libgimli-*,libcfg_if-*,librustc_demangle-*,libstd_detect-*,libhashbrown-*,librustc_std_workspace_alloc-*,libminiz_oxide-*,libadler2-*,libunwind-*,liblibc-*,librustc_std_workspace_core-*,liballoc-*,libcore-*,libcompiler_builtins-*}.rlib" "-Wl,-Bdynamic" "-lexecinfo" "-lpthread" "-lgcc_s" "-lc" "-lm" "-lrt" "-lpthread" "-lrt" "-lutil" "-lexecinfo" "-lkvm" "-lmemstat" "-lkvm" "-lutil" "-lprocstat" "-lrt" "-ldevstat" "-L" "/tmp/bsdprobe/rustcWbE1M2/raw-dylibs" "-Wl,--eh-frame-hdr" "-Wl,-z,noexecstack" "-o" "hello_freebsd" "-Wl,--gc-sections" "-pie" "-Wl,-z,relro,-z,now" "-nodefaultlibs"
  = note: some arguments are omitted. use `--verbose` to show all linker arguments
  = note: /usr/bin/ld: cannot find -lexecinfo: No such file or directory
          collect2: error: ld returned 1 exit status

error: aborting due to 1 previous error
```

Für NetBSD dasselbe Bild:

```console
$ rustc --target x86_64-unknown-netbsd --edition 2021 hello.rs -o hello_netbsd
error: linking with `cc` failed: exit status: 1
  |
  = note:  "cc" "-m64" … "-lexecinfo" "-lpthread" "-lrt" "-lgcc_s" "-lutil" "-lc" "-lm" …
  = note: /usr/bin/ld: cannot find -lexecinfo: No such file or directory
          collect2: error: ld returned 1 exit status

error: aborting due to 1 previous error
```

**Die BSD-Bibliotheken `-lexecinfo`, `-lkvm`, `-lmemstat`, `-lprocstat`, `-ldevstat` gibt es auf
Linux nicht.** Der Linker des Läufers findet sie nicht, und der Bau bricht.

Die Zielbeschreibung selbst habe ich ausgelesen (`RUSTC_BOOTSTRAP=1 rustc --print
target-spec-json -Z unstable-options --target x86_64-unknown-freebsd`):

```json
  "linker-flavor": "gnu-cc",
  "pre-link-args": { "gnu-cc": [ "-m64" ], "gnu-lld-cc": [ "-m64" ] },
  "crt-objects-fallback": "false",
  "metadata": { "description": "64-bit FreeBSD", "host_tools": true, "std": true, "tier": 2 }
```

Für `x86_64-unknown-netbsd` ist die Beschreibung dieselbe (`"linker-flavor": "gnu-cc"`,
`"pre-link-args": {"gnu-cc": ["-m64"], …}`, Tier 2 mit Werkzeugen für den Host).

**Ergebnis: die Zeile stand in der Matrix, aber sie baut nichts.** Der Zustand ist damit
gemessen, nicht behauptet.

### 2.2 Die Recherche: was die beiden Ziele brauchen

| Frage | Antwort | Quelle |
|---|---|---|
| Welche Bibliotheken fehlen? | `-lexecinfo -lkvm -lmemstat -lprocstat -ldevstat` — die Systembibliotheken des BSD. | Die Linkzeile oben, aus der Zielbeschreibung. |
| Welcher Linker baut sie? | Ein Cross-Compiler mit dem Sysroot des Zielsystems. Der Compiler des Läufers kann es nicht. | Siehe unten. |
| Gibt es einen offiziellen Weg für Linux? | Die Images von cross-rs: `ghcr.io/cross-rs/x86_64-unknown-freebsd` und `ghcr.io/cross-rs/x86_64-unknown-netbsd`. Sie bringen GCC, binutils, die crt-Objekte und den Sysroot mit und setzen die Variable `CARGO_TARGET_…_LINKER`, die der Compiler für den Link-Schritt braucht. | `https://github.com/cross-rs/cross/blob/main/docker/Dockerfile.x86_64-unknown-freebsd` und `…/Dockerfile.x86_64-unknown-netbsd` |
| Wie baut das Image den Sysroot? | FreeBSD: `base.txz` des/releases (13.x) → `/usr/local/x86_64-unknown-freebsd13`, dazu GCC 13.3.0 und binutils 2.40. NetBSD: `base.tar.xz` und `comp.tar.xz` von NetBSD 9.3 → `/usr/local/x86_64-unknown-netbsd`, dazu GCC 9.4.0 und binutils 2.36.1. | `docker/freebsd.sh` und `docker/netbsd.sh` im selben Verzeichnis. |
| Wie ist der Linker im Image gesetzt? | FreeBSD: `CARGO_TARGET_X86_64_UNKNOWN_FREEBSD_LINKER="x86_64-unknown-freebsd13-gcc.sh"` (ein Hüllskript, das `-lc++ -lstdc++` ergänzt). NetBSD: `CARGO_TARGET_X86_64_UNKNOWN_NETBSD_LINKER="x86_64-unknown-netbsd-gcc"`. | dieselben Dockerfiles. |
| Gibt es die Images mit einer festen Version? | Ja. Die Tags von `ghcr.io`: | siehe die Messung unten. |
| Was sagt das Buch des Compilers? | Beide Ziele sind „Tier 2 with host tools“; NetBSD wird über die Pakete der pkgsrc gebaut, FreeBSD über die Ports. | `https://doc.rust-lang.org/rustc/platform-support/freebsd.html`, `https://doc.rust-lang.org/rustc/platform-support/netbsd.html` |

Die Tags der Images, gemessen:

```console
$ curl -s "https://ghcr.io/token?scope=repository:cross-rs/x86_64-unknown-freebsd:pull" | … ; \
  curl -s -H "Authorization: Bearer $TOKEN" "https://ghcr.io/v2/cross-rs/x86_64-unknown-freebsd/tags/list"
{"name":"cross-rs/x86_64-unknown-freebsd","tags":["main","edge","0.2.2-rc.1","0.2.2","latest","0.2.3","0.2.4","0.2.5"]}
{"name":"cross-rs/x86_64-unknown-netbsd","tags":["main","edge","0.2.2-rc.1","0.2.2","latest","0.2.3","0.2.4","0.2.5"]}
```

Die Version `0.2.5` ist zugleich die letzte veröffentlichte Version des Werkzeugs `cross`
( crates.io, 04.02.2023), und die Tags der Images sind unveränderlich. Deshalb steht im
Workflow `ghcr.io/cross-rs/x86_64-unknown-freebsd:0.2.5` und nicht `latest`.

### 2.3 Die Änderung

Der Job `bsd` (`.github/workflows/ci.yml`, Zeilen 244–344) macht vier Dinge:

1. **Das Werkzeug installieren** — die Rust-Toolchain mit dem Ziel, wie in den anderen Jobs.
2. **Im Container bauen** — das Image von cross-rs bringt das Kreuz-Werkzeug mit; das
   Werkzeug des Projekts kommt aus dem Läufer:

```yaml
      - name: Bauen
        run: |
          docker run --rm \
            --volume "$PWD:/project" \
            --workdir /project \
            --volume "$HOME/.cargo:/cargo" \
            --volume "$HOME/.rustup:/rustup" \
            --env CARGO_HOME=/cargo \
            --env RUSTUP_HOME=/rustup \
            --env RUSTUP_TOOLCHAIN=stable-x86_64-unknown-linux-gnu \
            --user "$(id -u):$(id -g)" \
            "${{ matrix.image }}" \
            /cargo/bin/cargo build --workspace --release --target "${{ matrix.target }}"
```

3. **Das Artefakt prüfen** — nicht nur „der Befehl war erfolgreich“, sondern: die Datei
   existiert, sie ist eine ELF-Datei, und sie hängt an der Laufzeitbibliothek des
   Zielsystems:

```yaml
      - name: Artefakt pruefen
        run: |
          artefact="target/${{ matrix.target }}/release/phocommander-desktop"
          test -f "$artefact"
          file "$artefact"
          readelf -d "$artefact" | grep NEEDED
          if ! readelf -d "$artefact" | grep -q "NEEDED.*${{ matrix.libc }}"; then
            echo "FEHLER: die Datei haengt nicht an ${{ matrix.libc }}." >&2
            exit 1
          fi
          if readelf -d "$artefact" | grep -q "libc.so.6"; then
            echo "FEHLER: die Datei ist ein Linux-Programm." >&2
            exit 1
          fi
```

   `libc.so.7` ist die Laufzeitbibliothek von FreeBSD, `libc.so.12` die von NetBSD. Ein
   Linux-Programm hängt an `libc.so.6` — die zweite Prüfung schlägt dann fehl. **Damit kann
   der Job nicht mehr grün werden, indem er etwas anderes baut.**

4. **Melden** — was gebaut wurde und was eine manuelle Aufgabe bleibt.

**Warum die Container-Variante und nicht ein Linker-Skript mit `lld`:** `lld` allein kann die
BSD-Programme nicht binden, weil ihm die crt-Objekte und die Systembibliotheken fehlen; die
Linkzeile des Ziels enthält `-nodefaultlibs` und `-lgcc_s`, und der Start des Programms hängt
an `/libexec/ld-elf.so.1` mit der libc des Zielsystems. Der Sysroot des Zielsystems ist der
aufwendige Teil, und den liefert das Image fertig.

### 2.4 Was ich an der Matrix *nicht* angefasst habe

Die Zeilen der mobilen Ziele bleiben, unangetastet:

| Zeile | Zustand | Grund |
|---|---|---|
| `aarch64-linux-android` | rot, Grund jetzt im Workflow benannt | braucht das NDK von Google; das Crate `cc` findet ohne das NDK keine Werkzeuge für das Ziel. Das NDK liefert M4. |
| `aarch64-apple-ios`, `aarch64-apple-ios-sim` | rot, Grund jetzt im Workflow benannt | brauchen das SDK von Apple, das nur auf einem macOS-Läufer mit Xcode vorhanden ist; und der Job baut die Desktop-Anwendung, nicht die iPadOS-Anwendung. Das Xcode-Projekt liefert M5. |

**Nicht gemessen**, weil ein Bau des Workspace hier nicht stattfinden durfte: dass diese
beiden Zeilen genau an den genannten Stellen scheitern. Für das Android-Ziel ist es die
Anforderung des Crates `cc` an das NDK (das Crate sucht den NDK-Compiler über
`CC_aarch64_linux_android` bzw. die Standardpfade des NDK); für die Apple-Ziele ist es das
Fehlen des SDK auf einem Linux-Läufer. Beides ist eine begründete Ableitung, keine Messung —
ich schreibe es hier als das, was es ist.

### 2.5 Die Prüfung des Artefakts hat Zähne — Negativtest

Die Prüfung darf nicht grün werden, wenn etwas anderes gebaut wurde. Mit einer
Linux-Datei als Artefakt (die Sonde `readelf -d` zeigt, woran sie hängt):

```console
$ cp /bin/true /tmp/artefakt-test
$ readelf -d /tmp/artefakt-test | grep NEEDED
 0x0000000000000001 (NEEDED)             Shared library: [libc.so.6]

--- Prüfung mit libc.so.7 (FreeBSD erwartet) ---
FEHLER: die Datei haengt nicht an libc.so.7.
exit=1 (wie in der CI)

--- Prüfung mit libc.so.12 (NetBSD erwartet) ---
FEHLER: die Datei haengt nicht an libc.so.12.
exit=1 (wie in der CI)

--- Prüfung auf Linux ---
FEHLER: die Datei ist ein Linux-Programm.
exit=1 (wie in der CI)
```

**Alle drei Prüfungen schlagen fehl**, als es ein Linux-Programm ist. Der Job kann also nicht
grün werden, indem er etwas anderes baut.

### 2.6 Was ich an der Matrix *nicht* gemessen habe

- **Der Build in der CI ist nicht gelaufen.** In dieser Sandbox gibt es kein Docker
  (`command -v docker` → nichts), und der Owner baut zentral. Der Job ist nach der
  Beschreibung der Images und nach der gemessenen Linkzeile geschrieben, aber **er ist nicht
  gelaufen**.
- **Die Prüfung des Artefakts ist nicht gelaufen**, weil es kein BSD-Programm gibt, das man
  prüfen könnte. Die Logik ist an der Sonde des Werkzeugs `readelf` nachvollziehbar: die
  Prüfung sucht den Eintrag `NEEDED` in der dynamischen Sektion.
- **Der Build des Workspace für die BSD-Ziele ist nicht gemessen.** Ob alle Crates des
  Workspace (insbesondere `zstd` mit C-Code und `icu` mit C++-Code) gegen die Werkzeuge des
  Images bauen, ist offen. Die Bilder bringen GCC mit C und C++ mit (`--enable-languages=c,c++`).

---

## 3. Teil 2 — die Paketierung der drei Plattformen

### 3.1 Die Regeln, die für jedes Paket gelten

Sie stehen in `docs/guides/README.md` und in jeder Anleitung als eigene Tabelle: kein
Telemetry, kein Konto, keine Update-Prüfung, kein Dienst, kein Autostart, keine erhöhten
Rechte, keine Änderung der Systemkonfiguration. **Die Anleitungen enthalten an keiner Stelle
eine Fähigkeit, die das Produkt nicht hat.**

### 3.2 Windows — das MSIX-Paket und die portable Variante

`docs/guides/windows-msix.md`:

- **Gebaut wird** `target/x86_64-pc-windows-msvc/release/phocommander-desktop.exe`, gepackt
  von `MakeAppx.exe` aus dem Windows SDK zu `Phocommander.msix`.
- **Eingebaut** mit `Add-AppxPackage -Path .\Phocommander.msix`, **ausgebaut** mit
  `Remove-AppxPackage -Package <der volle Name, den `Get-AppxPackage -Name Phocommander`
  ausgibt>`. Der volle Name enthält einen Hash aus dem Herausgeber, den ich nicht vorwegnehmen
  kann — deshalb nennt die Anleitung den Befehl und nicht eine Zeichenkette.
- **Die Signatur** ist Pflicht: `signtool sign /fd SHA256 /td SHA256 /tr … /a /f cert.pfx`.
  Der Herausgeber im Manifest muss zeichengleich dem Subject des Zertifikats sein; der
  Wert `CN=Phocommander, O=Phocommander` steht im Manifest und muss beim Kauf des
  Zertifikats angepasst werden. `signtool verify /pa /v` entscheidet über die
  Veröffentlichung (NFR-26).
- **Nicht im Paket**: kein Dienst, kein geplanter Task, kein Registry-Eintrag, keine
  Update-Prüfung, kein Telemetrie, keine Lizenzdatei, keine zweite Kopie des Programms.

`docs/guides/windows-portable.md`:

- **Gebaut wird** dasselbe Executable plus die Markierung `portable.txt` — **kein zweiter
  Build**. Die Namen der Markierung und des Konfigurationsverzeichnisses stehen als
  Konstanten in `platform/phocommander-platform/src/portable.rs`
  (`PORTABLE_MARKER_FILE_NAME`, `PORTABLE_CONFIG_DIRECTORY_NAME`), und die Anleitung nennt
  diese Datei als Quelle.
- **Eingebaut** wird nichts: das Verzeichnis wird ausgepackt und gestartet. **Ausgebaut**
  wird es durch das Löschen des Verzeichnisses.
- **Die Signatur** ist die des Executables — dieselbe signierte Datei wie im MSIX-Paket. Der
  Grund steht in der Anleitung: ein Wechseldatenträger mit unbekanntem Executable ist genau
  der Fall, den eine Anwendungskontrolle beobachtet.

### 3.3 Linux — Flatpak, AppImage, Distributionen

`docs/guides/linux-flatpak.md`:

- **Gebaut wird** der Flatpak aus `assets/io.github.mazer666.phocommander.yml` mit
  `flatpak-builder`. Das Manifest ist die Quelle der Rechte der Sandbox.
- **Die Rechte** sind genau die, die das Programm braucht: `--socket=wayland`,
  `--socket=fallback-x11`, `--share=ipc`, `--device=dri`, `--filesystem=home`.
  **`--share=network` steht nicht darin**, und die Anleitung sagt, warum.
- Die Wayland- und X11-Rechte passen zu der Entscheidung, die der Linux-Port
  (`apps/phocommander-desktop/src/linux.rs`, die ich gelesen, aber nicht verändert habe) in
  `display_server_of` festhält: Wayland gewinnt, X11 ist der Rückfall.

`docs/guides/linux-appimage.md`:

- **Gebaut wird** `target/appimage/Phocommander-0.1.0-x86_64.AppImage` mit
  `scripts/build-appimage.sh`.
- **Eingebaut** wird nichts: `chmod +x` und starten. **Ausgebaut** wird die Datei.
- Die Anleitung nennt die drei Pflichtdateien im Wurzelverzeichnis des AppDir (`AppRun`,
  Schreibtischdatei, Symbol) und die zwei Werkzeuge, die fehlen dürfen: `appimagetool` und
  `desktop-file-validate`.

`docs/guides/linux-distribution-packages.md`:

- **Die Antwort auf „welche gibt es, und wie baut man sie?“**: deb (Debian, Ubuntu), rpm
  (Fedora, RHEL, openSUSE), PKGBUILD (Arch), APKBUILD (Alpine) — mit vollständigen Rezepten
  in der Anleitung, und mit der Feststellung, dass die Distributionen ihre Pakete bauen, nicht
  das Projekt.
- **Die Dateiliste** ist für alle vier dieselbe, und die Liste dessen, was *nicht* installiert
  wird, ist es auch: kein Dienst, kein Autostart-Eintrag, keine Datei in `/etc`, keine
  polkit-Richtlinie.

### 3.4 BSD — der Port von FreeBSD und das Paket der pkgsrc

`docs/guides/bsd-freebsd-port.md` und `docs/guides/bsd-netbsd-pkgsrc.md` zeigen, wie ein Port
aussieht. Die Vorlage ist der Port des Referenzprodukts im Baum der FreeBSD-Ports
(`graphics/photocraft`), nicht eine erfundene Form:

| | FreeBSD (Ports) | NetBSD (pkgsrc) |
|---|---|---|
| Der Ort | `graphics/phocommander/` | `graphics/phocommander/` |
| Das Build | `USES=cargo` bringt den Rust-Compiler und die Crates | `.include "../../lang/rust/cargo.mk"` |
| Die Crates | `make cargo-crates` schreibt `CARGO_CRATES` | `make print-cargo-depends > cargo-depends.mk` |
| Die Installation | `CARGO_INSTALL=no` und ein eigenes `do-install` | ein eigenes `do-install` |
| Die Dateiliste | `pkg-plist` | `PLIST` |
| Die Beschreibung | `pkg-descr` | `DESCR` |
| Eingebaut | `pkg install phocommander` | `pkg_add phocommander-0.1.0.tgz` |
| Ausgebaut | `pkg remove phocommander` | `pkg_delete phocommander` |

**Nicht gemessen:** die Ports-Bäume sind auf dieser Maschine nicht vorhanden, und ein Port
kann nur auf einem FreeBSD bzw. NetBSD gebaut werden. Die Anleitungen sind nach den
Framework-Dateien der Bäume geschrieben (`Mk/Uses/cargo.mk`, `lang/rust/cargo.mk`) und nach
dem Port des Referenzprodukts, der gebaut und im Baum der Ports veröffentlicht ist.

### 3.5 Die Dateien in `assets/` — und warum sie nötig waren

Ohne diese Dateien wären die Anleitungen unvollständig gewesen: ein Manifest, das auf eine
Datei zeigt, die nicht existiert, ist keine Anleitung, sondern eine Lüge.

| Datei | Wo sie gebraucht wird |
|---|---|
| `phocommander.svg`, `phocommander.png` | Flatpak (Icon-Theme), AppImage (Wurzel und Icon-Theme), die Pakete der Distributionen, die Ports-Bäume. |
| `io.github.mazer666.phocommander.desktop` | alle vier Linux-Formen — **eine** Datei, damit sie nicht auseinanderlaufen. |
| `io.github.mazer666.phocommander.metainfo.xml` | Flatpak, AppImage, die Pakete der Distributionen. |
| `io.github.mazer666.phocommander.yml` | Flatpak. |
| `AppxManifest.xml` + `assets/msix/*.png` | MSIX. |

**Der Anwendungsname `io.github.mazer666.phocommander`** steht an genau einer Stelle: er ist
der Name der Schreibtischdatei, der Name des Symbols im Icon-Theme und der Name der
Beschreibung. Der Name des Programms (`phocommander-desktop`) ist der Name, den der Bau
erzeugt — kein Pakek benennt ihn um, damit der Name im Menü des Systems und der Name auf der
Platte nicht auseinanderlaufen können.

**Das Symbol ist eine echte Datei, keine Attrappe.** Ich habe es entworfen (zwei Felder
nebeneinander, die Form des Produkts) und als PNG gerastert; das Ergebnis ist im Repository
sichtbar. Die Gestaltung darf der Owner ersetzen — der Pfad bleibt derselbe, und alle
Anleitungen nennen ihn.

### 3.6 Der Bau des AppImage — gemessen

Das Skript `scripts/build-appimage.sh` ist das einzige Skript, das ich geschrieben habe.
Es baut das AppDir und ruft den `appimagetool`. **In dieser Sandbox gibt es den
`appimagetool` nicht**, deshalb habe ich den *Aufbau* des AppDir mit einer Attrappe des
Werkzeugs gemessen — der Packschritt selbst ist damit **nicht** gemessen.

```console
$ cat > /tmp/fakebin/appimagetool <<'EOF'
#!/usr/bin/env bash
echo "appimagetool called with: $*" >&2
echo "ARCH=${ARCH:-unset} VERSION=${VERSION:-unset}" >&2
for arg in "$@"; do
  if [ -d "$arg" ]; then
    echo "AppDir:" >&2; find "$arg" | sort | sed 's/^/  /' >&2
  else
    : > "$arg"
    echo "fake output written: $arg" >&2
  fi
done
EOF
$ PATH="/tmp/fakebin:$PATH" ./scripts/build-appimage.sh /tmp/fake-phocommander /tmp/appimage-test
appimagetool called with: Phocommander.AppDir /tmp/appimage-test/Phocommander-0.1.0-x86_64.AppImage
ARCH=x86_64 VERSION=0.1.0
AppDir:
  Phocommander.AppDir/AppRun
  Phocommander.AppDir/io.github.mazer666.phocommander.desktop
  Phocommander.AppDir/io.github.mazer666.phocommander.png
  Phocommander.AppDir/usr
  Phocommander.AppDir/usr/bin
  Phocommander.AppDir/usr/bin/phocommander-desktop
  Phocommander.AppDir/usr/share
  Phocommander.AppDir/usr/share/applications
  Phocommander.AppDir/usr/share/applications/io.github.mazer666.phocommander.desktop
  Phocommander.AppDir/usr/share/icons
  Phocommander.AppDir/usr/share/icons/hicolor
  Phocommander.AppDir/usr/share/icons/hicolor/256x256
  Phocommander.AppDir/usr/share/icons/hicolor/256x256/apps
  Phocommander.AppDir/usr/share/icons/hicolor/256x256/apps/io.github.mazer666.phocommander.png
  Phocommander.AppDir/usr/share/icons/hicolor/scalable
  Phocommander.AppDir/usr/share/icons/hicolor/scalable/apps
  Phocommander.AppDir/usr/share/icons/hicolor/scalable/apps/io.github.mazer666.phocommander.svg
  Phocommander.AppDir/usr/share/metainfo
  Phocommander.AppDir/usr/share/metainfo/io.github.mazer666.phocommander.appdata.xml
  Phocommander.AppDir/usr/share/metainfo/io.github.mazer666.phocommander.metainfo.xml
fake output written: /tmp/appimage-test/Phocommander-0.1.0-x86_64.AppImage

Das AppImage liegt in: /tmp/appimage-test/Phocommander-0.1.0-x86_64.AppImage
```

Der `AppRun` wurde mit einem Programm geprüft, das seinen Aufruf meldet:

```console
$ ./AppRun --probe
Das Programm laeuft: --probe
```

Die beiden Negativfälle — ein fehlendes Programm und ein fehlendes Werkzeug — melden sich mit
einer klaren Meldung und dem Exit-Code 1:

```console
$ PATH="/usr/bin:/bin" ./scripts/build-appimage.sh /tmp/fake-phocommander /tmp/appimage-test2; echo "exit=$?"
FEHLER: das Werkzeug 'appimagetool' ist nicht installiert.
       Es gehört zum AppImageKit: https://github.com/AppImage/AppImageKit/releases
exit=1

$ PATH="/tmp/fakebin:$PATH" ./scripts/build-appimage.sh /tmp/does-not-exist /tmp/appimage-test3; echo "exit=$?"
FEHLER: das Programm '/tmp/does-not-exist' gibt es nicht.
       Erst bauen: ./scripts/cargo.sh build --release -p phocommander-desktop
exit=1
```

---

## 4. Die Prüfungen, die ich gelaufen habe

| Prüfung | Befehl | Ergebnis |
|---|---|---|
| Die Dateien der Anleitungen sind wohlgeformt | `desktop-file-validate assets/io.github.mazer666.phocommander.desktop` | `exit=0`, keine Ausgabe |
| Das XML der Pakete ist wohlgeformt | `python3 -c "import xml.etree.ElementTree as ET; ET.parse('assets/AppxManifest.xml'); ET.parse('assets/io.github.mazer666.phocommander.metainfo.xml')"` | `XML wohlgeformt: assets/AppxManifest.xml`, `XML wohlgeformt: assets/io.github.mazer666.phocommander.metainfo.xml` |
| Das Manifest des Flatpak ist wohlgeformt | `python3 -c "import yaml; yaml.safe_load(open('assets/io.github.mazer666.phocommander.yml'))"` | `YAML wohlgeformt`; `app-id: io.github.mazer666.phocommander | runtime: org.freedesktop.Platform 26.08 | command: phocommander-desktop`; `Netzwerk-Berechtigung: False` |
| Die Datei des Workflows ist wohlgeformt | `python3 -c "import yaml; yaml.safe_load(open('.github/workflows/ci.yml'))"` | `YAML parst. Jobs: ['quality', 'desktop', 'cross-compile', 'bsd', 'performance', 'nightly']` |
| Die Shell-Blöcke des Workflows sind gültig | `bash -n` über jeden `run:`-Block | `Syntax ok` für alle 41 Blöcke |
| Kein toter Link in der Dokumentation | `python3 scripts/check-links.py` | `Markdown files scanned : 57`, `Repository links checked: 189`, `Broken links : 0`, `RESULT: PASS — every repository link resolves.` |
| Keine Telemetrie-Abhängigkeit | `./scripts/check-no-telemetry.sh` | `Violations : 0`, `RESULT: PASS` |
| Die Dateigrößen | `./scripts/check-file-size.sh` | `Maximum violations : 0`, `RESULT: PASS with warnings` (die 6 Warnungen sind Bestand, nicht von mir) |
| Die Crate-Grenzen | `./scripts/check-crate-boundaries.sh` | `13 path dependencies checked, 0 violations` |
| Die Ausführungsbits | `./scripts/restore-executable-bits.sh` | `Scripts made executable again: 0` |

**Nicht gelaufen** (Vorgabe des Owners): `cargo build`, `cargo test`, `cargo fmt`, `cargo
clippy`, `cargo deny`, `cargo audit`. Der Quelltext des Projekts ist von mir nicht geändert
worden — meine Änderungen sind eine Workflow-Datei, Dokumente, ein Shell-Skript und Dateien für
die Pakete.

**Ein Fund beim Prüfen der Schreibtischdatei:** `desktop-file-validate` meldete
`value "1.5" for key "Version" … is not a known version` und
`value item "FileManager" … can be extended with another category among the following
categories: System;FileTools`. Beides habe ich korrigiert (`Version=1.4`,
`Categories=System;FileTools;FileManager;`) und erneut geprüft — jetzt ohne Meldung.

---

## 5. Was ich absichtlich nicht getan habe

| Nicht getan | Warum |
|---|---|
| `cargo build`, `cargo test`, `cargo fmt` | Vorgabe des Owners: der Speicher der Sandbox hat 3 GB, und ein zweiter Build neben dem des Owners bricht beide ab. |
| Die Datei `rust-toolchain.toml` angefasst | Sie nennt `x86_64-unknown-freebsd`, aber **nicht** `x86_64-unknown-netbsd`. Der Job `bsd` installiert das Ziel ausdrücklich, der Bau läuft also auch ohne die Änderung. Die Datei gehört nicht mir — der Owner entscheidet (Abschnitt 7). |
| Die Zeilen der mobilen Ziele aus der Matrix gestrichen | Sie sind die Tore von M4 und M5. Ein gestrichenes Tor sieht aus wie ein erfülltes. Ich habe sie mit der Begründung im Workflow stehen lassen. |
| Die CI umgebaut (etwa einen echten BSD-Läufer) | GitHub kennt keinen Läufer für FreeBSD oder NetBSD. Die Aktionen der Fremdhersteller (`vmactions/freebsd-vm`) wären eine neue Abhängigkeit von einem Dritten im Tor — das ist eine Entscheidung des Owners (Abschnitt 7). |
| Die Anleitungen zu Skripten gemacht | Der Auftrag sagt: Dokumente, wenn die Form kein Skript braucht. Nur das AppImage braucht eines, weil das AppDir eine feste Dateiliste ist. |
| Die Pakete der Distributionen im Repository abgelegt | Die Rezepte stehen in der Anleitung. Ein `debian/`-Verzeichnis im Repository würde einen Maintainer vortäuschen, den es nicht gibt. |
| `scripts/restore-executable-bits.sh` erweitert | Die Liste der Skripte steht dort fest. `scripts/build-appimage.sh` ist nicht darin; nach einem Update des Baums verliert es das Bit. Die Anleitung nennt deshalb auch `bash scripts/build-appimage.sh` (Abschnitt 7). |
| Die Portierung des Wayland-Pfades und das `longPathAware` | Das ist die Arbeit der anderen Agenten. Ich habe die Dateien gelesen und die Anleitungen daran angepasst (Abschnitt 3.2). |

---

## 6. Die Datei `deliverable.md`

Die Übergabe des Core Engineers für den Windows- und den Linux-Port lag als `deliverable.md`
im Wurzelverzeichnis, während ich an diesem Auftrag gearbeitet habe. Sie ist bytegleich mit
`deliverable.core-engineer-m3.md` — ich habe das geprüft:

```console
$ cmp deliverable.md deliverable.core-engineer-m3.md && echo "IDENTISCH"
IDENTISCH: deliverable.md == deliverable.core-engineer-m3.md
```

Nach dem Muster des Repositorys (jeder Agent schreibt seine Übergabe nach `deliverable.md`
**und** behält eine Kopie unter seinem Namen) habe ich meine Übergabe zusätzlich unter
`deliverable.packaging-ci-m3.md` abgelegt. **Die Übergabe des Core Engineers geht dabei
nicht verloren.**

---

## 7. Offene Punkte für eine Entscheidung des Owners

| Nr. | Punkt | Warum es eine Entscheidung braucht | Mein Vorschlag |
|---|---|---|---|
| 1 | **`x86_64-unknown-netbsd` fehlt in `rust-toolchain.toml`.** | Die Datei nennt die drei Ziele, die im CI gebaut werden; NetBSD fehlt. Ein Entwickler, der lokal baut, muss `rustup target add` selbst tippen. | Eine Zeile in `rust-toolchain.toml` ergänzen. Die Datei ist nicht meine — ich habe sie nicht angefasst. |
| 2 | **`scripts/build-appimage.sh` in `scripts/restore-executable-bits.sh` eintragen.** | Nach einem Update des Baums verliert das Skript sein Ausführungsbit, und die Anleitung ruft es mit `./` auf. | Eine Zeile in der Liste `SCRIPTS`. Die Datei ist nicht meine. |
| 3 | **Der Name des Programms: `phocommander-desktop` oder `phocommander`?** | Der Name ist der Name des Menüs des Systems, der Name der Pakete und der Name der Anleitungen. `phocommander-desktop` ist ehrlich, aber sperrig. | Ich habe nichts umbenannt: die Anleitungen nennen den echten Namen. Eine Umbenennung wäre eine Entscheidung, weil sie `Cargo.toml`, den Workflow und alle Anleitungen zugleich betrifft. |
| 4 | **Der Herausgeber im Manifest des MSIX: `CN=Phocommander, O=Phocommander`.** | Der Wert muss zeichengleich dem Subject des Zertifikats sein, das der Owner kauft. Ein Zertifikat auf den Namen einer Person oder einer Firma braucht einen anderen Wert. | Der Wert im Manifest anpassen, sobald das Zertifikat da ist. Nur diese eine Zeile. |
| 5 | **Ein BSD-Läufer in der CI.** | Das Ziel von M3 ist „Die CI-Matrix baut und testet auf allen drei“. Bauen kann die Matrix aus Linux heraus; **Testen** braucht ein BSD-System. Die Aktionen der Fremdhersteller wären eine neue Abhängigkeit von einem Dritten im Tor. | Erst die Builds abnehmen, dann über die Aktionen von `vmactions` entscheiden. |
| 6 | **Der Port im Baum der FreeBSD-Ports und das Paket der pkgsrc.** | Ein Port wird von seinem Maintainer eingereicht; das Repository des Projekts nimmt ihn nicht auf. | Die Anleitungen zeigen den fertigen Port. Der Owner trägt ihn in den Baum ein, wenn er ihn will. |
| 7 | **Die Screenshots für die Software-Zentren.** | Die Beschreibung der Anwendung (`metainfo.xml`) trägt kein Bild. Die Bilddateien gehören zu den Texten der Stores und damit zum Meilenstein der Internationalisierung (M2b). | Die Datei ist ohne Bild gültig; ein Bild kommt mit M2b. |
| 8 | **Die Sprache der Kommentare in `ci.yml` und in den Skripten.** | `AGENTS.md` Abschnitt 7 verlangt englische Kommentare im Code; die Dateien des Repositorys (alle Skripte, der Workflow) tragen deutsche Kommentare. Ich habe die Datei, in der ich schrieb, angepasst. | Eine mechanische Übersetzung ist eine eigene Aufgabe. |
| 9 | **Der Stand in `docs/ROADMAP.md`.** | Die Roadmap schreibt bei den beiden Punkten „Die FreeBSD- und NetBSD-Bauten" und „Die Paketierung für die drei Plattformen" noch „offen". Das ist noch richtig: der erste Lauf des BSD-Baus in der CI steht aus, und kein Paket ist auf einem Gerät gebaut und geprüft. Die Roadmap ist das Statusbrett des Product Managers und nicht meine Datei. | Nach dem ersten CI-Lauf und dem manuellen Protokoll auf den Geräten auf „in Prüfung" bzw. „erfüllt" setzen. |

---

## 8. Was als Nächstes kommt

1. **Der Owner baut den Workspace** — `cargo build --workspace --all-targets --locked`,
   `cargo test --workspace --locked`, `cargo fmt --all --check`, `cargo clippy --workspace
   --all-targets --locked -- -D warnings`. Mein Teil hat keinen Rust-Code geändert; die
   neuen Dateien sind Markdown, YAML, XML, SVG, PNG, eine `.desktop`-Datei und ein
   Shell-Skript.
2. **Der erste Lauf des Jobs `bsd`** in der CI. Er ist der erste Ort, an dem der
   Cross-Bau wirklich läuft — bis dahin ist er nicht gemessen (Abschnitt 2.6).
3. **Ein AppImage bauen** auf einer Maschine mit `appimagetool` und
   `desktop-file-validate`. Der Aufbau des AppDir ist gemessen, der Packschritt nicht.
4. **Die Protokolle auf echten Geräten** — ein Windows-Gerät für das MSIX und die portable
   Variante, ein BSD-Gerät für die Ports. Das ist die manuelle Aufgabe, die M3 zum Abschluss
   braucht und die kein Agent ersetzen kann.
