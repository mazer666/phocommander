# Übergabe Core Engineer — Tests für die Container-Adapter von `phocommander-archive`

| Feld | Wert |
|---|---|
| Rolle | Core Engineer (Rust-Kern, Dateisystemmodell, Auftragswarteschlange, Umbenennung, Archivadapter) |
| Repository | `/workspace/phocommander`, Branch `main`, **kein Commit, kein Push, kein Tag** |
| Arbeitsverzeichnis | `/workspace/phocommander` |
| Auftrag | Die drei Container-Adapter (`tar`, `zip`, `sevenz`) und die Formaterkennung bekommen Tests — die letzte offene Lücke der M1-Abnahme |
| Werkzeuge | Rust 1.99.1 / rustfmt 1.10.0. **Kein `cargo build`, kein `cargo test`, kein `cargo fmt`** (Vorgabe des Owners, RAM-Konflikt mit dem laufenden Gate-Test) |

---

## 0. Kurzfassung für den Teamleiter

**32 neue Tests** in fünf Dateien. Der Adapter-TEST-Stand ist damit vollständig: TAR (8), ZIP (8), 7z (7), Formaterkennung (9).

**Der Bericht beschreibt den Stand vor der Reparatur.** Der Defekt, den der eine Test aufgedeckt hat, ist behoben — siehe Abschnitt 0.1.

**Ein Test schlug bewusst fehl** — er war die Spezifikation eines echten Defekts im Produktionscode (FR-67, Abschnitt 6.1):
`formats::tests::a_compressed_tar_container_is_recognised_as_a_tar_container`.
`detect_format` kennt nur den *unkomprimierten* TAR-Marker. Ein `.tar.gz` / `.tar.bz2` / `.tar.xz` / `.tar.zst` wird deshalb als „kein Container" abgewiesen — `open_archive`, `verify_archive` und damit die **gesamte Extraktion** erreichen die vier Kompressionsstufen des TAR-Adapters nie. Der Fix sind fünf Zeilen in `detect_format` (Abschnitt 6.1).

**Zwei weitere Punkte brauchen eine Owner-Entscheidung** (Abschnitt 6.2 und 6.3): die Dateiendung als Erkennungsmerkmal und die Verzeichnis-Einträge im 7z-Adapter.

**Was der Owner jetzt tun muss:** nichts — der Defekt ist behoben, der Test ist grün.

### 0.1 Die Entscheidungen der Leitenden Instanz (25.10.2026)

| Punkt | Entscheidung | Warum |
|---|---|---|
| **Der Defekt von `detect_format`** | **Behoben.** Die Erkennung prüft jetzt auch die vier Komprimierungen und antwortet mit `ContainerFormat::Tar`. `cargo test -p phocommander-archive` — **53 Tests grün**. | Der TAR-Adapter konnte seine Komprimierungen über die öffentliche API nie erreichen. Das war ein Loch in FR-67. |
| **Die Dateiendung als Merkmal (6.2)** | **Nein.** Die Endung bleibt ein Hinweis, kein Merkmal. Der Test `the_extension_of_the_file_does_not_decide_the_format` bleibt wie er ist. | Eine Datei, die `.zip` heißt und kein ZIP ist, soll als das erkannt werden, was sie ist. Der klare Fehler ist ehrlicher als ein kryptischer Fehler aus dem ZIP-Leser. |
| **Die Verzeichnisse im 7z-Adapter (6.3)** | **Behoben.** Der 7z-Adapter reicht für ein Verzeichnis nichts weiter, genau wie ZIP und TAR. | Die drei Adapter verhalten sich jetzt gleich. Ein Besucher, der den Inhalt eines Verzeichnisses liest, liest sonst Bytes, die ihm nicht gehören. |

---

## 1. Dateiliste

