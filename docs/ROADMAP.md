# Phocommander — Roadmap

**Status:** Living document · **Owner:** Product Manager · **Letzte Aktualisierung:** 25.10.2026
**Verbindliche Grundlage:** [`REQUIREMENTS.md`](REQUIREMENTS.md) Fassung 2.0

Dieses Dokument ist die **Roadmap und das Tracking** des Projekts. Es sagt, welcher Meilenstein als
nächstes ansteht, was in ihm liegen muss, und welche Prüfpunkte einzuhalten sind, bevor er als
erreicht gilt.

Die Trennung zu den anderen Dokumenten ist strikt:

| Frage | Dokument |
|---|---|
| Was muss das Produkt können? | [`REQUIREMENTS.md`](REQUIREMENTS.md) — die Anforderungen FR-xx und NFR-xx |
| Wie wird jedes Detail geprüft? | [`ACCEPTANCE-CRITERIA.md`](ACCEPTANCE-CRITERIA.md) — 121 Akzeptanzkriterien mit „How to Verify" und „Pass Condition" |
| Welche Anforderung gehört zu welchem Meilenstein? | [`REQUIREMENTS-IDS.md`](REQUIREMENTS-IDS.md) — der Katalog mit 154 Zeilen |
| **Woran merke ich, wo ich stehe?** | **dieses Dokument** |
| Wurde ein Meilenstein abgenommen? | [`acceptance/`](acceptance/) — ein Dokument je Meilenstein mit den Belegen |

## Wie diese Roadmap gelesen wird

Jeder Meilenstein hat eine **Checkliste**. Die Checkliste ist die verbindliche Bedingung: erst wenn
jeder Punkt erfüllt und **belegt** ist, gilt der Meilenstein als erreicht. Ein Punkt ohne Beleg ist
nicht erfüllt.

Die drei Spalten der Checkliste sind:

| Spalte | Bedeutung |
|---|---|
| **Punkt** | Was einzuhalten ist |
| **Nachweis** | Woher der Beleg kommt — ein Test, ein Kommando, ein Dokument |
| **Status** | `offen`, `in Arbeit`, `erfüllt`, `NOT RUN` (mit Begründung) |

Die Status-Spalte wird von Hand gepflegt und bei jedem Meilensteinabschluss aktualisiert. Sie
ist der einzige Ort, an dem der Ist-Stand steht.

## Die Regeln, die für jeden Meilenstein gelten

Diese Punkte gelten **in jedem** Meilenstein, ohne Ausnahme. Sie sind nicht verhandelbar und
werden bei jeder Abnahme geprüft.

| Nr. | Punkt | Nachweis |
|---|---|---|
| R-01 | Der Workspace baut ohne Fehler | `cargo build --workspace --all-targets --locked` |
| R-02 | Alle Tests laufen durch, keiner wird übersprungen | `cargo test --workspace --locked` |
| R-03 | Die Formatierung ist sauber | `cargo fmt --all --check` |
| R-04 | Die statische Analyse ist ohne Warnung | `cargo clippy --workspace --all-targets --locked -- -D warnings` |
| R-05 | Keine Abhängigkeit mit Copyleft-Lizenz | `cargo deny check` |
| R-06 | Keine bekannte Schwachstelle in der Abhängigkeitsliste | `cargo audit` |
| R-07 | Die Abhängigkeitsrichtung ist eingehalten | `scripts/check-crate-boundaries.sh` |
| R-08 | Keine Quelldatei über 600 Zeilen (Zielwert) beziehungsweise 900 (Maximum) | `scripts/check-file-size.sh` |
| R-09 | Keine Telemetrie- oder Analyse-Abhängigkeit | `scripts/check-no-telemetry.sh` |
| R-10 | Keine tote Verlinkung in der Dokumentation | `scripts/check-docs-links.py` |
| R-11 | Die Anforderungs-IDs von Spec und Katalog sind identisch | `scripts/check-requirement-ids.py` |
| R-12 | Die Aktions-IDs von Keymap, Code und Dokumentation sind identisch | `scripts/check-action-ids.py` und `scripts/check-keymap-consistency.py` |
| R-13 | Die ausgelieferte Konfiguration entspricht dem Schema | `scripts/check-config-schema.py` |
| R-14 | Die Interface-Profile halten ihre Werte | `scripts/check-interface-profiles.py` |
| R-15 | Jedes Gate beweist seine Wirkung: rot bei Bruch, grün nach Reparatur | `scripts/check-gates-negative.sh` |
| R-16 | Jede öffentliche Funktion hat einen Test | Testabdeckungs-Matrix der Abnahme |
| R-17 | Jede Härtungsregel hat einen negativen Test | `tests/security/` |
| R-18 | Kein `unwrap()` oder `expect()` im Produktivcode, kein `unsafe` | Workspace-Lints plus Clippy |
| R-19 | Jedes Modul trägt einen Doc-Kommentar im WHAT/WHY/EXAMPLE-Stil | `missing_docs`-Lint |
| R-20 | Die Abnahme des Meilenstands liegt unter `docs/acceptance/` | Datei vorhanden, Entscheidung dokumentiert |
| R-21 | Der Stand liegt auf `main` in GitHub | Commit auf `main` |

