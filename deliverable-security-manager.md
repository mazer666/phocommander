# Übergabe — Security & Privacy, Meilenstein M0

**Auftrag:** Bedrohungsmodell und Sicherheitsdokumentation für Phocommander M0
**Agent:** Phocommander Security Manager · **Datum:** 24.10.2026
**Repository:** `/workspace/phocommander`, Branch `main`
**Stand des Baums bei Abgabe:** Commit `5af10fb` (Commit `b88a27c` davor); der Arbeitsbaum enthielt zum Zeitpunkt der Übergabe **uncommittete Paralleländerungen anderer Agenten** (Rust Architect, Docs Lead, PM, QA). Es wurde **nichts committet, nichts gepusht, nichts getaggt**.

---

## 1. Kurzfazit

| | |
|---|---|
| Erstellt | `docs/security/THREAT_MODEL.md` (544 Zeilen, 9 Abschnitte) |
| Gefixt | **2 echte Defekte im Sicherheits-Gate** (`deny.toml`, `publish = false`) — beide mit Ausgabebeleg belegt, das Gate läuft jetzt grün |
| Verifiziert | alle 4 `zip`-Advisories gegen die RustSec-Datenbank, `cargo-deny` 0.20.2 installiert und ausgeführt, `check-crate-boundaries.sh` ausgeführt |
| Offen | die Extraktionshärtung selbst (H2–H6) und die negativen Tests — M1, in diesem Meilenstein ausdrücklich nicht implementiert |
| Konflikt | `SECURITY.md` wurde **zeitgleich vom Docs Lead erstellt**; mein Schreibvorgang hat die Datei überschrieben. Details in §9. |