| Pfad | Zeilen | Art | Zweck |
|---|---:|---|---|
| `crates/phocommander-archive/src/formats/test_support.rs` | 196 | **neu** | Die Helfer, die alle drei Adapter-Tests teilen: `TempDir` (temporäres Verzeichnis, das sich beim Drop selbst aufräumt), `CollectingVisitor` (sammelt die Einträge eines Containers), `CollectedEntry` (ein Eintrag mit Inhalt), `collect()` (liest alle Einträge eines Lesers). Nur unter `#[cfg(test)]` kompiliert — der Produktionscode trägt kein Gewicht, das er nicht braucht. |
| `crates/phocommander-archive/src/formats/tar_tests.rs` | 265 | **neu** | Die 8 Tests des TAR-Adapters. Eigene Datei, weil `tar.rs` + Tests sonst über 600 Zeilen läge (Soft-Grenze des Projekts). Eingebunden mit `mod tar_tests;` in `formats/mod.rs`. |
| `crates/phocommander-archive/src/formats/mod.rs` | 547 (war 293) | geändert | + `#[cfg(test)] mod test_support;`, + `#[cfg(test)] mod tar_tests;`, + der Testblock `formats::tests` (9 Tests) für `detect_format`, `detect_tar_compression` und `open_archive`. |
| `crates/phocommander-archive/src/formats/zip.rs` | 539 (war 275) | geändert | + der Testblock `formats::zip::tests` (8 Tests) am Dateiende. Produktionscode unverändert. |
| `crates/phocommander-archive/src/formats/sevenz.rs` | 416 (war 234) | geändert | + der Testblock `formats::sevenz::tests` (7 Tests) am Dateiende. Produktionscode unverändert. |
| `crates/phocommander-archive/src/formats/tar.rs` | 398 | **unverändert** | Der TAR-Adapter selbst wurde nicht angefasst — seine Tests liegen in `tar_tests.rs`. |

**Nachweis, dass kein Produktionscode verändert wurde:**
`git diff --numstat crates/phocommander-archive/` → `254 0 mod.rs`, `182 0 sevenz.rs`, `264 0 zip.rs` — in allen drei Dateien **ausschließlich Zeilen hinzugefügt, keine geändert, keine gelöscht**.

**Zeilenlimit:** `./scripts/check-file-size.sh` meldet für meine Dateien nichts (größte Datei: `mod.rs` mit 547 Zeilen). Die vorhandene Überschreitung (`crates/phocommander-ui/src/keymap_view.rs`, 999 Zeilen) stammt nicht von mir.

---

## 2. Öffentliche API-Übersicht

Die Tests fügen **keine** öffentliche API hinzu. Alle neuen Helfer liegen in einem privaten Modul `crate::formats::test_support` unter `#[cfg(test)]` und sind damit weder aus der Anwendung noch aus einem Integrationstest erreichbar.

| Crate | Modul | Typ/Funktion | Zweck |
|---|---|---|---|
| `phocommander-archive` | `crate::formats::test_support` (privat, `cfg(test)`) | `TempDir::new(label)` / `join(name)` / `write_file(name, bytes)` | Temporäres Verzeichnis mit einmaligem Namen (Prozess, Zeit, Zähler), das sich beim Drop selbst aufräumt. |
| `phocommander-archive` | `crate::formats::test_support` | `CollectingVisitor` (`impl EntryVisitor`) | Sammelt die Einträge eines Containers inklusive Inhalt, damit ein Test einen Container mit dem vergleichen kann, was er geschrieben hat. |
| `phocommander-archive` | `crate::formats::test_support` | `CollectedEntry` | Ein gelesener Eintrag: Name, `name_is_lossy`, Art, Größe, Modus, Linkziel, gespeicherte Prüfsumme, Inhalt. |
| `phocommander-archive` | `crate::formats::test_support` | `collect(reader) -> Result<Vec<CollectedEntry>, ArchiveError>` | Liest alle Einträge eines Lesers mit *einem* Besucher — damit ist ein Unterschied zwischen zwei Adaptern ein Unterschied der Adapter und nicht der Tests. |

---

## 3. Teststand

### 3.1 Was gemessen wurde