---

## Die Übersicht

| Meilenstein | Inhalt | Zustand |
|---|---|---|
| **M0** — Foundation | Workspace, CI, Konfiguration, Bedrohungsmodell, Dokumentation | **abgenommen** |
| **M1** — The working file browser | Dateisystemmodell, Auftragswarteschlange, Umbenennung, Betrachter Stufe 1, Archive | **in Prüfung** |
| **M2** — Maturity on macOS | Registerkarten, Profile, sichere Kopiermodi, Layouts je Kontext | **in Arbeit** |
| **M2b** — Internationalisierung | Englisch und Deutsch vollständig, lokalisierte Formatierung, Sprachwechsel zur Laufzeit | offen |
| **M3** — The other desktop platforms | Windows, Linux, BSD | offen |
| **M4** — Android | SAF, Touch-Kontext, Store | offen |
| **M5** — iPadOS | Dokumenten-Picker, Bookmarks | offen |
| **M6** — Remote sources | SFTP, FTP, SMB, WebDAV | offen |
| **M7** — Cloud and controllers | OAuth, S3, Controller-Kontext | offen |
| **M8** — Television | D-Pad, Aktionspalette, 10-Fuß | offen |
| **M9** — The remaining capabilities | Betrachter Stufe 3, Editor, Tags, Makros | offen |

---

## M0 — Foundation

**Ziel.** Ein Repository, das sich auf drei Betriebssystemen selbst baut, prüft und testet, mit der
Spezifikation als verbindlicher Grundlage für alles Weitere.

| Punkt | Nachweis | Status |
|---|---|---|
| Cargo-Workspace mit allen Crates, gepinnten Abhängigkeiten, committed Lockfile | `Cargo.toml`, `Cargo.lock` | erfüllt |
| Die elf gemeinsamen Regeln R-01 bis R-21 | alle Gates | erfüllt |
| Die fünf CI-Jobs: Qualität, Desktop-Matrix, Quer-Kompilierung, Leistung, Nachtlauf | `.github/workflows/ci.yml` | erfüllt |
| Die Werkzeugversionen sind gepinnt und im Workflow verifiziert | `ci.yml`, `docs/REQUIREMENTS.md` 5.8 | erfüllt |
| `deny.toml` erlaubt ausschließlich permissive Lizenzen, kein GPL und kein AGPL | `deny.toml` | erfüllt |
| Die Konfigurationsdatei mit Vorgaben, die Tastenbelegung, die Umbenennungsvorlagen | `config/` | erfüllt |
| Das Bedrohungsmodell mit dem Residualrisiko und dem Benennung der TOCTOU-Lücke | `docs/security/THREAT_MODEL.md` | erfüllt |
| Die negativen Tests für das Zip-Slip, den Symlink-Escape, die SUID/SGID-Bits und die Dekompressionsbombe | `tests/security/` | erfüllt |
| Die Abnahme mit den Belegen unter `docs/acceptance/logs/` | `docs/acceptance/M0-ACCEPTANCE.md` | erfüllt |

**Ergebnis.** 129 Tests grün, kein übersprungener Test, die Auflagen der Abnahme geschlossen.

---

## M1 — The working file browser

**Ziel.** Ein funktionaler Zwei-Fenster-Dateibrowser auf macOS, der kopiert, verschiebt, umbenennt,
anseht und Archive öffnet.

