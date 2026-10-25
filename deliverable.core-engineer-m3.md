# Übergabe — M3: der Windows-Port und der Linux-Port

**Agent:** Core Engineer · **Auftrag:** Windows- und Linux-Port des Meilensteins M3
**Repository:** `/workspace/phocommander` (Branch `main`, Basis-Commit `1ddde82`)
**Stand der Übergabe:** die Ports sind fertig, geprüft und nicht committet.

---

## 1. Was in einem Satz passiert ist

Der Windows-Port besteht aus einem application manifest mit `longPathAware`, das der
Bau in das Executable einbettet, und aus einer Meldung im Core, die einen zu langen Pfad
in verständlicher Sprache erklärt. Der Linux-Port besteht aus einer Feststellung: **winit
0.30.13 entscheidet Wayland gegen X11 bereits selbst und richtig** — die Arbeit war, diese
Entscheidung sichtbar und testbar zu machen, nicht sie zu erfinden.

---

## 2. Die Dateien, die ich angefasst habe

| Datei | Zeilen vorher → nachher | Art | Zweck |
|---|---|---|---|
| `crates/phocommander-core/src/path.rs` | 381 → 392 | geändert | `pub mod length;` plus die Wiederausgabe der neuen Typen; der Modultext und die Ableitung `PathError` bleiben unberührt. |
| `crates/phocommander-core/src/path/length.rs` | — → 427 | **neu** | Die Regel der Pfadlänge: `LengthUnit`, `PathPlatform`, `PathLengthLimit`, `PathAdvice`, `PathTooLong`, `measure_path_length`, `check_path_length`. |
| `crates/phocommander-core/src/path/length/tests.rs` | — → 149 | **neu** | Die sechs Tests der neuen Regel, darunter der Pfad mit mehr als 260 Zeichen. |
| `crates/phocommander-core/src/path/tests.rs` | 300 → 300 | **unverändert** | Die Datei ist bytegleich mit `HEAD` (`git diff` ist leer) — die neuen Tests liegen in `path/length/tests.rs`. |
| `apps/phocommander-desktop/src/windows.rs` | — → 253 | **neu** | Der Windows-Port: das Manifest, die Grenze der Pfadlänge, das Einbetten des Manifests. Acht Tests. |
| `apps/phocommander-desktop/src/linux.rs` | — → 176 | **neu** | Der Linux-Port: die Entscheidung Wayland/X11. Sieben Tests. |
| `apps/phocommander-desktop/build.rs` | — → 212 | **neu** | Das Build-Skript, das das Manifest in das Executable einbettet. |
| `apps/phocommander-desktop/build/resource.rs` | — → 166 | **neu** | Der Generator der Resource-Datei, die das Manifest trägt. |
| `apps/phocommander-desktop/windows.manifest` | — → 34 | **neu** | Das application manifest mit `longPathAware`. |
| `apps/phocommander-desktop/src/lib.rs` | 33 → 35 | geändert | **Zwei Zeilen** (`pub mod linux;`, `pub mod windows;`) — das ist der Eingriff in die Hotspot-Datei, um den beide ich dich gebeten hatte (siehe Punkt 6). |

**Nicht angefasst:** `apps/phocommander-desktop/Cargo.toml` (das Build-Skript wird von Cargo
automatisch gefunden; es kam keine Abhängigkeit dazu), `src/main.rs`, `.github/workflows/ci.yml`,
`docs/`, `docs/guides/`.

---

## 3. Die öffentliche API