| Kommando | Ergebnis |
|---|---|
| `rustfmt --edition 2024 --check` (auf allen fünf Dateien, **ohne** `cargo fmt`) | **RUSTFMT CLEAN** — die Dateien parsen und sind formatkonform. `rustfmt` ist der einzelne Binäraufruf, kein Cargo-Prozess, kein Build, kein Test. |
| `./scripts/check-file-size.sh` | Keine neue Überschreitung (größte neue Datei 547 Zeilen). |
| `git diff --numstat` | Nur Additionen, kein Produktionscode verändert. |
| `cargo test --workspace` | **Nicht gelaufen** — vom Owner ausdrücklich untersagt (RAM-Konflikt mit dem laufenden Gate-Test). Die Testausgabe kann ich daher **nicht** liefern; sie entsteht beim zentralen Lauf des Owners. |

**Ich behaupte nicht, dass die Tests grün sind.** Ich habe sie geschrieben und die API der drei Container-Crates (`tar` 0.4.46, `zip` 9.1.0, `sevenz-rust2` 0.24.1) gegen die Registry-Quellen geprüft, aber **nicht kompiliert und nicht ausgeführt**.

### 3.2 Testbestand

| Datei | Testmodul | Tests |
|---|---|---:|
| `src/formats/tar_tests.rs` | `formats::tar_tests` | 8 |
| `src/formats/zip.rs` | `formats::zip::tests` | 8 |
| `src/formats/sevenz.rs` | `formats::sevenz::tests` | 7 |
| `src/formats/mod.rs` | `formats::tests` | 9 |
| | **Summe** | **32** |

### 3.3 Die einzelnen Tests

**TAR — `formats::tar_tests`**

| Test | Fall |
|---|---|
| `the_reader_returns_the_entries_that_were_written` | Runde: Name, Art, Größe, Modusbits |
| `the_reader_returns_the_content_that_were_written` | Runde: Inhalt byteweise identisch |
| `the_container_keeps_the_structure_of_the_directories` | Verzeichnis `docs` + Datei `docs/readme.md` |
| `the_reader_reports_a_link_as_a_link_and_not_as_its_target` | Link kommt als Link zurück (`EntryKind::Symlink`, `link_target`), nicht als Ziel |
| `the_container_is_read_back_with_every_compression` | **Alle vier Kompressionen + unkomprimiert** (None, Gzip, BZip2, Xz, Zstd) |
| `a_file_that_is_no_container_is_an_error_and_not_a_panic` | Fehler statt Panic (`catch_unwind` + Rückgabewert) |
| `the_writer_refuses_a_name_that_leaves_the_destination` | `../escape.txt` → `Refused`/`Traversal` |
| `a_name_that_is_not_valid_utf8_is_reported_as_lossy_and_is_not_cut_short` | **Unix only**: Name `broken\xffname.txt`, mit dem Container-Crate gebaut (fremder Erzeuger) → `name_is_lossy == true`, Name **nicht abgeschnitten** (`broken\u{fffd}name.txt`) |

**ZIP — `formats::zip::tests`**

| Test | Fall |
|---|---|
| `the_reader_returns_the_entries_that_were_written` | Runde: Name, Art, Größe |
| `the_reader_returns_the_content_that_were_written` | Runde: Inhalt identisch |
| `the_container_keeps_the_structure_of_the_directories` | Verzeichnis (im ZIP-Format mit `/` am Ende) + Datei |
| `the_reader_reports_a_link_as_a_link_and_not_as_its_target` | Link kommt als Link zurück |
| `the_reader_reports_the_checksum_that_the_container_stores` | Die im Container gespeicherte CRC-32 == `Crc32` des Projekts (FR-71) |
| `a_file_that_is_no_container_is_an_error_and_not_a_panic` | Fehler statt Panic |
| `the_writer_refuses_a_name_that_leaves_the_destination` | `../escape.txt` → `Refused`/`Traversal` |
| `a_name_that_is_not_valid_utf8_is_reported_as_lossy_and_is_not_cut_short` | Die Bytes des Namens werden nach dem Schreiben gepatcht (der Writer des Container-Crates nimmt nur `String`) → `name_is_lossy == true`, Name nicht abgeschnitten |