| Punkt | Nachweis | Status |
|---|---|---|
| Die elf gemeinsamen Regeln R-01 bis R-21 | alle Gates | in Arbeit — wird am Ende neu gemessen |
| Das Dateisystemmodell mit `Entry`, `EntryKind`, `Source`, `MutableSource` | `phocommander-core` | erfüllt |
| Die Auftragswarteschlange mit Fortschritt, Rate, Abbruch und Fehlersammlung | `phocommander-ops` | erfüllt |
| Die Umbenennungs-Engine mit den Platzhaltern, dem Ersetzen und dem Zähler | `phocommander-ops` | in Arbeit — Tests werden neu gemessen |
| Die Serienfolgen-Vorlage: Erkennung, Staffel mit zwei und drei Ziffern, Windows-Zeichen | `config/templates/series.toml` | erfüllt — mit Test für die zweistellige Staffel |
| Die Konfliktsprung-Liste mit F3 und Shift+F3 | `phocommander-ops` | erfüllt |
| Der Betrachter Stufe 1: Text, Hex, Bilder, Archivinhalt, Metadaten | `phocommander-viewer` | erfüllt |
| Die Archivadapter für ZIP, TAR mit fünf Kompressionen und 7z | `phocommander-archive` | erfüllt |
| Die gehärtete Extraktion: kanonisierte Pfade, keine Symlinks aus Archivmaterial, keine SUID/SGID-Bits | `phocommander-archive`, `tests/security/` | erfüllt |
| Die Filterung des aktuellen Ordners mit der Pattern-Sprache und der Live-Trefferzahl | `phocommander-ui` | erfüllt |
| Die Determinismus-Invariante der Umbenennung, geprüft mit einem Property-Test | `phocommander-ops` | erfüllt |
| Das Eigenschafts-Gate für den Rahmen: 10.000 und 100.000 Einträge | `crates/phocommander-ui/tests/` | erfüllt |
| Die achtzehn gemeinsamen Regeln der Sicherheitsprüfung, jede mit einem negativen Test | `tests/security/` | erfüllt |

**Offen.** Die Abnahme M1 liegt vor, wurde aber während der Parallel-Arbeit der M2-Agenten gemessen
und sagt deshalb **„not accepted"**. Nach dem Abschluss von M2 wird sie auf dem ruhenden Stand neu
geschrieben.

---

## M2 — Maturity on macOS

**Ziel.** Eine Anwendung, die im Alltag einen Dateibrowser ersetzen kann, in beiden Sprachen, mit
einem Interface, das sich der Kontext anpasst.

| Punkt | Nachweis | Status |
|---|---|---|
| Die elf gemeinsamen Regeln R-01 bis R-21 | alle Gates | offen — wird am Ende gemessen |
| Die Registerkarten mit eigenem Zustand je Karte, verschiebbar zwischen den Panels | `phocommander-ui` | in Arbeit |
| Die Umschaltung der Anordnung ohne Neustart | `phocommander-ui` | in Arbeit |
| Der Verzeichnisbaum je Panel, ohne rekursives Einlesen | `phocommander-ui` | in Arbeit |
| Die Schnellansicht des markierten Eintrags | `phocommander-ui` | in Arbeit |
| Die strukturell unterschiedlichen Layouts je Kontext | `phocommander-ui` | in Arbeit |
| Die Oberfläche ist aufgeräumt: gleiche Abstände, eine Trennlinie, sichtbarer Fokus | `phocommander-ui` | in Arbeit |
| Der atomare Kopiermodus: temporäre Datei, prüfen, umbenennen, immer aufräumen | `phocommander-ops` | in Arbeit |
| Der Synchronisationsplan mit den Zuständen je Eintrag, ohne Plan keine Bewegung | `phocommander-ops` | in Arbeit |
| Die atomare Umbenennung: alle Dateien oder keine | `phocommander-ops` | in Arbeit |
| Das Wiederholen nur der fehlgeschlagenen Einträge | `phocommander-ops` | in Arbeit |
| Der Verzeichnisvergleich auf Inhalt, Datum und Größe | `phocommander-ops` | in Arbeit |
| Das Rückgängig, mit der sichtbaren Einschränkung | `phocommander-ops` | in Arbeit |
| Die Archivprüfung ohne Entpacken | `phocommander-archive` | in Arbeit |
| Englisch und Deutsch vollständig, mit identischem Schlüsselsatz | `phocommander-i18n` | in Arbeit |
| Die Sprachkette: angefordert → Kette → Englisch → der Schlüssel selbst | `phocommander-i18n` | in Arbeit |
| Die Sprache ist zur Laufzeit umschaltbar, ohne Neustart | `phocommander-i18n` | in Arbeit |
| Eine dritte Sprache kommt ohne Codeänderung aus | `phocommander-i18n` | in Arbeit — `locales/fr/` |
| Die Zahlen-, Größen-, Datums- und Zeitformate folgen dem System | `phocommander-i18n` | in Arbeit |
| Die Interface-Profile `clean` und `full` sind wirksam, nicht nur beschrieben | `config/profiles/` | in Arbeit |
| Jede Aktion einzeln schaltbar: `visible`, `palette`, `hidden` | `phocommander-ui` | in Arbeit |
| Ein Profil enthält nur Oberflächenentscheidungen | `scripts/check-interface-profiles.py` | in Arbeit |
| Der About-Dialog nennt die Zahl der tatsächlich unterstützten Formate | `phocommander-viewer` | offen |
| Die Leistungsziele NFR-01 bis NFR-08 sind gemessen | `benches/`, `docs/acceptance/logs/` | offen |