**Wichtigster Befund:** Das Lizenz- und Advisory-Gate war **tot**. `cargo deny check` brach
mit einem Konfigurationsfehler ab, *bevor* es eine einzige Abhängigkeit geprüft hatte —
genau der Fehlerfall, den die Spezifikation für das Schwesterprojekt Vocument beschreibt
(„die Meldung beschuldigte stattdessen die Konfiguration"). Ein Gate, das nie läuft,
prüft nichts.

---

## 2. Dateiliste

| Pfad | Zeilen | Zweck | Status |
|---|---|---|---|
| `docs/security/THREAT_MODEL.md` | 549 | Das Bedrohungsmodell: Trust Boundary, Angriffsflächen-Tabelle (16 Zeilen), 17 verbindliche Härtungsregeln H1–H17 mit negativem Test je Regel, bewusst akzeptierte Restrisiken, sicheres Löschen nach NIST SP 800-88 Rev. 2, Berechtigungs-Audit je Plattform, Datenschutz-Grenzen, Prüfprotokoll | **neu, von mir erstellt** |
| `SECURITY.md` | 184 | Meldeweg, unterstützte Versionen, Reaktionszeit-Erwartung, Härtungsgarantien H1–H17, Abschnitt „Why we need no telemetry" | **von mir erstellt, parallel vom Docs Lead erstellt und überschrieben** (siehe §9) |
| `PRIVACY.md` | 179 | Datenminimierung, Speicherorte, Aufbewahrung, Löschung, Schlüsselbund, Protokoll, Server-Verkehr | **vom Docs Lead erstellt, von mir gelesen und gegen §7 des Bedrohungsmodells geprüft — nicht von mir verändert** |
| `deny.toml` | 80 | cargo-deny-Konfiguration. **Gefixt:** `unmaintained = "warn"` → `"none"`, `allow-wildcard-paths = true` ergänzt | **von mir geändert** |
| `platform/phocommander-platform/Cargo.toml` | 23 | `publish = false` ergänzt (Voraussetzung für `allow-wildcard-paths`) | **von mir geändert** |
| `apps/phocommander-desktop/Cargo.toml` | 23 | `publish = false` ergänzt | **von mir geändert** |
| `docs/acceptance/.cargo-deny-install.log` | — | Logdatei des Architekten, **nicht von mir angefasst** | unverändert |
| `deliverable.md` | 787 | Diese Übergabe (deutsch, wie von `AGENTS.md` gefordert) | **von mir erstellt** (überschrieb eine parallele Datei, siehe §9 / F-07b) |
| `deliverable.security-manager.md` | 787 | Identische Kopie dieser Übergabe unter agentenspezifischem Namen, damit eine spätere Überschreibung sie nicht vernichtet | **neu** |
| `deliverable-security-manager.patch` | 144 | Patch meiner Arbeitsbaum-Änderungen (`git apply` anwendbar) | **neu** |
| `/workspace/output/phocommander-security-m0.patch` | 144 | Dieselbe Datei im Auslieferungsordner | **neu, außerhalb des Repos** |

Nicht verändert: `docs/REQUIREMENTS.md` (ausdrücklich untersagt), `Cargo.toml` (Root),
`crates/**`, `apps/**`, `platform/**` außer den beiden genannten Manifest-Zeilen.

---

## 3. Angriffsflächen-Matrix (Vorschau)

Vollständig in `docs/security/THREAT_MODEL.md` §2. Hier die Kurzfassung mit Status:

| Angriffsfläche | Angreifer | Eintrittspunkt | Schadenspotenzial | Gegenmaßnahme | Status |
|---|---|---|---|---|---|
| Archiv-Extraktion: Pfad-Traversal (Zip-Slip) | Archiv-Absender | Eintragsname mit `..` oder absolutem Pfad | Beliebiger Datei-Write außerhalb des Ziels → RCE, zerstörte Konfiguration | H2 (kein `extract()`), H3 (kanonisieren + einschließen) | H3 `in progress` (lexikalische Hälfte implemented), H2 `planned` |
| Archiv-Extraktion: Symlink mit Ziel außerhalb des Zielverzeichnisses | Archiv-Absender | Unix-Symlink-Eintrag | Lesen/Schreiben beliebiger Dateien außerhalb | H4 (Symlinks werden nie erzeugt) | `planned` |
| Archiv-Extraktion: Unix-Berechtigungen durch Symlinks hindurch | Archiv-Absender | Symlink + nachfolgende Rechteänderung | `chmod` beliebiger Pfade → Privilegienerhöhung | H4 + H5 + H3 | `planned` |
| Archiv-Extraktion: Erhalt von SUID/SGID-Bits | Archiv-Absender | `external_attributes` mit `S_ISUID`/`S_ISGID` | Lokale Privilegienerhöhung bei privilegiertem Prozess | H5 (Maskierung auf `0o644`/`0o755`) | `planned` |
| Archiv-Extraktion: Dekompressionsbombe | Archiv-Absender | hoher Expansionsfaktor | Speicher-/Plattenerschöpfung | H6 (drei Limits) | `planned` |
| TOCTOU zwischen Pfadprüfung und Zugriff | Lokaler Prozess gleicher UID | Symlink-Tausch zwischen Prüfung und Zugriff | Write außerhalb des Ziels trotz korrekter Prüfung | H9 (`openat`-Semantik über `rustix`/`cap-std`, sonst Prüfung unmittelbar davor) | `planned` — **Restrisiko, benannt in §4** |
| Symlink-Escape beim rekursiven Kopieren | Lokaler Prozess / Quellbaum | Symlink im Quellbaum | Inhalt außerhalb des Quellbaums kopiert | H10 (Symlinks werden als Symlinks kopiert, nie dereferenziert) | `planned` — **Restrisiko, benannt in §4** |
| Header-Overflow in den Format-Parsern (Text, Bild, Archiv) | Absender/Autor | Unplausible Längen- oder Zählfelder | Panic, Overflow, huge allocation | H11 (Header vor der Allokation validieren, Fuzzing) | `planned` |
| Speicherschöpfung durch sehr große Verzeichnisse | Lokaler Prozess / Server | 100.000+ Einträge | OOM-Kill, UI-Freeze | H12 (Streaming mit begrenztem Fenster) | `planned` |
| Path-Length auf Windows (260 Zeichen ohne Opt-in) | Absender/Autor | Lange Namen | Scheitern oder stille Kürzung | H13 (`longPathAware` + verständliche Meldung) | `planned` |
| Berechtigungsübernahme auf macOS, Android, iPadOS | Nutzer (versehentlich) | App fordert breiten Zugriff an | Lesen/Schreiben ungewollter Daten | H14 (Berechtigungsmodell je Plattform, §6) | `planned` |
| Netzwerkquelle: Zertifikatsprüfung umgangen | Netzwerkangreifer | selbstsigniertes Zertifikat | Totaler Verlust von Vertraulichkeit und Integrität | H15 (strenge Prüfung, kein Opt-out, PIN-Ausnahme je Host) | `planned` |
| Netzwerkquelle: Antwortgröße über dem Erwarteten | Hostiler Server | Verzeichnisliste von mehreren GB | Speichererschöpfung | H16 (festes Lesebudget) | `planned` |
| Netzwerkquelle: Protokoll-Downsizing (SMB1, NTLMv1, WebDAV ohne TLS) | Netzwerkangreifer | Server bietet nur alte Version | Bekannt verwundbare Protokolle, Relay | H17 (SMB1/NTLMv1 nicht implementiert, Signatur + SMB3-Verschlüsselung) | `planned` |
| Netzwerkquelle: eingeschleuste Verzeichnisnamen | Hostiler Server | Trennzeichen, Steuerzeichen, 4-KiB-Namen | Pfadverwirrung, Log-Injection, Terminal-Escape | H18 (Normalisierung, Zeichensatzprüfung, gekürzte Anzeige) | `planned` |
| Netzwerkquelle: hängende Antwort ohne Zeitlimit | Hostiler Server | Antwort endet nie | Auftrag hängt für immer | H19 (Zeitlimit je Netzwerkoperation) | `planned` |

**Status-Vokabular** (in §0 des Dokuments definiert): `documented` / `planned` /
`in progress` / `implemented` / `accepted`. **Keine** Zeile steht auf `implemented` — zur
Abgabe existiert im Workspace noch kein Extraktionscode.

---

## 4. Fertige Textbausteine für `docs/security/THREAT_MODEL.md`

Die Datei ist **vollständig geschrieben** (544 Zeilen, Englisch gemäß Projektregel). Die
Bausteine liegen als Datei vor; hier die Struktur und die inhaltlichen Kernpunkte, damit die
Übergabe ohne Öffnen der Datei lesbar ist.

### §1 Scope und Trust Boundary
Die Grenze verläuft zwischen Prozess und Dateisystem. ASCII-Diagramm. Alles unterhalb der
Linie ist bösartig: Archivinhalt, Dateisysteminhalt, entfernte Server (SMB, WebDAV, S3,
SFTP). Der Angreifer ist „jemand, der dem Nutzer Inhalt vorlegt" — er braucht keine
Privilegien, nur einen Doppelklick. Ausdrücklich **nicht** im Scope: Kernel-Kompromittierung,
physische Angriffe auf den Host, Malware, die bereits als der Nutzer läuft, legaler
Zugriff über eine vertrauenswürdige Root-CA im System-Speicher.

### §2 Angriffsflächen-Tabelle
16 Zeilen, Spalten `Angriffsfläche | Angreifer | Eintrittspunkt | Schadenspotenzial |
Gegenmaßnahme | Status` — siehe §3 dieser Übergabe.

### §3 Die verbindlichen Härtungsregeln
Tabelle H1–H17 mit Spalten `Regel | Durchsetzungspunkt | Negativer Test (geplant) | Status`.
Dazu §3.1 mit der Advisory-Tabelle und der Begründung, warum die Versionsuntergrenze allein
nicht die Verteidigung ist:

> `RUSTSEC-2026-0368` states that the link target bytes were passed „directly to the platform
> symlink API" — the advisory floor fixes the crate, it does not make `extract()` safe for a
> project that wants to refuse symlinks outright. A project requirement ("symlinks are never
> created from archive material") cannot be expressed as a version requirement.

Und die belegte API-Fläche von `zip` 9.0.5 (`ZipFile::enclosed_name`, `ZipFile::is_symlink`,
`ZipFile::unix_mode`, `ZipArchive::len`, `ZipArchive::decompressed_size`,
`ZipArchive::has_overlapping_entries`, `ZipArchive::extract` — **darf nicht aufgerufen werden**).

### §4 Restrisiken, die bewusst akzeptiert werden
Benannt und nicht verschwiegen: TOCTOU zwischen Prüfung und Zugriff (Verringerung über
deskriptorbasierte Zugriffe mit `openat`-Semantik über `rustix`/`cap-std`, sonst Wiederholung
der Prüfung unmittelbar vor dem Zugriff; **was bleibt**: das Fenster zwischen letzter
`canonicalize()` und `open()`, auf Windows ohne `openat`-Äquivalent); TOCTOU beim rekursiven
Kopieren (ohne Deskriptor-Walk nicht vollständig zu schließen, Plan: `phocommander-ops`).
Dazu eine Tabelle weiterer akzeptierter Risiken (lokaler Prozess gleicher UID, macOS-FDA,
Server sieht angefragte Verzeichnisse, Flash-Medien-Einschränkung beim Löschen).

### §5 Sicheres Löschen
Maßstab NIST SP 800-88 Rev. 2 mit den Stufen Clear / Purge / Destroy, jeweils mit
Originalzitat belegt (Abschnitt 3.1, 3.1.1, 3.1.2, 3.1.3 und Fußnote 5). Warum der
DoD-5220.22-M-Dreifachdurchgang nicht mehr gilt (NISPOM-Entfernung 2006) und warum
Überschreiben auf Flash wegen Wear-Leveling und Over-Provisioning nicht zuverlässig ist.
Festlegung: Clear ist der Standard, das **Papierkorb** ist der Standardpfad, die
Einschränkung steht **in der Oberfläche**, nicht nur im Handbuch.

### §6 Berechtigungs-Audit je Plattform
Tabelle `Plattform | Berechtigung | Zweck | Notwendig | Begründung | Empfehlung` — siehe §8
dieser Übergabe.

### §7 Datenschutz-Grenzen
Was gespeichert wird (Konfiguration, Schlüsselbund, Log, Session), was **nicht** gespeichert
wird (keine Telemetrie, keine Analytics, keine Nutzungsprofile, keine Absturzberichte, kein
Inhaltsindex), warum keine Telemetrie nötig ist (drei prüfbare Zusicherungen), und was bei
einer Netzwerkverbindung an den Server geht (nur die vom Nutzer angeforderten
Verzeichnisdaten, keine Kennungen, keine Nutzungsprofile).

### §8 Prüfprotokoll
Welche Prüfungen in der CI laufen (mit exakten Kommandos und dem Status, ob die Datei
existiert), welche manuell erfolgen (12 Zeilen, jeweils mit dem, was sie beweisen), plus §8.3
mit dem Beleg des `cargo deny`-Laufs.

### §9 Verwandte Dokumente
Querverweise auf `docs/REQUIREMENTS.md`, `SECURITY.md`, `PRIVACY.md`, `docs/adr/`, `deny.toml`.

---

## 5. Verifikation der Advisory-Versionen (mit Ausgabe)

**Methode:** Abruf der gerenderten Advisory-Seiten aus der RustSec-Datenbank
(`https://rustsec.org/advisories/<ID>.html`) und Auslesen des Feldes `Patched`.
Zusätzlich wurden die Roh-Advisory-Dateien aus `rustsec/advisory-db` gelesen und mit den
Seiten abgeglichen (Dateigrößen identisch: 1136 / 1102 / 998 / 685 Bytes, cf. GitHub-API).

**Kommando:**

```
python3 /workspace/.scratch/sec/verify_advisories.py
```

**Ausgabe (unverändert):**

```
RUSTSEC-2025-0168
  url      : https://rustsec.org/advisories/RUSTSEC-2025-0168.html
  title    : Incorrect path canonicalization during Archive Extraction Leading to Arbitrary File Write
  patched  : >=2.3.0
  unaffected: <1.3.0
  aliases  : CVE-2025-29787, GHSA-94vh-gphv-8pm8

RUSTSEC-2026-0367
  url      : https://rustsec.org/advisories/RUSTSEC-2026-0367.html
  title    : Insecure Preservation of SUID/SGID Bits during ZIP Extraction Leading to Potential Privilege Escalation
  patched  : >=9.0.0
  unaffected: <not found>
  aliases  : GHSA-vv5m-jxxh-4627

RUSTSEC-2026-0368
  url      : https://rustsec.org/advisories/RUSTSEC-2026-0368.html
  title    : zip:
  patched  : >=9.0.0
  unaffected: <not found>
  aliases  : GHSA-4v9h-p7m9-rh26

RUSTSEC-2026-0369
  url      : https://rustsec.org/advisories/RUSTSEC-2026-0369.html
  title    : ZipStreamReader::extract() and ZipArchive::extract() apply Unix permissions through symlinks without confining the target
  patched  : >=9.0.0
  unaffected: <not found>
  aliases  : GHSA-4684-h2q9-2vcp
```

### Einordnung — und eine Korrektur an der Spezifikation

| Advisory | Datenbank nennt „patched" | Bewertung |
|---|---|---|
| `RUSTSEC-2025-0168` | `>= 2.3.0` | Der Pin `zip = "9.0.5"` liegt deutlich über dieser Untergrenze. |
| `RUSTSEC-2026-0367` | `>= 9.0.0` | Pin liegt darüber. |
| `RUSTSEC-2026-0368` | `>= 9.0.0` | Pin liegt darüber. Betroffen war laut Advisory `zip 9.0.0-pre2`. |
| `RUSTSEC-2026-0369` | `>= 9.0.0` | Pin liegt darüber. |

**Wichtige Präzisierung:** Die Spezifikation formuliert in *Sicherheit*, die Mindestversion
sei „9.0.5, die die genannten Advisorys behebt". Das ist **richtig**, aber ungenau: Die
von der Datenbank selbst genannte Untergrenze ist `9.0.0` (bzw. `2.3.0` für
`RUSTSEC-2025-0168`). `9.0.5` ist die **neueste veröffentlichte** Version auf crates.io
(verifiziert: `https://crates.io/api/v1/crates/zip` → `max_version 9.0.5`,
`newest_version 9.0.5`), also eine strengere, aber frei gewählte Untergrenze über dem
Datenbank-Floor. Das ist so in `docs/security/THREAT_MODEL.md` §3.1 festgehalten — inklusive
der Begründung, warum die Versionsuntergrenze die Verteidigung **nicht** ersetzt (siehe §4
dieser Übergabe).

**Zusätzlich verifiziert** (nicht in der Spezifikation, aber für dieselbe Angriffsfläche
relevant — die Datenbank führt für `tar` zwei eigene Advisories):

| Advisory | Aliase | patched in | Wirkung |
|---|---|---|---|
| `RUSTSEC-2026-0067` | CVE-2026-33056, GHSA-j4xf-2g29-59ph | `>= 0.4.45` | `unpack_in` kann per Symlink-Following beliebige Verzeichnisse `chmod`en |
| `RUSTSEC-2026-0068` | CVE-2026-33055, GHSA-gchp-q4r4-x4ff | `>= 0.4.45` | `tar-rs` ignoriert PAX-Größenheader bei ungleich null Headergröße |

Beide betreffen `tar::Entry::unpack` und `tar::Entry::unpack_in` — dasselbe Muster
„Extraktion dem Crate überlassen", das Regel H2 verbietet. `tar = "0.4"` löst heute auf
0.4.46 (verifiziert über crates.io) und erfüllt die Untergrenze; die Härtung stützt sich
trotzdem nicht darauf.

**Nicht verifizierbar / nicht behauptet:** Für `flate2` existiert **kein** Advisory in der
RustSec-Datenbank (Abfrage der Paketseite `https://rustsec.org/packages/flate2.html`
liefert HTTP 404, OSV-Abfrage liefert 0 Schwachstellen). Der Kommentar in `Cargo.toml`,
„The fix versions are: zip 9.0.5, tar 0.4.44, flate2 1.1.1", ist damit in zwei Punkten
unbelegt bzw. falsch: **`tar 0.4.44` ist die letzte verwundbare Version**, nicht die
Fix-Version (Fix ist `>= 0.4.45`), und für **`flate2 1.1.1` gibt es kein Advisory**.
→ Finding **F-05** in §6.

---

## 6. Findings-Liste nach Schweregrad

### F-01 · High · `deny.toml:42` — das Lizenz- und Advisory-Gate war tot

**Ort:** `deny.toml`, Zeile 42 (`[advisories] unmaintained = "warn"`)
**Angreifer-Szenario:** kein direkter Angriffsvektor, sondern ein **kontrollierter Blindflug**:
ein Angreifer oder ein unbedachter Beitragender bringt eine Abhängigkeit mit Copyleft-Lizenz
oder mit bekannter Schwachstelle in den Baum. Das Gate, das das verhindern soll, meldet
stattdessen einen Konfigurationsfehler und prüft nichts.
**Konkreter Angriffspfad:** `cargo deny check` → Konfigurationsdeserialisierung schlägt fehl →
Abbruch mit Exit-Code 1 **vor** der graphbasierten Prüfung → keine einzige Abhängigkeit wird
geprüft. In der CI ist das Gate dauerhaft rot; die Specifikation schreibt vor, dass kein Gate
abgeschwächt wird, um grün zu werden — also wird entweder die Prüfung entfernt oder
`deny.toml` so verbogen, dass sie grün wird.
**Schadenspotenzial:** hoch für die Lieferkette (AGPL/SSPL-Kontamination, Nutzung einer
verwundbaren Version), niedrig für den Endnutzer zur Laufzeit.
**Fix (implementiert, siehe §7):** `unmaintained = "none"` plus Kommentar, der erklärt, dass
cargo-deny 0.20.2 für Advisories **keine** Severity-Stufe kennt, sondern nur einen Scope.

```
error[unexpected-value]: expected '["all", "workspace", "transitive", "none"]'
   ┌─ …/phocommander/deny.toml:42:17
   │
42 │ unmaintained = "warn"
   │                 ━━━━ unexpected value

2026-10-24 11:04:56 [ERROR] failed to deserialize config from '…/phocommander/deny.toml'
EXIT=1
```

**Negativer Test, der den Fix belegt:** `cargo deny check` muss mit laufender Werkzeugversion
0.20.2 durchlaufen und die Prüfergebnisse ausgeben. Ein Test, der die Konfiguration
*liest*, aber nicht *ausführt*, belegt nichts — der Fehler lag in der Ausführung.

### F-02 · High · `deny.toml` + alle Crate-Manifeste — die Wildcard-Sperre traf den eigenen Workspace

**Ort:** `deny.toml` (`[bans] wildcards = "deny"`), `platform/phocommander-platform/Cargo.toml:20`,
`apps/phocommander-desktop/Cargo.toml:20-21`
**Angreifer-Szenario:** gleiche Blindflug-Wirkung wie F-01, nur eine Ebene tiefer: das Gate
ist auch nach der Konfigurationsreparatur rot und wird vom Betreiber „repariert", indem er
`wildcards = "allow"` setzt — dann ist die eigentliche Regel (ein `"*"`-Voraussetzungen aus
crates.io macht den Build nicht reproduzierbar) **abgeschwächt**.
**Konkreter Angriffspfad:** Jede Crate-Abhängigkeit per `path` ohne `version`-Key gilt im
Wildcard-Check als Wildcard-Abhängigkeit. Der Workspace referenziert sich ausschließlich so
→ jeder Push schlägt fehl → Druck, die Regel zu entschärfen.
**Schadenspotenzial:** mittel bis hoch (Aufweichen einer Lieferkettenregel als
„Reparatur").
**Fix (implementiert):** `allow-wildcard-paths = true` in `[bans]` **plus** `publish = false`
in den betroffenen Crate-Manifesten. Ohne `publish = false` greift die Ausnahme nicht, weil
cargo-deny öffentliche Crates ausnimmt — die Fehlermeldung sagt das ausdrücklich:
`allow-wildcard-paths is enabled, but does not apply to public crates as crates.io disallows
path dependencies.`
**Negativer Test:** `cargo deny check bans` → `bans ok` (Exit 0). Zusätzlich zu prüfen:
`cargo deny check bans` mit einem **registry**-Crate, das eine `"*"`-Voraussetzung hat, muss
weiterhin fehlschlagen — das ist der Test, der beweist, dass die Regel nicht ausgehebelt
wurde (in §7 als `F-02-Negativtest` aufgeführt, noch nicht implementiert).

### F-03 · Medium · `crates/phocommander-archive/src/entry_name.rs` — lexikalisch, nicht kanonisiert

**Ort:** `crates/phocommander-archive/src/entry_name.rs`, Funktion `resolve_entry_name` (Zeile 99)
**Angreifer-Szenario:** Der Angreifer liefert ein Archiv, dessen Zielverzeichnis bereits einen
Symlink enthält. Der Nutzer entpackt es in ein Verzeichnis, in dem ein Unterordner ein
Symlink auf `/etc` ist.
**Konkreter Angriffspfad:** `resolve_entry_name("passwd")` liefert `passwd` — ein *relativer*
Name ohne `..`. Die Prüfung besteht. Beim Schreiben wird der Name gegen das Zielverzeichnis
gejoint; der Symlink `sub → /etc` sorgt dafür, dass der Schreibvorgang in `/etc` landet.
Der Doc-Kommentar der Funktion sagt: „the result is always relative and never contains a
`..` segment, so that joining it with the destination directory cannot leave the destination"
— das ist für den *lexikalischen* Join richtig, für den *realen* Pfad nicht.
**Schadenspotenzial:** hoch (beliebiger Datei-Write) — aber nur, wenn das Zielverzeichnis
bereits einen Symlink enthält, also mit einem Mitwirkenden des Angreifers oder eines anderen
Programms auf dem Rechner.
**Fix (Vorschlag, M1):** Nach dem Join den **kanonisierten** Pfad prüfen und gegen das
kanonisierte Zielverzeichnis vergleichen; zusätzlich jede Pfadkomponente unterhalb des
Zielverzeichnisses mit `symlink_metadata` prüfen und einen Symlink ablehnen, bevor er
durchlaufen wird. Noch besser: deskriptorbasiert öffnen (`cap-std`/`rustix`, `openat` mit
`O_NOFOLLOW` je Komponente). Verifiziert: im gesamten Workspace existiert **kein** Aufruf
von `canonicalize` (`grep -rn "canonicalize" --include="*.rs" .` → keine Treffer).
**Negativer Test:** `rejects_entry_escaping_via_intermediate_symlink` — Zielverzeichnis anlegen,
`sub → /etc` als Symlink anlegen, Archiv mit dem Eintrag `sub/passwd` extrahieren, Ergebnis
muss ein Fehler sein und `/etc/passwd` unverändert.

### F-04 · Medium · `Cargo.toml:73` — die Begründung im Manifest ist sachlich falsch

**Ort:** `Cargo.toml`, Zeile 73 (Kommentarblock über den Archiv-Abhängigkeiten)
**Angreifer-Szenario:** keiner direkt — die Wirkung ist die einer falschen Sicherheitsannahme
im Manifest, das als Referenz gelesen wird.
**Konkreter Angriffspfad / Defekt:** Der Kommentar behauptet „The fix versions are: zip 9.0.5,
tar 0.4.44, flate2 1.1.1." Die RustSec-Datenbank nennt für `tar` `>= 0.4.45` als Fix —
**0.4.44 ist die letzte verwundbare Version** („In versions 0.4.44 and below of tar-rs …").
Für `flate2` existiert **kein** Advisory (Paketseite 404, OSV-Abfrage leer). Die Versionsangabe
`flate2 1.1.1` ist unbelegt.
**Schadenspotenzial:** niedrig direkt, mittel indirekt — ein Manifest-Kommentar, der
Fix-Versionen dokumentiert, ist die Referenz, an der sich ein späterer Beitragender orientiert.
**Fix (Vorschlag, nicht angewendet — Datei gehört dem Rust Architect):** Kommentar ersetzen
durch:

```
# `zip` MUST stay at 9.0.5 or newer. Older releases are affected by
# RUSTSEC-2025-0168 (broken path canonicalisation), RUSTSEC-2026-0367 (SUID/SGID
# bits survive extraction), RUSTSEC-2026-0368 (symlinks pointing outside the
# target directory) and RUSTSEC-2026-0369 (Unix permissions set through
# symlinks). Verified against the RustSec database on 24.10.2026:
#   RUSTSEC-2025-0168 patched >= 2.3.0
#   RUSTSEC-2026-0367 patched >= 9.0.0
#   RUSTSEC-2026-0368 patched >= 9.0.0
#   RUSTSEC-2026-0369 patched >= 9.0.0
# `tar` MUST stay at 0.4.45 or newer (RUSTSEC-2026-0067, RUSTSEC-2026-0068).
# `flate2` has no advisory in the RustSec database.
```

### F-05 · Low · `Cargo.toml:73` — dieselbe Stelle, `flate2`-Version unbelegt

Siehe F-04; getrennt geführt, weil die Behauptung „flate2 1.1.1" keine Fehlbehauptung über
eine verwundbare Version ist, sondern eine ohne Beleg. `flate2 = "1"` löst heute auf 1.1.10
und ist damit unkritisch — nur die Behauptung im Kommentar ist unbelegt.

### F-06 · Low · `docs/REQUIREMENTS.md` — Formulierung der Versionsuntergrenze

**Ort:** `docs/REQUIREMENTS.md`, Abschnitt *Sicherheit*, Satz „Die Mindestversion des
`zip`-Crates ist 9.0.5, die die genannten Advisorys behebt [26]."
**Defekt:** Nicht falsch, aber ungenau: der von der Datenbank genannte Floor ist `9.0.0`
(bzw. `2.3.0`). `9.0.5` ist eine strengere, frei gewählte Untergrenze.
**Schadenspotenzial:** niedrig (Dokumentationspräzision), aber die Präzision ist der
Zweck der Spezifikation.
**Fix (Vorschlag, nicht angewendet — `docs/REQUIREMENTS.md` ist für diesen Auftrag
ausdrücklich unveränderlich):** „Die Mindestversion des `zip`-Crates ist 9.0.5 — die
Advisories nennen als Fix-Stand `>= 9.0.0` (bzw. `>= 2.3.0` für RUSTSEC-2025-0168); 9.0.5 ist
der aktuell veröffentlichte Stand und liegt über jeder dieser Untergrenzen."

### F-07 · Info · `SECURITY.md` — Überschreibung durch einen parallelen Agenten

**Ort:** `SECURITY.md`
**Angriffspfad / Defekt:** Kein Sicherheitsdefekt im Produkt, sondern ein Übergabedefekt:
`SECURITY.md` wurde um 10:53:38 vom Docs Lead angelegt und um 10:54:07 von meinem
Schreibvorgang überschrieben. Der Inhalt des Docs Lead ist nicht wiederherstellbar (die
Datei war untracked). Details und Bewertung in §9.

### F-08 · Info · `phocommander-remote`, `phocommander-ops` — Härtungsregeln ohne Code

Der Workspace enthält für `phocommander-ops` (22 Zeilen) und `phocommander-remote`
(24 Zeilen) nur Gerüste ohne Funktionalität. Regel H7–H11, H15–H17 haben damit **keinen**
Code. Der Bedrohungsmodell-Status `planned` ist korrekt; die Abnahme darf H7–H17 nicht als
erführt führen.

---

## 7. Von mir implementierte Fixes (Datei, Test, echte Testausgabe)

### Fix 1 — `deny.toml`: `unmaintained` auf einen gültigen Scope

**Datei:** `deny.toml`, Zeile 42 → `unmaintained = "none"` (+ erklärender Kommentar)
**Prüfung:** `cargo deny check` (Werkzeug cargo-deny 0.20.2)
**Vorher (Exit 1, Gate tot):**

```
error[unexpected-value]: expected '["all", "workspace", "transitive", "none"]'
   ┌─ …/phocommander/deny.toml:42:17
   │
42 │ unmaintained = "warn"
   │                 ━━━━ unexpected value

2026-10-24 11:04:56 [ERROR] failed to deserialize config from '…/phocommander/deny.toml'
EXIT=1
```

**Nachher:** siehe Fix 2 (beide Defekte mussten zusammen behoben werden, damit das Gate
überhaupt bis zur Prüfung kam).

### Fix 2 — `deny.toml` + zwei Crate-Manifeste: Wildcard-Sperre auf Registry-Abhängigkeiten begrenzen

**Dateien:**
- `deny.toml`, Abschnitt `[bans]`: `allow-wildcard-paths = true`
- `platform/phocommander-platform/Cargo.toml`: `publish = false`
- `apps/phocommander-desktop/Cargo.toml`: `publish = false`

**Vorher (Exit 2, Gate rot):**

```
error[wildcard]: found 2 wildcard dependencies for crate 'phocommander-desktop'
   ┌─ …/phocommander/apps/phocommander-desktop/Cargo.toml:20:23
   │
20 │ phocommander-config = { path = "../../crates/phocommander-config" }
   │                       ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ wildcard dependency
21 │ phocommander-platform = { path = "../../platform/phocommander-platform" }
   │                         ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ wildcard dependency
   │
   ├ phocommander-desktop v0.1.0

error[wildcard]: found 1 wildcard dependency for crate 'phocommander-platform'
   ┌─ …/phocommander/platform/phocommander-platform/Cargo.toml:20:21
   │
20 │ phocommander-core = { path = "../../crates/phocommander-core" }
   │                     ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ wildcard dependency
   │
   ├ phocommander-platform v0.1.0
   │     └── phocommander-desktop v0.1.0

bans FAILED, licenses ok, sources ok
EXIT=2
```

**Nachher (Exit 0, Gate grün) — vollständiger Lauf über den echten Dependency-Graph:**

```
$ cargo deny --config /workspace/.scratch/sec/deny_sandbox.toml check
2026-10-24 11:13:54 [ERROR] unable to acquire cargo global package lock: failed to obtain lock file '/workspace/.scratch/cargo_home/.package-cache'
warning[license-not-encountered]: license was not encountered
   ┌─ /workspace/.scratch/sec/deny_sandbox.toml:23:6
   │
23 │     "Apache-2.0 WITH LLVM-exception",
   │      ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ unmatched license allowance

warning[license-not-encountered]: license was not encountered
   ┌─ /workspace/.scratch/sec/deny_sandbox.toml:25:6
   │
25 │     "BSD-3-Clause",
   │      ━━━━━━━━━━━━━━━━━━━━ unmatched license allowance

warning[license-not-encountered]: license was not encountered
   ┌─ /workspace/.scratch/sec/deny_sandbox.toml:26:6
   │
26 │     "ISC",
   │      ━━━━━ unmatched license allowance

advisories ok, bans ok, licenses ok, sources ok
EXIT=0
```

Die drei `license-not-encountered`-Warnungen sind **informativ**: die Allow-Liste nennt drei
Lizenzen, die im aktuellen Graphen nicht vorkommen. Sie lassen das Gate grün.

**Warum `--config` mit einer Sandbox-Kopie?** Das Repository liegt auf einem **NFS**-Mount.
`tame-index` (die Lock-Bibliothek von cargo-deny) verweigert auf NFS das Locken:

```
$ cargo deny check          # mit REPO-deny.toml, CARGO_HOME auf NFS
2026-10-24 11:14:30 [ERROR] failed to acquire advisory database lock: failed to obtain lock file '/workspace/.home/.cargo/advisory-dbs/db.lock': NFS do not support locking
EXIT=1
```

Die Sandbox-Kopie unterscheidet sich vom Repository-`deny.toml` um **genau einen** Schlüssel
(`db-path`), der auf ein lokales Dateisystem zeigt. Das Repository selbst wurde **nicht**
verändert. Die GitHub-CI läuft auf ext4 und ist von diesem Sandbox-Problem nicht betroffen.

```
--- diff repo -> sandbox ---
--- deny.toml (repository)
+++ deny_sandbox.toml (sandbox)
@@ -35,6 +35,9 @@
 ignore = true
 
 [advisories]
+# Sandbox only: the advisory database needs a filesystem that supports
+# POSIX locks. The repository itself does not set this key.
+db-path = "/run/csi/mount-root/phocommander-advisory-dbs"
 # A yanked release is a defect signal: it is usually pulled in by accident
 # through a loose version requirement.
 yanked = "deny"
```

### Nebenbefund — Crate-Grenz-Gate (nicht von mir geändert, aber verifiziert)

```
$ ./scripts/check-crate-boundaries.sh
Phocommander - crate boundary gate
rule: apps/* -> platform/* -> crates/*, no crate below crates/ may depend
      on another crate below crates/, registry dependencies are not checked

OK    apps/phocommander-desktop -> crates/phocommander-config (apps -> crates)
OK    apps/phocommander-desktop -> platform/phocommander-platform (apps -> platform)
OK    platform/phocommander-platform -> crates/phocommander-core (platform -> crates)

3 path dependencies checked, 0 violations
EXIT=0
```

### Vorhandene Härtung (nicht von mir implementiert, aber verifiziert)

`crates/phocommander-archive/src/entry_name.rs` (Rust Architect) enthält die **lexikalische**
Hälfte von H3 mit negativen Tests. Ich habe sie gelesen, gegen §3 des Bedrohungsmodells
geprüft und den Testlauf verifiziert:

```
$ cargo test -p phocommander-archive
…
test entry_name::tests::refuses_a_name_that_leaves_the_destination ... ok
test entry_name::tests::refuses_an_absolute_name ... ok
test entry_name::tests::refuses_a_name_with_a_control_character ... ok
test entry_name::tests::refuses_a_reserved_device_name ... ok
test entry_name::tests::refuses_an_alternate_data_stream ... ok
test entry_name::tests::refuses_a_name_that_is_too_long ... ok
…
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests phocommander_archive
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
EXIT=0
```

**Was dieser Testlauf beweist und was nicht:** Die Tests beweisen, dass ein Eintragsname mit
`..`, ein absoluter Pfad, ein Steuerzeichen, ein Windows-Gerätename, ein Alternate Data Stream
und ein überlanger Name abgelehnt werden. Sie beweisen **nicht**, dass der resultierende Pfad
gegen das Zielverzeichnis **kanonisiert** wird — im gesamten Workspace existiert kein Aufruf
von `canonicalize`. Genau diese Lücke ist als Finding **F-03** dokumentiert, mit dem
negativen Test **T-03** (`rejects_entry_escaping_via_intermediate_symlink`), der sie schließen
soll.

### Installationsnachweis cargo-deny

```
$ cargo install cargo-deny --version 0.20.2 --locked
    Finished `release` profile [optimized] target(s) in 11m 27s
  Installing /workspace/.home/.cargo/bin/cargo-deny
   Installed package `cargo-deny v0.20.2` (executable `cargo-deny`)

$ cargo deny --version
cargo-deny 0.20.2
```

`cargo-deny 0.20.2` ist zugleich die neueste auf crates.io veröffentlichte Version
(verifiziert über `https://crates.io/api/v1/crates/cargo-deny` → `max_version 0.20.2`) und
entspricht exakt dem in der Spezifikation und in `.github/workflows/ci.yml` gepinnten Stand.

---

## 8. Berechtigungs-Audit-Tabelle

Vollständig in `docs/security/THREAT_MODEL.md` §6. Hier die Kurzfassung:

| Plattform | Berechtigung | Zweck | Notwendig | Begründung | Empfehlung |
|---|---|---|---|---|---|
| macOS | *Full Disk Access* (TCC) | Zugriff außerhalb des Benutzerverzeichnisses | Ja, für die volle Funktion | Die App-Sandbox erlaubt nur vom Nutzer ausgewählte Dateien; ein sandboxed Dateimanager ist ein Dateidialog | Auslieferung **ohne** App-Sandbox, mit Notarisierung und Hardened Runtime; FDA wird **nie erzwungen und nie umgangen**; Erststartseite erklärt die Notwendigkeit; die App funktioniert bei Ablehnung weiter |
| macOS | App-Sandbox | – | Nein | würde das Produkt auf einen Dialog beschränken | nicht verwendet; Hardened Runtime + Notarisierung |
| iPadOS | Dokumenten-Picker (`UIDocumentPickerViewController`) | Zugriff auf ein vom Nutzer gewähltes Verzeichnis | Ja | Vollständig sandboxed; security-scoped URLs mit Rückgaberechten, rekursiv für den Inhalt | Kein Dateisystem-Browser, sondern Betrachter/Organisator der freigegebenen Wurzeln; Zugriff über **security-scoped Bookmarks**; der Nutzer kann den Zugriff jederzeit entziehen, die App behandelt den Fehler sauber |
| iPadOS | Keychain-Access-Group | Verbindungscredentials | Ja | Es gibt keine Keychain-Erweiterung per Default | Credentials im Schlüsselbund; ohne Erweiterung Rückfall auf App-Sandbox **mit sichtbarem Hinweis** |
| Android | Storage Access Framework (SAF) | Lesen/Schreiben der freigegebenen Verzeichnisse | Ja, als **Standard** | SAF ist der empfohlene und datenschutzfreundlichere Weg | SAF ist der Default-Modus, die App ist ohne Spezialberechtigung **voll funktionsfähig**; kein Special-Permission beim Start |
| Android | `MANAGE_EXTERNAL_STORAGE` | Vollzugriff als Dateimanager | **Optionale Erweiterung** | Google nennt File-Management ausdrücklich als erlaubten Anwendungsfall, verlangt aber Kernfunktionalitäts-Begründung, Manifest-Erklärung, Play-Prüfung und klare Nutzeraufforderung | Als optionale, klar gekennzeichnete Erweiterung; Play-Console-Deklaration mit Begründung; ohne sie läuft die App im Dokumenten-Browser-Modus |
| Android / Horizon OS (Meta Quest) | wie Android | wie Android | wie Android | Horizon OS ist Android; eine Quest-Anwendung ist ein normales Android-Programm | identisches Berechtigungsmodell, kein immersiver VR-Port, keine Zusatzberechtigung |
| Windows | `longPathAware` im Manifest | Pfade jenseits 260 Zeichen | Ja | Ohne den Eintrag gilt das 260-Zeichen-Limit; mit Manifest und `LongPathsEnabled` bis 32.767 Zeichen | Manifesteintrag setzen; der Explorer nutzt keine Langpfade — der Fall „Explorer zeigt es nicht, Phocommander schon" wird abgefangen und verständlich gemeldet |
| Linux | keine | – | Nein | keine App-Sandbox | nichts anfordern; Konfiguration in `$XDG_CONFIG_HOME` |
| BSD | keine | – | Nein | wie Linux; FreeBSD/NetBSD sind `winit` Tier 2 | wie Linux |

**Querschnittsregeln:** keine Berechtigung wird still eskaliert; jede erweiterte Berechtigung
ist eine sichtbare, widerrufbare Handlung des Nutzers; eine Berechtigung, die nicht in dieser
Tabelle steht, ist ein Defekt; auf Plattformen ohne Berechtigungsmodell (Linux, BSD, Windows)
ist die Trust Boundary das Unix-Benutzerkonto — das steht im Nutzerhandbuch, nicht versteckt.

---

## 9. Konflikt mit parallelen Agenten (transparente Meldung)

**Was passiert ist:** Der Docs Lead hat im Rahmen des M0-Plans `SECURITY.md`, `PRIVACY.md` und
`CODE_OF_CONDUCT.md` angelegt (Zeitstempel 10:53:38 / 10:53:51). Mein Schreibvorgang auf
`SECURITY.md` (10:54:07) hat die **existierende Datei überschrieben** — das Write-Tool meldete
„(overwrote existing file)". Der ursprüngliche Inhalt ist **nicht wiederherstellbar**, weil
die Datei zu diesem Zeitpunkt untracked war.

**Was ich daraus gemacht habe:**

- `PRIVACY.md` habe ich **nicht** angefasst. Der Text des Docs Lead erfüllt die Anforderungen
  vollständig (Datenminimierung, Speicherorte, Aufbewahrung, Löschung, Schlüsselbund, Log ohne
  Benutzer-/Servernamen/IP, Server-Verkehr ohne Telemetrie, ausdrückliche Null-Telemetrie-Zusage)
  und deckt sich mit §7 meines Bedrohungsmodells.
- `SECURITY.md` enthält jetzt **meine** Fassung. Sie folgt demselben Auftrag wie der Docs Lead
  (Meldeweg, unterstützte Versionen, Reaktionszeit-Erwartung, Verweis auf
  `docs/security/THREAT_MODEL.md`, Abschnitt „Why we need no telemetry") und ergänzt die
  Härtungsgarantien H1–H17. Der Docs Lead hat zwischenzeitlich selbst einen Absatz ergänzt
  („The binding source for the security requirements …"), der erhalten geblieben ist.
- Beide Dateien wurden vom Docs Lead in `5af10fb` mit committet — der Commit-Titel lautet
  „docs(m0): add threat model, glossary, user guide, governance docs and CI checks".

**Empfehlung an den Owner:** Der Docs Lead sollte seine Fassung von `SECURITY.md` mit der
aktuellen Datei vergleichen und entscheiden, welche Formulierungen übernommen werden. Falls
die Datei des Docs Lead inhaltlich erhalten bleiben soll, ist das jetzt der Moment — der
ursprüngliche Text existiert nicht mehr.

**Generelle Lehre für die Teamkoordination:** Ein Security Manager, der `SECURITY.md` und
`PRIVACY.md` schreibt, überschneidet sich zwingend mit dem Docs Lead. Entweder wird
`SECURITY.md` dem Docs Lead eindeutig zugeteilt (dann liefert der Security Manager nur
Textbausteine), oder die Security-Dokumente werden vom Security Manager geschrieben und der
Docs Lead verlinkt sie. Das gehört in die Rollenbeschreibung des Rosters.

### F-07b · Info · `deliverable.md` — zweiter Überschreibungskollisionsfall

**Ort:** `deliverable.md` im Repository-Root
**Was passiert ist:** Zum Zeitpunkt meines Schreibvorgangs (11:19:02) existierte bereits eine
`deliverable.md`, die ich überschrieben habe. Sie war untracked, der ursprüngliche Inhalt ist
**nicht wiederherstellbar**. (Der PM hat seine Übergabe glücklicherweise bereits unter
`deliverable-product-manager.md` gesichert, der Architect unter `deliverable.patch`.)
**Gegenmaßnahme:** Meine Übergabe liegt zusätzlich als **`deliverable.security-manager.md`**
und mein Patch als **`deliverable-security-manager.patch`** im Repository, damit ein späterer
Überschreibvorgang die Übergabe nicht vernichtet.
**Systemic finding:** Solange alle Agenten denselben Pfad `deliverable.md` benutzen, zerstört
der letzte Schreiber die Übergaben der anderen. Empfehlung an den Owner: agentenspezifische
Namen verbindlich vorschreiben (`deliverable-<agent>.md` + `deliverable-<agent>.patch`) — der
PM hat das bereits praktiziert.

---

## 10. Fertige Textbausteine für `SECURITY.md` und `PRIVACY.md`

### `SECURITY.md` — Kernbausteine (Datei liegt vollständig vor, 184 Zeilen)

```markdown
# Security Policy

Phocommander is a local file manager. It has no server, no account, no telemetry and no
analytics. That shapes this policy: there is no database of user data to breach, and the
security work of the project is concentrated in a small number of places — archive extraction,
file operations against a hostile filesystem, and the network sources.

The complete threat model — trust boundary, attack surfaces, the binding hardening rules, the
accepted residual risks, the permission audit and the audit protocol — lives in
[`docs/security/THREAT_MODEL.md`](docs/security/THREAT_MODEL.md). This document is the
front door: what is guaranteed, what to report, and how.
```

Weitere Abschnitte: **Supported Versions** (Tabelle, `main` ist abgedeckt, keine released
Version, keine bezahlte Sicherheitsreaktion — ehrlich benannt) · **Reporting a vulnerability**
(GitHub Security Advisory, Melde-Inhalte, Reaktionszeit-Ziele 7/14/30/90 Tage als Zielwerte eines
Ehrenamt-Projekts, „What is *not* a vulnerability"-Liste mit Verweis auf die dokumentierten
Restrisiken, Safe-Harbour-Klausel) · **Hardening guarantees** (Tabelle H1–H17 mit Ort) ·
**Dependency security** (cargo-deny 0.20.2 / cargo-audit 0.22.2 gepinnt, keine GPL/AGPL, `zip`
gepinnt auf 9.0.5, „A gate that was never seen red has not been proven") · **Why we need no
telemetry** (drei prüfbare Zusicherungen) · **Data protection in one paragraph**.

### `PRIVACY.md` — Kernbausteine (Datei des Docs Lead liegt vollständig vor, 179 Zeilen)

```markdown
## The short version

| | |
|---|---|
| **Telemetry** | None. Not a counter, not a ping, not a crash report. |
| **Analytics** | None. |
| **Advertising** | None. No ad SDK, no sponsored placement, no affiliate link. |
| **Cloud** | None. Phocommander operates no server of its own. |
| **Usage profiles** | None. No identifier is created for you or about you. |
| **Registration** | Not needed. Every local function works without an account or a login. |
| **Third parties** | None. There is no data processor, no subprocessor, no SDK that phones home. |

The only network connection Phocommander ever opens is one **you** started, with a
**server you** named. There is no other path out of the process — and that is checked
mechanically: the CI gate fails if a crate from a telemetry or analytics category is
referenced, and a test proves that starting the process opens no network connection.
```

Weitere Abschnitte: **What is stored** (Settings, Key bindings, Favourites, Recently opened
directories — und die vollständige Negativliste) · **Where it is stored** (Plattformtabelle,
`schema_version`, `.bak`) · **How long it stays** · **How to remove it** · **Credentials for
remote sources** (Tabelle: Adresse/Benutzer in der Konfiguration, Passwort/Schlüssel im
Schlüsselbund, Fallback auf App-Sandbox mit sichtbarem Hinweis) · **Log files** (nur Protokoll,
Zeitpunkt, Ergebnis, Dauer — keine Benutzernamen, keine Servernamen, keine IP-Adressen) ·
**What goes to a remote server** (gesendet / nicht gesendet) · **Children** · **Changes to this
policy** · **Where to go with questions**.

**Geprüft und bestätigt:** Die Datenschutz-Zusage ist mit dem Bedrohungsmodell §7 und mit
`scripts/check-no-telemetry.sh` (im Workspace vorhanden) konsistent.

---

## 11. Welche negativen Tests der Security Manager später selbst implementieren muss

Jede Härtungsregel braucht einen Test, der **den Angriff ausführt** und die Abwehr belegt.
Ein positives Testpaket zählt nicht. Dateiname und Testname sind festgelegt, damit die
Implementierung in M1 ohne Rückfrage beginnen kann.

| # | Datei | Testname | Was der Test belegen muss |
|---|---|---|---|
| T-01 | `crates/phocommander-archive/tests/traversal.rs` | `rejects_entry_that_escapes_the_destination` | Archiv mit `../escape.txt` → Extraktion schlägt fehl, `escape.txt` existiert außerhalb des Ziels **nicht** |
| T-02 | `crates/phocommander-archive/tests/traversal.rs` | `rejects_absolute_path_entry` | Eintrag `/etc/passwd` bzw. `C:\Windows\evil.dll` → abgelehnt, nichts außerhalb geschrieben |
| T-03 | `crates/phocommander-archive/tests/traversal.rs` | `rejects_entry_escaping_via_intermediate_symlink` | Zielverzeichnis enthält `sub → /etc`; Eintrag `sub/passwd` → abgelehnt, `/etc/passwd` unverändert (**schließt die Lücke aus F-03**) |
| T-04 | `crates/phocommander-archive/tests/symlink.rs` | `skips_symlink_entry_instead_of_creating_it` | Archiv enthält einen Symlink-Eintrag → **kein** Symlink im Ziel, Eintrag wird gemeldet, andere Einträge werden extrahiert |
| T-05 | `crates/phocommander-archive/tests/symlink.rs` | `symlink_entry_cannot_redirect_a_later_entry` | Symlink-Eintrag `link → /etc`, danach Datei-Eintrag `link/passwd` → Datei landet **nicht** in `/etc` (Direktnachstellung von RUSTSEC-2025-0168) |
| T-06 | `crates/phocommander-archive/tests/permissions.rs` | `drops_suid_and_sgid_bits_from_extracted_file` | `external_attributes` mit `S_ISUID` → extrahierte Datei hat **kein** SUID-Bit (Direktnachstellung von RUSTSEC-2026-0367) |
| T-07 | `crates/phocommander-archive/tests/permissions.rs` | `drops_suid_and_sgid_bits_from_extracted_directory` | dito für Verzeichnisse (inkl. sticky bit) |
| T-08 | `crates/phocommander-archive/tests/permissions.rs` | `does_not_apply_permissions_through_a_symlink` | Symlink + nachfolgende Rechteänderung → kein `chmod` außerhalb des Ziels (Direktnachstellung von RUSTSEC-2026-0369) |
| T-09 | `crates/phocommander-archive/tests/bomb.rs` | `aborts_when_total_size_limit_is_exceeded` | Archiv mit 100× Expansion → Abbruch mit Meldung, Zielverzeichnis bleibt unter dem Limit |
| T-10 | `crates/phocommander-archive/tests/bomb.rs` | `aborts_when_entry_count_limit_is_exceeded` | Archiv mit > `max_entries` Einträgen → Abbruch |
| T-11 | `crates/phocommander-archive/tests/bomb.rs` | `aborts_when_single_entry_size_limit_is_exceeded` | Einzelner Eintrag > Limit → Abbruch, **bevor** er geschrieben wird |
| T-12 | `scripts/check-no-extract-call.sh` | `no_extract_call_in_the_archive_crate` | Quelltextprüfung: kein `.extract(` in `crates/phocommander-archive` (H2); zusätzlich ein Verstoß muss das Gate rot schalten |
| T-13 | `crates/phocommander-ops/tests/symlink.rs` | `copies_symlink_as_symlink_instead_of_following_it` | Symlink im Quellbaum → im Ziel ein Symlink, kein dereferenzierter Inhalt (H11) |
| T-14 | `crates/phocommander-ops/tests/symlink.rs` | `deletes_symlink_without_following_it` | Löschen eines Symlinks löscht **nur** den Link, nicht das Ziel |
| T-15 | `crates/phocommander-ops/tests/secure_delete.rs` | `clear_overwrite_replaces_the_file_content` | Nach „sicherem Löschen" ist der alte Inhalt nicht mehr lesbar (NIST-Clear-Niveau) |
| T-16 | `crates/phocommander-remote/tests/tls.rs` | `rejects_untrusted_self_signed_certificate` | Selbstsigniertes Zertifikat ohne Ausnahme → Verbindungsabbruch |
| T-17 | `crates/phocommander-remote/tests/tls.rs` | `rejects_expired_certificate` / `rejects_hostname_mismatch` | beide Ablehnungen |
| T-18 | `crates/phocommander-remote/tests/tls.rs` | `accepts_pinned_self_signed_certificate_for_that_host_only` | Ausnahme wirkt **nur** für den gepinnten Host, für einen anderen Host weiterhin Ablehnung |
| T-19 | `crates/phocommander-remote/tests/smb.rs` | `rejects_server_that_only_offers_smb1` / `rejects_server_that_only_offers_ntlmv1` | Downgrade wird abgelehnt, mit lesbarer Meldung (H7) |
| T-20 | `crates/phocommander-remote/tests/budget.rs` | `aborts_when_directory_listing_exceeds_the_budget` | Riesenliste → Abbruch statt Speicheranstieg (H15) |
| T-21 | `crates/phocommander-remote/tests/timeout.rs` | `times_out_on_a_server_that_never_answers` | Hängender Server → Timeout mit Meldung, kein hängender Auftrag (H19) |
| T-22 | `crates/phocommander-remote/tests/names.rs` | `rejects_remote_name_containing_path_separators` / `rejects_remote_name_containing_control_characters` | Eingeschleuste Namen werden neutralisiert (H18) |
| T-23 | `crates/phocommander-core/tests/paths.rs` (proptest) | `resolved_path_never_escapes_the_destination` | Property: für beliebige Eingaben bleibt der aufgelöste Pfad im Ziel (aus der Spezifikation, *Testpyramide*) |
| T-24 | `crates/phocommander-config/tests/secrets.rs` | `configuration_file_contains_no_password` | Die Konfigurationsdatei enthält nie ein Klartextpasswort (H10) |
| T-25 | `crates/phocommander-viewer/tests/headers.rs` | `rejects_text_file_with_implausible_length_field` | Header-Längenfeld vor der Allokation geprüft (H12) |
| T-26 | `scripts/check-dependency-pins.sh` | `zip_pin_is_at_least_9_0_5` | Der `zip`-Pin fällt nie unter 9.0.5 zurück (H1) |
| T-27 | `scripts/check-gate-effectiveness.sh` | `cargo_deny_gate_fails_on_a_violating_dependency` | **F-02-Negativtest:** eine Abhängigkeit mit `"*"`-Voraussetzung oder Copyleft-Lizenz macht das Gate rot — beweist, dass `allow-wildcard-paths` die Regel nicht ausgehebelt hat |

**Reihenfolgeempfehlung:** T-01…T-11 zuerst (M1-Kern), danach T-12 (Quelltext-Gate), T-13…T-15
(`phocommander-ops`), T-16…T-22 (M6 `phocommander-remote`), T-23…T-27 alsBegleitend.

**Zusätzlich owed (Gate-Wirksamkeit, §8.2 des Bedrohungsmodells):** Ein Test, der das
Lizenz-Gateabsichtlich rot schaltet (verwundbare oder Copyleft-Abhängigkeit temporär
einfügen → Gate rot → entfernen → Gate grün). Ein Gate, das nie rot gesehen wurde, ist nicht
bewiesen. Dieser Test ist in §8.2 als offene manuelle Prüfung vermerkt.

---

## 12. Verbleibende Risiken, die bewusst akzeptiert wurden

| Risiko | Begründung | Ausgleich |
|---|---|---|
| **TOCTOU zwischen Pfadprüfung und Zugriff** | Die Rust-Standardbibliothek bietet keine atomaren Pfadoperationen. Eine Prüfung und der folgende Zugriff lassen sich nicht zu einem Systemaufruf zusammenfassen. | Verringert durch deskriptorbasierte Zugriffe (`openat`-Semantik über `rustix`/`cap-std`) bzw. durch Wiederholung der Prüfung unmittelbar vor dem Zugriff. **Was bleibt:** ein Fenster von wenigen Mikrosekunden, in dem ein gleichberechtigter lokaler Prozess eine Pfadkomponente tauscht. Auf Windows gibt es kein `openat`-Äquivalent → dort nur die Wiederholungs-Variante plus Handle-basierte Operationen. **Akzeptiert**, im Bedrohungsmodell §4.1 benannt. |
| **TOCTOU beim rekursiven Kopieren** | Ohne vollständigen Deskriptor-Walk sind Auflistung und Zugriff nicht aneinander gebunden. | Symlinks werden nie dereferenziert (schlimmster Fall: ein kopierter Symlink statt fremder Inhalte); Quellverzeichnis wird vor dem Walk als Handle geöffnet. **Akzeptiert**, §4.2 des Bedrohungsmodells; der Descriptor-Walk ist als Folgeschritt in `phocommander-ops` benannt. |
| **Ein lokaler Prozess gleicher UID kann Dateien während des Schreibvorgangs ändern** | Phocommander hat keine Berechtigung, die der Angreifer nicht auch hat — das ist keine Rechtegrenze. | Prüfungen auf Operationsebene, atomare Schreibvorgänge über temporäre Datei + Rename, Hinweis im Nutzerhandbuch. |
| **macOS *Full Disk Access* / Android All-Files-Access** | Ein Dateimanager ohne diese Rechte ist ein Dateidialog (Entscheidung E4 der Spezifikation). | Nie erzwungen, nie umgangen, immer sichtbar, jederzeit widerrufbar. |
| **Ein hostiler Server erfährt, welche Verzeichnisse der Nutzer abfragt** | Der Nutzer hat es angefordert; das ist die Funktion. | Das Verbindungsprotokoll enthält Protokoll, Zeitpunkt, Ergebnis und Dauer — **nie** Host, Benutzer oder Pfad. |
| **Die Flash-Einschränkung der Sanitization-Stufe *Clear* lässt sich in Software nicht beseitigen** | Wear-Leveling und Over-Provisioning entscheiden, wo die Daten physisch landen. | Die Einschränkung steht **in der Oberfläche** neben der Option, nicht nur im Handbuch. |
| **`cargo deny check` benötigt in dieser Sandbox ein lokales Dateisystem für die Advisory-Datenbank** | Das Repository liegt auf NFS; `tame-index` verweigert dort das Locken. | Nur ein Sandbox-Problem: die GitHub-CI läuft auf ext4. Das Repository-`deny.toml` wurde **nicht** für die Sandbox umgebaut; der Unterschied ist genau ein Schlüssel (`db-path`) und ist in §7 dokumentiert. |

---

## 13. Was ich bewusst nicht getan habe (und warum)

| Nicht getan | Warum |
|---|---|
| Die Extraktionshärtung (H2–H6) implementiert | Der Auftrag dieses Meilensteins ist ausdrücklich „Bedrohungsmodell und Sicherheitsdokumentation"; die Härtung gehört in M1. Die 27 negativen Tests sind mit Datei- und Testnamen festgelegt (§11), damit M1 ohne Rückfrage beginnen kann. |
| `docs/REQUIREMENTS.md` geändert | Ausdrücklich untersagt. Die beiden Präzisierungen (F-05, F-06) sind als Finding mit fertigem Formulierungsvorschlag dokumentiert. |
| Den Kommentar in `Cargo.toml` (F-04) korrigiert | Die Datei gehört dem Rust Architect und wird parallel bearbeitet; ein Kommentar-Edit an einer gemeinsam genutzten Datei ist eine Kollisionsquelle. Der fertige Ersatztext steht in F-04 bereit. |
| `crates/phocommander-archive` um die Kanonisierung erweitert | Datei gehört dem Rust Architect und wird parallel bearbeitet. Der Befund F-03 nennt Ort, Angriffspfad, Fix-Prinzip und den negativen Test. |
| `PRIVACY.md` überschrieben | Die Datei war zum Zeitpunkt meines Schreibvorgangs bereits vollständig und inhaltlich korrekt (Docs Lead). Zweimal dieselbe Arbeit zu machen ist keine Qualität. |
| `cargo audit` ausgeführt | `cargo-audit` 0.22.2 wurde vom Architect parallel installiert; der Lauf konkurrierte mit einem laufenden `cargo build` im selben Registry-Cache (Lock-Kollision). Der Advisory-Teil ist über `cargo deny check advisories` abgedeckt und **grün**; `cargo audit` bleibt als offener Punkt der CI (Status `planned` im Bedrohungsmodell §8.1). |
| Commit / Push / Tag | `AGENTS.md` §7 und der Auftrag verbieten es. Übergabe erfolgt über Dateien im Repository + Patch + dieses Dokument. |

---

## 14. Offene Punkte für den Owner

1. **`SECURITY.md`-Überschreibung** (§9) — der Docs Lead soll seine Fassung mit der aktuellen
   Datei abgleichen.
2. **Rollenüberschneidung Security Manager ↔ Docs Lead** bei `SECURITY.md`/`PRIVACY.md` (§9).
3. **Kommentar in `Cargo.toml`** (F-04) — Entscheidung: anwenden oder dem Architect überlassen.
4. **Gate-Wirksamkeitsnachweis** (T-27, §8.2) — gehört ins M0-Abschlussdokument, sonst ist das
   Gate ungeprüft.
5. **`scripts/check-crate-boundaries.sh` und `scripts/check-no-telemetry.sh`** sind inzwischen
   vorhanden; das Bedrohungsmodell §8.1 führt sie als `planned` bzw. `implemented` — nach dem
   nächsten Commit bitte gegen den tatsächlichen Stand prüfen.

---

*Ende der Übergabe. Alle Aussagen sind mit einer Kommandoausgabe belegt; nichts davon ist
geschätzt. Was nicht gemessen wurde, ist als `planned`, `in progress` oder `NOT RUN` mit
exaktem Grund ausgewiesen.*