**7z — `formats::sevenz::tests`**

| Test | Fall |
|---|---|
| `the_reader_returns_the_entries_that_were_written` | Runde: Name, Art, Größe |
| `the_reader_returns_the_content_that_were_written` | Runde: Inhalt identisch |
| `the_container_keeps_the_structure_of_the_directories` | Verzeichnis `docs` + Datei `docs/readme.md` |
| `the_reader_reports_the_checksum_that_the_container_stores` | Die im Container gespeicherte CRC-32 == `Crc32` des Projekts |
| `the_name_of_an_entry_is_never_lossy_because_the_format_carries_it_as_utf16` | Das 7z-Format speichert Namen als UTF-16 — ein Name, der kein UTF-8 ist, **kann** nicht vorkommen; `name_is_lossy` bleibt `false` |
| `the_writer_refuses_to_store_a_symbolic_link` | Das Format kennt keine Links → `ArchiveError::Format` |
| `a_file_that_is_no_container_is_an_error_and_not_a_panic` | Fehler statt Panic |

**Erkennung — `formats::tests`**

| Test | Fall |
|---|---|
| `a_zip_container_is_recognised_by_its_magic_bytes` | ZIP per Magic Bytes |
| `a_tar_container_is_recognised_by_its_magic_bytes` | TAR per Magic Bytes |
| `a_7z_container_is_recognised_by_its_magic_bytes` | 7z per Magic Bytes |
| `a_container_with_a_wrong_extension_is_recognised_by_its_content` | ZIP-Container mit der Endung `.tar` → **ZIP** (der Inhalt entscheidet) |
| `a_compressed_tar_container_is_recognised_as_a_tar_container` | **SCHLÄGT HEUTE FEHL** — siehe 6.1 |
| `the_extension_of_the_file_does_not_decide_the_format` | Eine Textdatei mit der Endung `.zip` → **Fehler**, nicht „ZIP" — dokumentiert den heutigen Vertrag (siehe 6.2) |
| `a_file_that_is_no_container_is_an_error_and_not_a_panic` | Fehler statt Panic (`catch_unwind`) |
| `the_compression_of_a_tar_container_is_recognised_by_its_magic_bytes` | Alle fünf Kompressionsstufen an ihren Magic Bytes erkannt |
| `open_archive_opens_a_container_of_every_format` | `open_archive` öffnet ZIP, TAR und 7z und liefert den Inhalt |

---

## 4. Testabdeckungs-Matrix