## M2b — Internationalisierung

**Ziel.** Die Oberfläche in **zwei Sprachen** — Englisch und Deutsch — mit derselben Qualität, mit
lokalisierter Formatierung und ohne eine einzige hartkodierte Zeichenkette im Quelltext.

Dieser Meilenstein ist **eigener Stand** und kein Teil von M2, weil die Sprachvollständigkeit eine
Eigenschaft des Produkts ist, die man getrennt abnehmen und getrennt halten muss. Ein Release, das
nur halb übersetzt ist, ist schlimmer als ein Release in einer Sprache.

| Punkt | Nachweis | Status |
|---|---|---|
| Die elf gemeinsamen Regeln R-01 bis R-21 | alle Gates | offen — wird am Ende gemessen |
| **Kein hartkodierter Text in der Oberfläche** — jedes sichtbare Wort läuft über einen Schlüssel | Prüfskript über die Oberflächen-Crates | offen |
| Englisch und Deutsch sind vollständig, mit **identischem Schlüsselsatz** | `scripts/check-i18n-parity.py` | offen |
| Die Kette der Sprachsuche: angefordert → Sprachkette → Englisch → **der Schlüssel selbst** | `phocommander-i18n`, Test je Stufe | offen |
| Ein fehlender Schlüssel ergibt **nie** einen leeren Text | Test | offen |
| Die Sprache ist zur Laufzeit umschaltbar, ohne Neustart; die Oberfläche aktualisiert sich sofort | Test | offen |
| Zahlen, Größen, Datums- und Zeitangaben folgen dem System der Sprache | `phocommander-i18n`, `icu` | offen |
| **Eine dritte Sprache kommt ohne Codeänderung aus** — nur ein Katalogverzeichnis | `locales/fr/` | offen |
| Die Oberflächensprache und die Sprache der Dateinamen sind unabhängig | Test | offen |
| Die Sprachdateien sind nach Themen getrennt, damit ein Übersetzer nicht den ganzen Code liest | `locales/<sprache>/*.ftl` | offen |
| Die Benutzerdokumentation existiert in beiden Sprachen | `docs/user-guide/` | offen |
| Die Screenshot- und Textdateien der Stores liegen in beiden Sprachen vor | `docs/guides/` | offen |

**Ergebnis.** Eine Oberfläche, die auf Deutsch und Englisch gleichwertig ist, in der kein Text
hartkodiert ist und in der eine neue Sprache nur eine Datei braucht.


---

## M3 — The other desktop platforms

**Ziel.** Dieselbe Anwendung auf Windows, Linux und BSD, aus derselben Codebasis.

| Punkt | Nachweis | Status |
|---|---|---|
| Die elf gemeinsamen Regeln R-01 bis R-21 | alle Gates | offen |
| Der Windows-Port mit dem Manifest `longPathAware` und der verständlichen Meldung bei Pfadlänge | `apps/`, `docs/` | offen |
| Der Linux-Port mit dem Wayland-Pfad und dem X11-Rückfall | `apps/` | offen |
| Die FreeBSD- und NetBSD-Bauten | CI-Matrix | offen |
| Die Paketierung für die drei Plattformen | `docs/guides/` | offen |
| Die manuelle Testprotokoll für das plattformspezifische Verhalten | `docs/acceptance/` | offen |