| Crate | Modul | Typ / Funktion | Zweck |
|---|---|---|---|
| `phocommander-core` | `path::length` | `WINDOWS_MAX_PATH_CHARACTERS` (260) | Die Grenze von Windows ohne lange Pfade. |
| `phocommander-core` | `path::length` | `WINDOWS_LONG_PATH_CHARACTERS` (32 767) | Die Grenze von Windows mit langen Pfaden. |
| `phocommander-core` | `path::length` | `LINUX_MAX_PATH_BYTES` (4095) | Die nutzbaren Bytes eines Pfades unter Linux (`PATH_MAX` 4096 abzüglich der Endnull). |
| `phocommander-core` | `path::length` | `LengthUnit` | `Characters` oder `Bytes` — die Einheit, in der gezählt wird. |
| `phocommander-core` | `path::length` | `PathPlatform` | `Windows`, `Linux`, `Other`. |
| `phocommander-core` | `path::length` | `PathLengthLimit` | Die Grenze als Daten: `windows()`, `windows_with_long_paths()`, `linux()`, `new()`, `platform()`, `amount()`, `unit()`, `is_windows_without_long_paths()`. |
| `phocommander-core` | `path::length` | `PathAdvice` | `ShorterName`, `ShorterDirectory`, `MoveToShorterDirectory`, `DriveLetter`, `LongPathsOfTheSystem`, mit `text()`. |
| `phocommander-core` | `path::length` | `PathTooLong` | Die Ablehnung: `path()`, `amount()`, `limit()`, `advice()`, `message()`; `Display` und `Error`. |
| `phocommander-core` | `path::length` | `measure_path_length(path, unit)` | Die Länge eines Pfades in der Einheit der Grenze. |
| `phocommander-core` | `path::length` | `check_path_length(path, limit)` | `Ok(())` oder `Err(PathTooLong)`. |
| `phocommander-core` | `path` | Wiederausgabe aller Namen von oben | `phocommander_core::path::check_path_length` usw. bleiben erreichbar. |
| `phocommander-desktop` | `windows` | `MANIFEST_FILE_NAME`, `LONG_PATH_AWARE_ELEMENT` | Die Namen, die das Manifest trägt. |
| `phocommander-desktop` | `windows` | `manifest_path()`, `manifest_text()` | Das Manifest der Ablage (nur für die Tests und die Werkzeuge). |
| `phocommander-desktop` | `windows` | `declares_long_paths(manifest)` | Prüft, ob ein Manifest `longPathAware` mit dem Wert `true` erklärt. |
| `phocommander-desktop` | `windows` | `path_length_limit(long_paths_of_the_system)` | 260 oder 32 767 Zeichen. |
| `phocommander-desktop` | `windows::resource` | `manifest_resource(manifest, machine)`, `machine_of(arch)` | Die Resource-Datei, die das Manifest trägt. |
| `phocommander-desktop` | `linux` | `DisplayServer` | `Wayland`, `X11`, `None`, mit `as_str()`. |
| `phocommander-desktop` | `linux` | `display_server_of(wayland_display, wayland_socket, display)` | Die Regel von winit, als reine Funktion. |
| `phocommander-desktop` | `linux` | `display_server()` | Die Regel auf der laufenden Maschine. |

---

## 4. Die Meldung bei Pfadlänge (FR-28)

Die Meldung entsteht im Core, weil die Grenze eine Regel des Pfades ist und nicht des
Fensters. Sie nennt den Pfad, die gemessene Länge, die Grenze und die möglichen Handlungen.
Echte Ausgabe (aus dem Prüflauf, siehe Punkt 7):

```
the path C:\long-directory-name\long-directory-name\... is 263 characters long, and Windows
accepts at most 260 characters for a path. Give the file a shorter name, give one of the
directories on the way to it a shorter name, move the file into a directory with a shorter
name, map the directory to a drive letter, so that the path becomes shorter or let the
system use the long paths: the setting LongPathsEnabled has to be 1 and the program has to
declare that it can handle them.
```

Die Meldung liegt als `PathTooLong::message()` vor und zusätzlich als `Display`. Die
Handlungen liegen als `PathAdvice` (Daten, nicht nur Text) vor, damit die Oberfläche sie in
der Sprache des Benutzers und als Liste zeigen kann.

---

## 5. Der Windows-Port

### 5.1 Das Manifest