| Bereich | getestet? | Testdatei | Testname | Lücke |
|---|---|---|---|---|
| TAR: Runde (Einträge) | ✅ | `formats/tar_tests.rs` | `the_reader_returns_the_entries_that_were_written` | — |
| TAR: Runde (Inhalt) | ✅ | `formats/tar_tests.rs` | `the_reader_returns_the_content_that_were_written` | — |
| TAR: Verzeichnisse | ✅ | `formats/tar_tests.rs` | `the_container_keeps_the_structure_of_the_directories` | — |
| TAR: symbolische Links | ✅ | `formats/tar_tests.rs` | `the_reader_reports_a_link_as_a_link_and_not_as_its_target` | — |
| TAR: Kompression (5 Stufen) | ✅ | `formats/tar_tests.rs` | `the_container_is_read_back_with_every_compression` | — |
| TAR: kein Container → Fehler | ✅ | `formats/tar_tests.rs` | `a_file_that_is_no_container_is_an_error_and_not_a_panic` | — |
| TAR: Name ohne UTF-8 | ✅ (unix) | `formats/tar_tests.rs` | `a_name_that_is_not_valid_utf8_...` | Unter Windows nicht ausführbar (das Format kann dort keine Byte-Namen tragen) |
| TAR: Prüfsumme | ➖ | — | — | Das TAR-Format speichert keine (nur die Header-Prüfsumme) — bewusst nicht getestet |
| ZIP: Runde (Einträge + Inhalt) | ✅ | `formats/zip.rs` | `the_reader_returns_the_entries_that_were_written`, `..._content_...` | — |
| ZIP: Verzeichnisse | ✅ | `formats/zip.rs` | `the_container_keeps_the_structure_of_the_directories` | — |
| ZIP: symbolische Links | ✅ | `formats/zip.rs` | `the_reader_reports_a_link_as_a_link_and_not_as_its_target` | — |
| ZIP: gespeicherte CRC-32 | ✅ | `formats/zip.rs` | `the_reader_reports_the_checksum_that_the_container_stores` | — |
| ZIP: kein Container → Fehler | ✅ | `formats/zip.rs` | `a_file_that_is_no_container_is_an_error_and_not_a_panic` | — |
| ZIP: Name ohne UTF-8 | ✅ | `formats/zip.rs` | `a_name_that_is_not_valid_utf8_...` | — |
| ZIP: Name mit Verzeichnis (`../`) | ✅ | `formats/zip.rs` | `the_writer_refuses_a_name_that_leaves_the_destination` | — |
| 7z: Runde (Einträge + Inhalt) | ✅ | `formats/sevenz.rs` | `the_reader_returns_the_entries_that_were_written`, `..._content_...` | — |
| 7z: Verzeichnisse | ✅ | `formats/sevenz.rs` | `the_container_keeps_the_structure_of_the_directories` | — |
| 7z: gespeicherte CRC-32 | ✅ | `formats/sevenz.rs` | `the_reader_reports_the_checksum_that_the_container_stores` | — |
| 7z: Name ohne UTF-8 | ✅ (als Eigenschaft) | `formats/sevenz.rs` | `the_name_of_an_entry_is_never_lossy_...` | Das Format kann solche Namen nicht tragen |
| 7z: Links | ✅ | `formats/sevenz.rs` | `the_writer_refuses_to_store_a_symbolic_link` | — |
| 7z: kein Container → Fehler | ✅ | `formats/sevenz.rs` | `a_file_that_is_no_container_is_an_error_and_not_a_panic` | — |
| Erkennung: ZIP/TAR/7z per Magic Bytes | ✅ | `formats/mod.rs` | `a_zip_...`, `a_tar_...`, `a_7z_...` | — |
| Erkennung: falsche Endung, richtiger Inhalt | ✅ | `formats/mod.rs` | `a_container_with_a_wrong_extension_is_recognised_by_its_content` | — |
| Erkennung: **richtige Endung, falscher Inhalt** | ⚠️ | `formats/mod.rs` | `the_extension_of_the_file_does_not_decide_the_format` | **Offen:** die Endung wird heute nicht gelesen (6.2) |
| Erkennung: komprimiertes TAR | ❌ **fehlt im Code** | `formats/mod.rs` | `a_compressed_tar_container_is_recognised_as_a_tar_container` | **Defekt**, Test schlägt fehl (6.1) |
| Erkennung: kein Container → Fehler | ✅ | `formats/mod.rs` | `a_file_that_is_no_container_is_an_error_and_not_a_panic` | — |
| Erkennung: Kompression per Magic Bytes | ✅ | `formats/mod.rs` | `the_compression_of_a_tar_container_is_recognised_by_its_magic_bytes` | — |
| `open_archive` für alle drei Formate | ✅ | `formats/mod.rs` | `open_archive_opens_a_container_of_every_format` | — |
| Extraktionssicherheit (Pfade, Bomben, Grenzen) | ➖ | `tests/security/*` | — | **Nicht angefasst** — die Adapter-Tests umgehen die Sicherheit nicht, sie lesen und schreiben nur |
| Verschlüsselte Container (FR-68) | ➖ | — | — | Das Schreiben verschlüsselter Container ist im Adapter nicht vorhanden (siehe 6.4) |

---

## 5. Determinismus

Die Adapter-Tests prüfen keine Umbenennungsregeln — die liegen in `phocommander-ops`, nicht in `phocommander-archive`. Für die Umbenennungsregeln gilt weiterhin die Evidenz aus der vorherigen Übergabe (`deliverable.core-engineer.md`); dieser Auftrag hat sie nicht berührt.