---

## M4 — Android

**Ziel.** Eine Android-Version, die in den Store kann.

| Punkt | Nachweis | Status |
|---|---|---|
| Die elf gemeinsamen Regeln R-01 bis R-21 | alle Gates | offen |
| Das Gradle-Projekt und die Rust-`cdylib` | `apps/phocommander-android/` | offen |
| Der SAF-Modus ohne die Spezialberechtigung | `apps/` | offen |
| Der optionale Vollzugriff, im Play Console deklariert und erklärt | `apps/`, `docs/` | offen |
| Der Touch-Kontext mit den gestapelten Panels und 44-Punkt-Zielen | `phocommander-ui` | offen |
| Die Erklärung im ersten Start, warum die Berechtigung gebraucht wird | `apps/` | offen |

---

## M5 — iPadOS

**Ziel.** Eine iPadOS-Version für den App Store.

| Punkt | Nachweis | Status |
|---|---|---|
| Die elf gemeinsamen Regeln R-01 bis R-21 | alle Gates | offen |
| Das Xcode-Projekt und die Rust-`cdylib` | `apps/phocommander-ios/` | offen |
| Der Dokumenten-Picker mit den freigegebenen Wurzeln | `apps/` | offen |
| Die security-scoped Bookmarks, die einen Neustart überstehen | `apps/` | offen |
| Die geteilte Ansicht statt des Fensterpaars | `phocommander-ui` | offen |
| Der Umgang mit einer entzogenen Berechtigung, ohne Absturz | `apps/`, Test | offen |
| Die Einschränkung ist im Dokument ausgewiesen, nicht verschwiegen | `docs/` | offen |

---

## M6 — Remote sources

**Ziel.** Entfernte Quellen verhalten sich wie lokale Verzeichnisse.

| Punkt | Nachweis | Status |
|---|---|---|
| Die elf gemeinsamen Regeln R-01 bis R-21 | alle Gates | offen |
| SFTP, FTP, SMB und WebDAV hinter derselben Schnittstelle | `phocommander-remote` | offen |
| Der selbst geschriebene WebDAV-Client ohne Copyleft-Abhängigkeit | `phocommander-remote` | offen |
| SMB 1 und NTLMv1 werden nicht implementiert | `phocommander-remote`, negativer Test | offen |
| SMB-Signing und SMB-3-Verschlüsselung werden verlangt, wo der Server sie kann | `phocommander-remote`, Integrationstest | offen |
| Jede Netzwerkoperation hat ein Zeitlimit | `phocommander-remote`, Test | offen |
| Die Antwortgröße ist begrenzt, unabhängig von der angekündigten Größe | `phocommander-remote`, Test | offen |
| Die Zugangsdaten liegen im Schlüsselbund | `phocommander-remote` | offen |
| Die Protokolle nennen keine Benutzer-, Server- oder IP-Adresse | `phocommander-remote`, Test | offen |
| Die Synchronisation und die Suche im Inhalt | `phocommander-ops` | offen |

---

## M7 — Cloud and controllers

**Ziel.** Cloud-Zugriff ohne einen Server von uns, und Bedienung mit einem Controller.

| Punkt | Nachweis | Status |
|---|---|---|
| Die elf gemeinsamen Regeln R-01 bis R-21 | alle Gates | offen |
| Die OAuth-2.0-Anmeldung direkt beim Anbieter, ohne Umweg | `phocommander-remote` | offen |
| S3-kompatible Ziele | `phocommander-remote` | offen |
| Kein Server von uns, keine eigene Datenverarbeitung | Architekturprüfung | offen |
| Der `controller`-Kontext auf der Android-Version | `phocommander-ui` | offen |
| Die Controller-Anbindung auf dem Desktop | `phocommander-ui` | offen |

---

## M8 — Television

**Ziel.** Eine Fire-TV- und Google-TV-Version, die die Store-Prüfung besteht.