`apps/phocommander-desktop/windows.manifest` erklärt `longPathAware` in der Form, die
Microsoft für das application manifest dokumentiert
(<https://learn.microsoft.com/en-us/windows/win32/sbscs/application-manifests#longpathaware>):

```xml
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0" xmlns:asmv3="urn:schemas-microsoft-com:asm.v3">
  <assemblyIdentity type="win32" name="Phocommander" version="1.0.0.0" processorArchitecture="*"/>
  <trustInfo xmlns="urn:schemas-microsoft-com:asm.v2">
    <security>
      <requestedPrivileges xmlns="urn:schemas-microsoft-com:asm.v3">
        <requestedExecutionLevel level="asInvoker" uiAccess="false"/>
      </requestedPrivileges>
    </security>
  </trustInfo>
  <asmv3:application>
    <asmv3:windowsSettings xmlns:ws2="http://schemas.microsoft.com/SMI/2016/WindowsSettings">
      <ws2:longPathAware>true</ws2:longPathAware>
    </asmv3:windowsSettings>
  </asmv3:application>
</assembly>
```

**Quellen:** die Form des Elements und die Windows-Version (10, Version 1607 und später)
stehen in „Application Manifests“, Abschnitt `longPathAware`
(<https://learn.microsoft.com/en-us/windows/win32/sbscs/application-manifests>). Die Grenze
von 260 Zeichen, die Bedingung „Registry-Wert `LongPathsEnabled` = 1 **und** Manifest mit
`longPathAware`“ und die Ausnahme der relativen Pfade stehen in „Maximum path length
limitation“ (<https://learn.microsoft.com/en-us/windows/win32/fileio/maximum-file-path-limitation>),
das ist die Referenz [35] der Spezifikation. Beide Seiten habe ich am 25.10.2026 abgerufen.

Der Block `requestedExecutionLevel level="asInvoker"` ist nicht verlangt, aber er macht die
Anforderung NFR-23 („the application never runs with elevated privileges“) zu einer
Eigenschaft der Datei und nicht nur zu einer Absicht. Ein Test hält ihn fest.

### 5.2 Das Einbetten in den Bau

Das Manifest ist ein **Resource** des Executables (Typ 24 `RT_MANIFEST`, Nummer 1). Der Weg:

1. `build.rs` liest `windows.manifest`.
2. `build/resource.rs` schreibt daraus eine Resource-Datei im Format, das `cvtres.exe`
   schreibt: ein COFF-Objekt mit einem Abschnitt `.rsrc`, dessen Baum aus Typ 24 → Nummer 1
   → Sprache `0x0409` → Daten besteht. Der Zeitstempel des Kopfes ist null, damit zwei
   Bauten dieselbe Datei ergeben (NFR-21, reproduzierbarer Bau).
3. `build.rs` übergibt die Datei dem Linker:
   `cargo:rustc-link-arg-bin=phocommander-desktop=<OUT_DIR>/phocommander.resources`.

**Warum so und nicht mit `mt.exe`?** `lld-link` (der Linker, den rustc für
`x86_64-pc-windows-msvc` benutzt) kann ein Manifest nur über `/manifestinput:` einbinden —
und dafür braucht er `mt.exe` aus dem Windows SDK. Der eigene Generator braucht **weder
`mt.exe` noch `cvtres.exe` noch eine neue Abhängigkeit** und funktioniert mit `link.exe`
ebenso wie mit `lld-link`, weil beide eine `.res`-Datei als Eingabe nehmen. Das ist der
Grund für die Entscheidung; sie steht auch in Punkt 10 als offener Punkt.

Der Generator kennt die Maschinennummern der Zielarchitektur (`x86_64` → `0x8664`,
`aarch64` → `0xAA64`, `x86` → `0x014C`, `arm` → `0x01C0`); der Linker weist eine Resource
zurück, deren Maschine nicht zum Programm passt (gemessen: `rust-lld: error: app.res:
machine type x86 conflicts with x64`).

**Für das GNU-Toolchain** (`x86_64-pc-windows-gnu`) nimmt der Linker keine `.res`-Datei an.
Dort benutzt das Build-Skript `windres`, wenn es im Pfad liegt, und sagt es mit
`cargo:warning`, wenn es fehlt. Das Skript bleibt in diesem Fall erfolgreich — ein Bau, der
stirbt, ist schlimmer als ein Programm ohne die Erklärung.

---

## 6. Was ich zusätzlich anfassen musste (Hotspots)

| Datei | Eingriff | Warum | Wie klein |
|---|---|---|---|
| `apps/phocommander-desktop/src/lib.rs` | zwei Zeilen: `pub mod linux;` und `pub mod windows;` | Ohne die Deklaration existieren die beiden Module nicht, und das Crate baut nicht. | 2 Zeilen, keine Logik. |

`src/main.rs` habe ich **nicht** angefasst. Die drei Stellen, an denen der Owner die Module
in den Start hängen kann, stehen fertig im Punkt 9.

---

## 7. Welche Befehle ich wirklich ausgeführt habe — und ihre Ausgabe

**Wichtig:** Ich habe **kein** `cargo build`, **kein** `cargo test` und **kein** `cargo fmt`
ausgeführt (deine Vorgabe: die Sandbox hat 3 GB RAM, ein zweiter Build neben deinem bricht
beide ab). Stattdessen habe ich die neuen Dateien mit `rustc` **allein** geprüft — ohne Cargo,
ohne `target/`, ohne die Cargo-Sperre. Für den Core habe ich eine Prüfkopie gemacht, in der
nur das `thiserror`-Ableitung durch eine handgeschriebene `Display`-Implementierung ersetzt
ist (das Makro lässt sich ohne Cargo nicht bauen); **der Code der neuen Regel ist in der
Prüfkopie wortgleich mit dem Repository.**

### 7.1 Core — Unit-Tests (22, davon 6 neue)

```
$ rustc --edition 2024 --test --crate-name phocommander_core lib.rs -o pathtests
$ ./pathtests
running 22 tests
test path::length::tests::a_path_of_the_limit_of_windows_is_accepted ... ok
test path::length::tests::a_path_that_is_longer_than_the_limit_of_windows_is_refused ... ok
test path::length::tests::the_advice_does_not_ask_for_the_long_paths_when_they_are_already_in_force ... ok
test path::length::tests::the_limit_of_linux_counts_the_bytes_of_the_path ... ok
test path::length::tests::the_message_of_a_refused_path_names_the_path_the_length_and_the_limit ... ok
test path::length::tests::the_message_of_a_refused_path_says_what_the_user_can_do ... ok
test path::tests::a_link_that_is_the_target_itself_is_refused ... ok
...
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

### 7.2 Core — Dokumentationsbeispiele (16, davon 8 neue)

```
$ rustdoc --test --edition 2024 --crate-name phocommander_core lib.rs \
      --extern phocommander_core=libphocommander_core.rlib -L .
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

### 7.3 `src/windows.rs` — Unit-Tests (8, alle neu)

```
$ rustc --edition 2024 --test --crate-name windows_check src/windows.rs \
      --extern phocommander_core=libphocommander_core.rlib -o windowstests
$ ./windowstests
running 8 tests
test tests::a_manifest_with_the_element_and_the_value_true_declares_the_long_paths ... ok
test tests::a_manifest_without_the_element_does_not_declare_the_long_paths ... ok
test tests::the_limit_of_windows_is_the_limit_of_windows ... ok
test tests::the_machine_numbers_of_the_known_architectures_are_known ... ok
test tests::the_manifest_of_the_repository_asks_for_no_elevation ... ok
test tests::the_manifest_of_the_repository_declares_the_long_paths ... ok
test tests::the_resource_carries_the_manifest_as_the_resource_one_of_the_type_manifest ... ok
test tests::the_resource_of_a_machine_of_the_target_carries_the_machine ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

### 7.4 `src/linux.rs` — Unit-Tests (7, alle neu)

```
$ rustc --edition 2024 --test --crate-name linux_check src/linux.rs -o linuxtests
$ ./linuxtests
running 7 tests
test tests::a_machine_without_a_display_server_has_none ... ok
test tests::a_variable_that_is_empty_counts_as_a_variable_that_is_not_set ... ok
test tests::a_wayland_session_is_the_wayland_path ... ok
test tests::an_x11_server_without_a_wayland_session_is_the_fallback ... ok
test tests::the_decision_of_the_machine_is_one_of_the_three ... ok
test tests::the_wayland_session_wins_over_the_x11_server ... ok
test tests::the_wayland_socket_is_a_wayland_session_as_well ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

### 7.5 Dokumentationsbeispiele der Desktop-Module (5, alle neu)

```
$ rustdoc --test --edition 2024 --crate-name phocommander_desktop desktopstub.rs \
      --extern phocommander_core=libphocommander_core.rlib \
      --extern phocommander_desktop=libphocommander_desktop.rlib -L .
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

### 7.6 Das Build-Skript und der Linker — der Beweis, dass das Manifest im Programm landet

```
$ CARGO_CFG_TARGET_OS=windows CARGO_CFG_TARGET_ENV=msvc CARGO_CFG_TARGET_ARCH=x86_64 \
      OUT_DIR=/workspace/.scratch/m3/out ./buildcheck
cargo:rerun-if-changed=build.rs
cargo:rerun-if-changed=build/resource.rs
cargo:rerun-if-changed=windows.manifest
cargo:rustc-link-arg-bin=phocommander-desktop=/workspace/.scratch/m3/out/phocommander.resources

$ rust-lld -flavor link /dll /noentry /out:phocommander.exe empty.obj out/phocommander.resources
link: no error
phocommander.exe: PE32+ executable (DLL) (GUI) x86-64, for MS Windows

$ objdump -x phocommander.exe | sed -n '/Sections:/,/^$/p'
Sections:
Idx Name          Size      VMA               LMA               File off  Algn
  0 .rsrc         000006fe  0000000180001000  0000000180001000  00000200  2**2
                  CONTENTS, ALLOC, LOAD, READONLY, DATA

$ python3 -c "data = open('phocommander.exe','rb').read(); ..."
<ws2:longPathAware>true</ws2:longPathAware>
    </asmv3:windowsSettings>
  </asm
---- asInvoker: True
```

Das `empty.obj` ist ein 20 Byte großes COFF-Objekt ohne Abschnitte, das ich nur erzeugt
habe, damit `lld-link` überhaupt eine Eingabe hat; es ist kein Teil des Projekts.

### 7.7 Formatierung — `rustfmt` direkt, nicht `cargo fmt`

```
$ for f in crates/phocommander-core/src/path.rs crates/phocommander-core/src/path/length.rs \
           crates/phocommander-core/src/path/length/tests.rs crates/phocommander-core/src/path/tests.rs \
           apps/phocommander-desktop/src/windows.rs apps/phocommander-desktop/src/linux.rs \
           apps/phocommander-desktop/build.rs apps/phocommander-desktop/build/resource.rs; do
      rustfmt --edition 2024 --check "$f" && echo "$f clean"; done
crates/phocommander-core/src/path.rs                      clean
crates/phocommander-core/src/path/length.rs               clean
crates/phocommander-core/src/path/length/tests.rs         clean
crates/phocommander-core/src/path/tests.rs                clean
apps/phocommander-desktop/src/windows.rs                  clean
apps/phocommander-desktop/src/linux.rs                    clean
apps/phocommander-desktop/build.rs                       clean
apps/phocommander-desktop/build/resource.rs               clean
```

`rustfmt --check` auf `apps/phocommander-desktop/src/lib.rs` meldet **keinen** Unterschied
(dort habe ich nur zwei Zeilen eingefügt).

### 7.8 Was ich nicht ausgeführt habe

`cargo build --workspace`, `cargo test --workspace`, `cargo clippy --workspace --all-targets
-- -D warnings`, `cargo fmt --check` — **nicht ausgeführt**, nach deiner Vorgabe. Die
Prüfungen oben sind Ersatz, nicht dasselbe: sie kennen weder die anderen Crates des
Workspace noch die Lints des Workspace (`missing_docs`, `clippy::unwrap_used` …).

---

## 8. Die Anforderungen der Spezifikation

| Nummer | Anforderung | Stand | Wie |
|---|---|---|---|
| **FR-28** | „The path length limit of the platform is detected and reported in plain language“ (M3, Must, §4.3, Zeile 238) | **erfüllt** | `check_path_length` + `PathTooLong::message()` im Core, Test mit einem Pfad von 263 Zeichen. |
| **M3, Work-Paket** | „The Windows port with the long path manifest and the path length messaging“ (§M3, Zeile 1344) | **erfüllt** | `windows.manifest` + `build.rs` + `build/resource.rs`; die Einbettung ist mit dem Linker bewiesen (Punkt 7.6). |
| **M3, Work-Paket** | „The Linux port with the Wayland path and the X11 fallback“ (§M3, Zeile 1345) | **erfüllt** | `src/linux.rs`; die Regel ist die von winit, mit Tests für beide Fälle (Punkt 8.1). |
| **NFR-23** | „The application never runs with elevated privileges and never asks for them“ | **unterstützt** | `requestedExecutionLevel level="asInvoker"` im Manifest, mit Test. |
| **NFR-21** | „The build is reproducible from the committed lock file“ | **unterstützt** | Die Resource-Datei entsteht ohne Zeitstempel und ohne Werkzeug des SDK. |
| **NFR-27** | „The application requests no permission that it does not use“ | **unterstützt** | Das Manifest fordert genau eine Sache an: die langen Pfade. |
| **M3** | „The FreeBSD and NetBSD builds“ | **nicht meine Aufgabe** | Die CI-Matrix macht ein anderer Agent. |
| **M3** | „the packaging for the three platforms“ | **nicht meine Aufgabe** | Die Doku macht ein anderer Agent (`docs/guides/`). |

### 8.1 Der Linux-Port: was wirklich nötig war

**Es war kein neuer Code nötig.** `winit` 0.30.13 entscheidet selbst, und zwar so
(gelesen in der Quellkopie des Crates, `~/.cargo/registry/src/index.crates.io-*/winit-0.30.13/src/platform_impl/linux/mod.rs`,
Funktion `EventLoop::new`, Zeilen 731 bis 770):

```rust
// NOTE: Wayland first because of X11 could be present under Wayland as well. Empty
// variables are also treated as not set.
let backend = match (
    attributes.forced_backend,
    env::var("WAYLAND_DISPLAY").ok().filter(|var| !var.is_empty())
        .or_else(|| env::var("WAYLAND_SOCKET").ok()).filter(|var| !var.is_empty()).is_some(),
    env::var("DISPLAY").map(|var| !var.is_empty()).unwrap_or(false),
) {
    (Some(backend), ..) => backend,
    (None, true, _) => Backend::Wayland,
    (None, _, true) => Backend::X,
    (_, wayland_display, x11_display) => { /* Fehler: neither WAYLAND_DISPLAY ... */ }
};
```

Die Regel ist also: **Wayland gewinnt**, wenn `WAYLAND_DISPLAY` (nicht leer) oder
`WAYLAND_SOCKET` gesetzt ist; sonst **X11**, wenn `DISPLAY` (nicht leer) gesetzt ist; sonst
ein Fehler. Genau das steht in `src/linux.rs`, als reine Funktion `display_server_of`, damit
sie ohne Bildschirm prüfbar ist. Der Rückfall auf X11 ist der Fall „keine Wayland-Sitzung“;
`winit` fällt **nicht** auf X11 zurück, wenn eine Wayland-Sitzung zwar benannt, aber nicht
erreichbar ist (der Wayland-Backend verbindet sich mit `Connection::connect_to_env()` und
meldet den Fehler).

Weitere belegte Fakten:

* `eframe` 0.37.0 aktiviert über seine Standard-Features `winit/default`, und `winit`
  0.30.13 hat in seinen Standard-Features **beides**: `x11` und `wayland`. Das Programm
  enthält also beide Backends (`Cargo.toml` von `eframe-0.37.0`, Zeilen 62–72).
* `wayland-dlopen` und `x11-dl` laden die Bibliotheken zur Laufzeit
  (`winit-0.30.13/Cargo.toml`, Zeile 117: `wayland-dlopen = ["wayland-backend/dlopen"]`).
  Das Programm läuft also auf einer Maschine, die nur eine der beiden Bibliotheken hat.
* `winit` bietet mit `EventLoopBuilderExtWayland::with_wayland()` und
  `EventLoopBuilderExtX11::with_x11()` das Erzwingen eines Backends, und `eframe` bietet
  dafür den Haken `NativeOptions::event_loop_builder`. **Ich habe das nicht benutzt**, weil
  es eine neue Abhängigkeit (`winit` im App-Crate) und einen Eingriff in `main.rs` gebraucht
  hätte und weil es nichts an der Entscheidung ändert: winit wählt ohnehin dasselbe. Wenn
  der Owner den Weg *ausdrücklich* erzwingen will, ist es eine Zeile in `main.rs` plus
  `winit` als Abhängigkeit — siehe Punkt 9.

---

## 9. Was offen bleibt (und wie der Owner es schließt)

1. **Den Linux-Port in den Start hängen.** In `main.rs` (das ist deine Datei) eine Zeile
   nach `report_plan(&plan);`:

   ```rust
   println!("display server: {}", phocommander_desktop::linux::display_server().as_str());
   ```

   Und optional das Backend erzwingen (braucht `winit` als Abhängigkeit des App-Crates):

   ```rust
   use winit::platform::wayland::EventLoopBuilderExtWayland as _;
   use winit::platform::x11::EventLoopBuilderExtX11 as _;
   // in den options:
   options.event_loop_builder = Some(Box::new(|builder| match display_server {
       DisplayServer::Wayland => { builder.with_wayland(); }
       DisplayServer::X11 => { builder.with_x11(); }
       DisplayServer::None => {}
   }));
   ```

2. **`check_path_length` anrufen.** Die Regel liegt im Core, aber kein Aufrufer benutzt sie
   noch. Der natürliche Ort ist die Operation, die einen Pfad schreibt (das Crate
   `phocommander-ops`), plus die Oberfläche, die `PathTooLong` anzeigt. Beides liegt nicht
   in meinem Auftrag. **Eigene Entscheidung nötig:** soll die Operation *ablehnen* oder
   *durchlassen und melden*? Ich habe im Core nur die Prüfung gebaut, nicht die decision.

3. **Ein stabiler Fehlercode für die zu lange Pfad.** `PathTooLong` hat bewusst **keinen**
   `ErrorCode`: `ErrorCode` liegt in `crates/phocommander-core/src/error.rs` und die
   Codes trägt die Oberfläche in ihrem Katalog. Ein neuer Code (`path.too_long`, Nummer 1104
   in der Gruppe 11xx) wäre die saubere Anbindung an die Meldungskataloge — das ist eine
   Entscheidung des Owners und ein Eingriff in `error.rs` und in die `.ftl`-Dateien.

4. **Der Build mit `link.exe` statt `lld-link`.** Der Beweis in Punkt 7.6 stammt von
   `rust-lld`, weil das der Linker ist, den der Sandkasten hat. `link.exe` nimmt dieselbe
   `.res`-Datei, aber der erste Windows-Bau des Owners ist der Beweis.

5. **Die Version der Assembly im Manifest** steht fest auf `1.0.0.0`, während das Crate die
   Version `0.1.0` trägt. Das ist harmlos (die Version der Assembly ist nicht die Version
   des Programms), aber wenn der Owner es mitführen will, ist es eine Zeile im Build-Skript
   (Ersetzen von `version="1.0.0.0"` durch die Crate-Version).

6. **Das Ziel `x86_64-pc-windows-gnu`** bekommt das Manifest nur, wenn `windres` auf der
   Maschine liegt. Die CI-Matrix (anderer Agent) sollte entweder das MSVC-Ziel bauen oder
   `windres` bereitstellen.

---

## 10. Die Testabdeckung

| Bereich | geprüft? | Testdatei | Testname | Lücke |
|---|---|---|---|---|
| Pfad länger als 260 Zeichen | ja | `crates/phocommander-core/src/path/length/tests.rs` | `a_path_that_is_longer_than_the_limit_of_windows_is_refused` | — |
| Pfad genau an der Grenze | ja | dieselbe | `a_path_of_the_limit_of_windows_is_accepted` | — |
| Die Meldung nennt Pfad, Länge, Grenze | ja | dieselbe | `the_message_of_a_refused_path_names_the_path_the_length_and_the_limit` | — |
| Die Meldung sagt, was zu tun ist | ja | dieselbe | `the_message_of_a_refused_path_says_what_the_user_can_do` | — |
| Kein Rat zu den langen Pfaden, wenn sie schon gelten | ja | dieselbe | `the_advice_does_not_ask_for_the_long_paths_when_they_are_already_in_force` | — |
| Die Grenze von Linux zählt Bytes | ja | dieselbe | `the_limit_of_linux_counts_the_bytes_of_the_path` | — |
| Das Manifest der Ablage erklärt die langen Pfade | ja | `apps/phocommander-desktop/src/windows.rs` | `the_manifest_of_the_repository_declares_the_long_paths` | — |
| Das Manifest fordert keine Erhöhung | ja | dieselbe | `the_manifest_of_the_repository_asks_for_no_elevation` | — |
| Ein Manifest ohne das Element | ja | dieselbe | `a_manifest_without_the_element_does_not_declare_the_long_paths` | — |
| Die Resource trägt das Manifest als RT_MANIFEST/1 | ja | dieselbe | `the_resource_carries_the_manifest_as_the_resource_one_of_the_type_manifest` | — |
| Die Maschinennummern | ja | dieselbe | `the_machine_numbers_of_the_known_architectures_are_known` | — |
| Wayland, wenn eine Sitzung benannt ist | ja | `apps/phocommander-desktop/src/linux.rs` | `a_wayland_session_is_the_wayland_path` | — |
| X11 als Rückfall | ja | dieselbe | `an_x11_server_without_a_wayland_session_is_the_fallback` | — |
| Wayland gewinnt gegen X11 | ja | dieselbe | `the_wayland_session_wins_over_the_x11_server` | — |
| `WAYLAND_SOCKET` zählt auch | ja | dieselbe | `the_wayland_socket_is_a_wayland_session_as_well` | — |
| Eine leere Variable gilt als „nicht gesetzt“ | ja | dieselbe | `a_variable_that_is_empty_counts_as_a_variable_that_is_not_set` | — |
| Kein Bildschirmserver | ja | dieselbe | `a_machine_without_a_display_server_has_none` | — |
| Das Manifest liegt im Executable | **manuell** | — | Punkt 7.6 (`lld-link` + `objdump`) | kein Test im Repository, weil das ein Windows-Build ist |
| Die Regel bleibt mit winit in Schritt | **manuell** | — | Punkt 8.1 (Quelle von winit 0.30.13 gelesen) | kein Test, weil winit eine Bildschirmsitzung braucht |
| Die Meldung erscheint im Fenster | nein | — | — | die Oberfläche ist nicht meine Aufgabe (Punkt 9.2) |

---

## 11. Die Übergabe als Patch

`deliverable.core-engineer-m3.patch` enthält die Änderungen der drei geänderten Dateien und
die sechs neuen Dateien. Er ist mit `git apply` anwendbar und enthält **nicht** die Dateien
der anderen Agenten (`assets/`, `scripts/build-appimage.sh`, `.github/workflows/ci.yml`).

Anwenden:

```bash
cd /workspace/phocommander
git apply --check deliverable.core-engineer-m3.patch
git apply deliverable.core-engineer-m3.patch
```

---

## 12. Die Quellen, die ich gelesen habe

| Was | Woher | Wofür |
|---|---|---|
| Die Grenze von 260 Zeichen, die Bedingung „Registry **und** Manifest“, die Ausnahme der relativen Pfade | <https://learn.microsoft.com/en-us/windows/win32/fileio/maximum-file-path-limitation> (Referenz [35] der Spezifikation) | die Zahlen und der Rat im Manifest |
| Die Form des Elements `longPathAware`, die Windows-Version | <https://learn.microsoft.com/en-us/windows/win32/sbscs/application-manifests#longpathaware> | das Manifest |
| Die Entscheidung Wayland/X11 | `winit-0.30.13/src/platform_impl/linux/mod.rs`, Zeilen 731–770 (Quellkopie im Registry) | `src/linux.rs` |
| Die Backends von `eframe` | `eframe-0.37.0/Cargo.toml`, Zeilen 62–72 (`default` trägt `winit/default`) | die Behauptung „beide Backends sind im Programm“ |
| `wayland-dlopen`, `x11-dl` | `winit-0.30.13/Cargo.toml`, Zeile 117 | die Behauptung „die Bibliotheken werden zur Laufzeit geladen“ |
| Die Optionen des Linkers | `rust-lld -flavor link -help` (Ausgabe des Werkzeugs) | die Wahl des Einbettens |
| Das Erzeugen des PE | eigener Versuch mit `rust-lld` + `objdump` | der Beweis in Punkt 7.6 |

---

## 13. Die Grenzen dieser Übergabe

* **Ich habe kein `cargo` ausgeführt.** Alle Testausgaben in diesem Dokument stammen aus
  `rustc`-Läufen auf Kopien der Dateien. Sie belegen, dass der neue Code übersetzt, dass
  die neuen Tests grün sind und dass das Manifest in ein PE landet — sie belegen **nicht**,
  dass der ganze Workspace baut, dass die Lints des Workspace (`missing_docs`,
  `clippy::unwrap_used`, `clippy::expect_used`) sauber sind oder dass es keine
  Wechselwirkung mit den Änderungen der anderen Agenten gibt. **Der zentrale Lauf des
  Owners ist die noch offene Prüfung.**
* **Kein Commit, kein Push, kein Tag.** Die Übergabe ist der Patch plus dieses Dokument.