Was die neuen Tests deterministisch halten:
- **Die Reihenfolge der Einträge** wird nicht über die Position, sondern über den Namen bestimmt (`entry(&entries, name)`), weil das 7z-Format die Verzeichnis-Einträge nach den Blöcken liefert.
- **Die Temporärverzeichnisse** bekommen einen einmaligen Namen (Prozess-ID, Nanosekunden, Zähler) — zwei parallel laufende Tests teilen sich kein Verzeichnis.
- **Kein Zeit- oder Zufallswert** fließt in eine Zusicherung: der TAR-Adapter schreibt `mtime = 0`, und die Prüfsummen werden gegen den Inhalt der Datei gerechnet, nicht gegen einen gespeicherten Wert.

---

## 6. Befunde (das ist das Wichtigste in dieser Übergabe)

### 6.1 Defekt: `detect_format` kennt kein komprimiertes TAR (FR-67) — 1 Test schlägt fehl

`crates/phocommander-archive/src/formats/mod.rs`, `detect_format`:

```rust
if signature.starts_with(&signature::ZIP) || signature.starts_with(&signature::ZIP_EMPTY) { … }
if signature.starts_with(&signature::SEVENZ) { … }
if is_tar(&signature) { … }          // ← nur der unkomprimierte TAR-Marker
Err(ArchiveError::Format { … })
```