| Punkt | Nachweis | Status |
|---|---|---|
| Die elf gemeinsamen Regeln R-01 bis R-21 | alle Gates | offen |
| Der `dpad`-Kontext mit der Aktionspalette | `phocommander-ui` | offen |
| Ein Panel sichtbar, das zweite als Overlay | `phocommander-ui` | offen |
| Das 10-Fuß-Layout mit großer Schrift und kräftigem Fokusrahmen | `phocommander-ui` | offen |
| Das Leanback-Manifest mit dem Banner und ohne Touchscreen-Pflicht | `apps/` | offen |
| **Jede Aktion ist im `dpad`-Kontext erreichbar** — geprüft durch das Erreichbarkeitsprotokoll | `docs/acceptance/` | offen |
| Die Zurück-Taste führt zurück und verlässt die Anwendung ohne Schleife | Test | offen |
| Die Anwendung hängt nicht an einer Menü-Taste der Fernbedienung | Test | offen |

---

## M9 — The remaining capabilities

**Ziel.** Die Erweiterungen des Funktionsumfangs.

| Punkt | Nachweis | Status |
|---|---|---|
| Die elf gemeinsamen Regeln R-01 bis R-21 | alle Gates | offen |
| Der Betrachter Stufe 3: Audio, Video, Metadaten, Standbild | `phocommander-viewer` | offen |
| Der Texteditor mit Syntaxhervorhebung | `phocommander-viewer` | offen |
| Die Tags an Dateien und Ordnern | `phocommander-viewer` | offen |
| Die Makros | `phocommander-ops` | offen |
| Alles in beiden Sprachen dokumentiert | `docs/user-guide/` | offen |

### Welcher Meilenstein welches Release auslöst

| Meilenstein abgeschlossen | Release | Plattformen, die mit diesem Release ausgeliefert werden |
|---|---|---|
| M1 | **Alpha 1** (0.1.0-alpha.1) | macOS (Apple Silicon, Intel) |
| M2 | **Alpha 2** (0.1.0-alpha.2) | zusätzlich Linux (x86_64, aarch64) |
| M2b | **Beta 1** (0.2.0-beta.1) | zusätzlich Windows — beide Sprachen vollständig |
| M3 | **Beta 2** (0.2.0-beta.2) | zusätzlich FreeBSD und NetBSD, Windows-Pakete, die portable Variante |
| M4 und M5 | **Beta 3** (0.3.0-beta.3) | zusätzlich Android und iPadOS aus den Stores |
| M6 und M7 | **Beta 4** (0.3.0-beta.4) | zusätzlich die Netzwerkquellen, der Cloud-Zugriff, der Controller-Kontext |
| M8 | **Beta 5** (0.3.0-beta.5) | zusätzlich Fire TV und Google TV |
| M9 | **Release 1.0** (1.0.0) | alle sechs Plattformen, alle obigen Wege |

**Ein Meilenstein löst ein Release aus, aber ein Release ist nicht automatisch fällig.** Es
gilt zusätzlich die Checkliste „Was ein Release auslöst" mit den Bedingungen T-01 bis T-15. Fehlt
eine davon, wird das Release **verschoben**, nicht unvollständig ausgeliefert.

---

## Die Releases

Ein **Release** ist nicht dasselbe wie ein Meilenstein. Der Meilenstein ist die Arbeit, das
Release ist das **ausgelieferte Binary** für bestimmte Plattformen. Beide hängen zusammen, aber
sie sind nicht identisch: Ein Meilenstein kann abgeschlossen sein, ohne dass ein Release
ausgeliefert wird — zum Beispiel weil die Signaturzertifikate noch nicht vorliegen.

Die Versionsfolge folgt der bewährten Regel: `0.x` ist die Arbeit am Produkt, `1.0` ist das
Release, das jemand im Alltag benutzt.