`detect_tar_compression` kennt GZip, BZip2, XZ und Zstd — `detect_format` ruft es aber **nie** auf, weil es vorher schon abbricht. Folge: `open_archive("x.tar.gz")` scheitert, und damit auch `verify_archive` und `extract_archive`. **Die vier Kompressionsstufen des TAR-Adapters sind über die öffentliche API nicht erreichbar.** FR-67 („TAR with GZip, BZip2, XZ and Zstd are supported") ist damit zur Hälfte nicht erfüllt.

Der Test `a_compressed_tar_container_is_recognised_as_a_tar_container` ist die Spezifikation dieses Vertrags und **schlägt bis zur Reparatur fehl**. Er ist bewusst nicht weggelassen und nicht auf das heutige Verhalten umgeschrieben: ein Test, der den Defekt festschreibt, wäre schlimmer als kein Test.

**Vorgeschlagene Änderung (5 Zeilen, von mir nicht ausgeführt — der Auftrag war „nur Test-Dateien"):**

```rust
    // A compressed stream can carry a TAR container. The detection has to
    // know that, or the four compressions of the adapter can never be
    // reached.
    if signature.starts_with(&signature::GZIP)
        || signature.starts_with(&signature::BZIP2)
        || signature.starts_with(&signature::XZ)
        || signature.starts_with(&signature::ZSTD)
    {
        return Ok(ContainerFormat::Tar);
    }
```

`open_archive` wählt danach über `detect_tar_compression` bereits den richtigen Dekoder ✓.

### 6.2 Offen: die Dateiendung als Erkennungsmerkmal

Der Auftrag nannte „Dateiendung **und** Magic Bytes". `detect_format` liest die Endung **nicht** — der Doc-Kommentar der Funktion sagt sogar ausdrücklich: „the extension of a file is a hint, not a fact. The magic bytes of the container decide what the file really is."

Ich habe den heutigen Vertrag in `the_extension_of_the_file_does_not_decide_the_format` festgeschrieben (Textdatei mit der Endung `.zip` → Fehler). **Das ist eine bewusste Entscheidung, keine Verweigerung:** der Test benennt den Konflikt zwischen Brief und Code. Wenn der Owner die Endung als Merkmal will, ist der Test umzudrehen und `detect_format` um einen Fallback zu ergänzen:

```rust
    // The extension is a hint. It is only used when the magic bytes tell
    // nothing, so that a file that is called ".zip" is at least reported as
    // the ZIP container it pretends to be — and the user then sees why it
    // is broken.
    match path.extension().and_then(|extension| extension.to_str()) {
        Some("zip") => return Ok(ContainerFormat::Zip),
        Some("7z") => return Ok(ContainerFormat::SevenZ),
        Some("tar" | "tgz" | "tbz2" | "txz" | "tzst") => return Ok(ContainerFormat::Tar),
        _ => {}
    }
```

**Owner-Entscheidung:** Endung als Fallback — ja oder nein?

### 6.3 Beobachtung: der 7z-Adapter meldet Verzeichnisse mit Inhalt

`formats/sevenz.rs` reicht für **jeden** Eintrag einen Reader weiter (`Some(content)`), auch für Verzeichnisse. ZIP und TAR reichen für Verzeichnisse `None` weiter. Für die Extraktion ist das unkritisch (sie prüft `entry.is_file()`), aber die drei Adapter sind damit nicht symmetrisch. Ich habe die Tests deshalb so geschrieben, dass sie den Inhalt **nur bei Dateien** prüfen. Wenn die Symmetrie gewünscht ist, ist das eine Zeile im 7z-Adapter.

### 6.4 Nicht in diesem Auftrag: verschlüsselte Container (FR-68)

FR-68 verlangt „Archives can be created, including encrypted". Die Writer der drei Adapter bieten **keine** Verschlüsselung (der `zip`-Crate kann sie, der Adapter ruft sie nicht auf; 7z und TAR gar nicht). Das ist eine Lücke der M1-Abnahme, aber sie gehört in die Produktionsimplementierung und nicht in die Tests — ich kann sie nicht durch einen Test schließen, der eine nicht vorhandene Funktion erwartet.

---

## 7. Was der Owner jetzt tun muss

1. `cargo test -p phocommander-archive` — erwartet: **31 grün, 1 rot** (`a_compressed_tar_container_is_recognised_as_a_tar_container`).
2. Entscheiden, ob `detect_format` die fünf Zeilen aus 6.1 bekommt. Danach ist der Test grün und bleibt als Wächter stehen.
3. Entscheiden zu 6.2 (Endung als Fallback) — der Test in `formats/mod.rs` ist umzudrehen, falls ja.
4. `cargo clippy --workspace --all-targets -- -D warnings` und `cargo fmt --check` (der Formatter ist bereits vorformatiert: `rustfmt --check` ist grün).

**Sollte der zentrale Lauf etwas anderes zeigen als 31/1**, sind die wahrscheinlichsten Ursachen in dieser Reihenfolge: (a) die ZIP-Verzeichnis-Einträge tragen die Endung `/` — der Test prüft `"docs/"`; (b) die Anzahl der Namensvorkommen im ZIP-Container (der Test ersetzt **alle** Vorkommen und verlangt nur `> 0`); (c) der Moduswert des TAR-Headers (`0o644`).

---

## 8. Nicht ausgeführt (Wahrheitsgemäßheit)

| Kommando | Status | Grund |
|---|---|---|
| `cargo build` / `cargo check` | **nicht gelaufen** | Vorgabe des Owners (RAM-Konflikt mit dem laufenden Gate-Test) |
| `cargo test` | **nicht gelaufen** | dito — die Testausgabe dieser Übergabe kann ich nicht liefern |
| `cargo clippy --workspace --all-targets -- -D warnings` | **nicht gelaufen** | dito |
| `cargo fmt` | **nicht gelaufen** | dito — stattdessen der einzelne `rustfmt`-Aufruf auf den fünf Dateien (`--check` grün) |
| `./scripts/check-file-size.sh` | **gelaufen** | braucht keinen Build: keine neue Überschreitung |
| `rustfmt --edition 2024 --check` (5 Dateien) | **gelaufen** | RUSTFMT CLEAN |
| `git diff --numstat` | **gelaufen** | nur Additionen |

Es wurden keine Commits, keine Pushes, keine Tags erzeugt. Die Dateien `README.md` und `docs/STATUS.md` waren bereits vor dieser Sitzung geändert — sie stammen **nicht** von mir und wurden nicht angefasst.