| Release | Version | Auslöser | Plattformen | Was ausgeliefert wird |
|---|---|---|---|---|
| **Alpha 1** | 0.1.0-alpha.1 | Ende von M1 | macOS (Apple Silicon, Intel) | Die erste lauffähige Anwendung: browsen, kopieren, umbenennen, ansehen, Archive öffnen. Ausdrücklich **Alpha**: Fehler sind zu erwarten |
| **Alpha 2** | 0.1.0-alpha.2 | Ende von M2 | macOS, Linux (x86_64, aarch64) | Registerkarten, Profile, sichere Kopiermodi, die Oberfläche in beiden Layouts |
| **Beta 1** | 0.2.0-beta.1 | Ende von M2b | macOS, Linux, Windows | **Beide Sprachen vollständig**, die Oberfläche ist an einem echten Tag benutzbar, keine Datenverluste |
| **Beta 2** | 0.2.0-beta.2 | Ende von M3 | macOS, Linux, Windows, FreeBSD, NetBSD | Alle Desktop-Plattformen, Windows-Pakete, die portable Variante |
| **Beta 3** | 0.3.0-beta.3 | Ende von M4 und M5 | zusätzlich Android und iPadOS | Die mobilen Versionen aus den Stores |
| **Beta 4** | 0.3.0-beta.4 | Ende von M6 und M7 | zusätzlich alle obigen | Die Netzwerkquellen und der Cloud-Zugriff, der Controller-Kontext |
| **Beta 5** | 0.3.0-beta.5 | Ende von M8 | zusätzlich Fire TV und Google TV | Die Fernseher-Version |
| **Release 1.0** | 1.0.0 | Ende von M9 | alle sechs Plattformen | Das erste Release, das jemand im Alltag benutzt. Kein Alpha, kein Beta |

### Was ein Release auslöst

Ein Release wird **nicht** durch das Ende eines Meilensteins ausgelöst, sondern durch eine
Checkliste. Sie ist die Bedingung, und sie wird bei jedem Release vollständig abgearbeitet.

| Nr. | Bedingung | Nachweis |
|---|---|---|
| T-01 | Der zugehörige Meilenstein ist abgenommen | `docs/acceptance/` |
| T-02 | Die elf gemeinsamen Regeln R-01 bis R-21 sind erfüllt | alle Gates |
| T-03 | Alle Tests laufen durch, keiner wird übersprungen | `cargo test --workspace --locked` |
| T-04 | **Die Version ist in genau einer Stelle geschrieben** und überall identisch | `Cargo.toml`, `CHANGELOG.md`, alle Shells |
| T-05 | Die Version folgt der Regel `MAJOR.MINOR.PATCH` mit `0.x` vor `1.0` | `Cargo.toml` |
| T-06 | Die Prüfsummen der Artefakte sind berechnet und im Release hinterlegt | `SHA256SUMS` |
| T-07 | Jedes Artefakt ist **signiert** | Signatur des Herstellers |
| T-08 | Bei macOS ist das Artefakt **notarisiert** und das Ticket angeheftet | `notarytool`, Ticket |
| T-09 | Bei Windows ist das Artefakt **Authenticode-signiert** | Signatur des Herstellers |
| T-10 | Bei den Stores ist die Einreichung angenommen, die Prüfung bestanden | Bestätigung des Stores |
| T-11 | Die Release-Notizen nennen die Änderungen, die Bekannten Einschränkungen und die Auslöser | `CHANGELOG.md` |
| T-12 | Die Dokumentation beschreibt den Stand dieses Releases | `docs/user-guide/` |
| T-13 | Die Sicherheitsregeln sind erfüllt, inklusive der drei nicht schaltbaren Garantien | `docs/security/` |
| T-14 | Die Binaries sind auf der Zielplattform **gebaut**, nicht nur auf einer | Artefakte des Release |
| T-15 | Die Artefakte tragen **keine** Telemetrie, **keine** Werbung und **keinen** Account-Zwang | Prüfskript plus Sichtprüfung |

### Die Plattformen und ihre Wege

Jede Plattform hat ihren eigenen Weg in die Hand des Nutzers. Das bestimmt, was vor dem Release
fertig sein muss.

| Plattform | Weg | Was vorher fertig sein muss | Signatur |
|---|---|---|---|
| **macOS** | Über die eigene Website, als `.dmg` | Notarisierung mit dem Ticket, Hardened Runtime, Developer-ID-Signatur | Apple Developer ID, notarisiert |
| **Windows** | Über die eigene Website, als `.msix` und als portable Variante | Authenticode-Signatur; die portable Variante ist **ein** Executable ohne Installer | Eigenes Code-Signing-Zertifikat |
| **Linux** | Über die eigene Website, als Flatpak, AppImage und als Paket der Distributionen | Keine Signatur nötig, aber reproduzierbare Builds | Keine, Integrity über Prüfsummen |
| **FreeBSD, NetBSD** | Über die eigene Website, als Paket | Ports-Paket, sofern ein Port existiert | Keine |
| **Android** | Über den Play Store | Die Deklaration der Spezialberechtigung, die Datenschutzerklärung, die Altersfreigabe | Play App Signing |
| **iPadOS** | Über den App Store | Datenschutzerklärung, Altersfreigabe, die Screenshots in beiden Sprachen | Apple Developer |
| **Meta Quest** | Über den Meta Horizon Store, oder als APK von der Website | Die App laeuft als flaches Panel, kein immersiver VR-Port | Meta Store |
| **Fire TV, Google TV** | Über den Play Store | Das Leanback-Manifest, die Erreichbarkeit mit dem D-Pad, das Banner | Play App Signing |

### Was ein Release **nicht** ist

- **Kein Alpha ohne Abnahme.** Eine Alpha ohne die Abnahme des Meilensteins ist eine
  unkontrollierte Auslieferung. Der Auslöser ist der Meilenstein, nicht das Datum.
- **Kein Release ohne Signatur.** Ein unsigniertes Binary erzeugt auf macOS die Gatekeeper-Warnung
  und auf Windows die SmartScreen-Warnung. Beide sind für ein Werkzeug, das der Nutzer auf sein
  Firmennotebook installiert, der Moment, in dem das Vertrauen verloren geht.
- **Kein Release mit offenen Sicherheitsbefunden.** Ein offener Befund aus dem Bedrohungsmodell
  blockiert das Release.
- **Kein Release, das mehr verspricht als die Software kann.** Der Umfang eines Releases steht im
  Changelog, und der Changelog nennt die bekannten Einschränkungen.

### Die Version im Alltag

Die Anwendungen zeigen ihre Version im About-Dialog an: Anwendung, Version, Commit und Bauplattform.
Der Konfigurationsdatei trägt ihre eigene `schema_version` — das ist die Version des **Formats**
der Datei, nicht die Version der Anwendung. Die beiden werden getrennt, weil sie sich zu
verschiedenen Zeiten ändern.


---

## Die Reihenfolge und ihre Gründe

Die Meilensteine sind linear, weil jeder auf dem vorigen aufsetzt. Zwei Entscheidungen sind
bewusst so getroffen:

**Der Kern kommt vor der Oberfläche.** Jeder Meilenstein liefert zuerst eine testbare Schicht im
Rust-Kern. Die Oberfläche bindet sich daran. Eine Oberflächenfunktion ohne Kern-Unterstützung wird
nicht begonnen.

**Die netzwerkreichen Meilensteine kommen nach den lokalen.** Eine Quelle, die nicht funktioniert,
ist reparierbar und sichtbar. Eine halbfertige Netzwerkschicht, die in M1 dazwischenhängt,
gefährdet den Kern.

**Die Internationalisierung ist ein eigener Meilenstein.** Sie steht zwischen M2 und M3, weil die
Sprachvollständigkeit eine Eigenschaft des Produkts ist, die man getrennt abnehmen muss. Vor M2b
darf die Oberfläche nur in einer Sprache existieren — für die Entwicklung ist das richtig, und für
ein Release ist es falsch. Der Beta-1-Punkt der Roadmap setzt deshalb die Vollständigkeit beider
Sprachen voraus, und ein Release, das nur halb übersetzt ist, ist keines.

## Die Offenen Punkte, die eine Entscheidung brauchen

Diese Punkte sind in der Spezifikation unter „Offene Punkte für die Umsetzung" festgehalten. Sie
blockieren den Start nicht, brauchen aber eine Antwort, bevor die betroffene Zeile Code entsteht.

| Frage | Vorschlag | Betrifft |
|---|---|---|
| Bezugsgröße der Serienfolgen-Erkennung | Der Verzeichnisname ist die Vorgabe, er ist immer sichtbar und überschreibbar | M1, M2 |
| Umfang der Aktionspalette | Nur die im Kontext erreichbaren Aktionen zeigen | M8 |
| Grenze der Ergebnisgröße bei Netzwerkquellen | Festes Lesebudget, der Wert wird in M6 gemessen | M6 |
| Platzhalter-Dialekt im Umbenennungsdialog | Eckige Klammern für den Dialog, geschweifte für die Vorlagen | M1, M2 |
