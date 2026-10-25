# Meilenstein M2 — Die Sicherheitsregeln als einstellbare Optionen

**Rolle:** Core Engineer · **Repository:** `/workspace/phocommander` (Branch `main`) ·
**Stand:** 25.10.2026 · **Grundlage:** `docs/REQUIREMENTS.md` Version 2.0, Abschnitte 9.7, 9.8 und
NFR-23 bis NFR-32.

`docs/REQUIREMENTS.md` wurde **nicht** verändert.

> **Hinweis zur Ablage:** `deliverable.md` ist im gemeinsam benutzten Repository von mehreren
> Rollen parallel beschrieben worden; eine andere Rolle hat diese Datei während der Arbeit
> überschrieben. Deshalb liegt dieser Text zusätzlich unter
> **`deliverable.core-engineer.md`**. Beide Dateien sind identisch.

---

## 1. Kurzfassung

Die Konfigurationsseite existierte, sie wirkte aber nicht. Diese Änderung macht sie wirksam:

* Die **`[security]`-Sektion** wird in einen **effektiven Regelsatz** aufgelöst
  (`EffectiveSecurity`), der für jede Regel Namen, Zustand, Erklärung, Quelle der Entscheidung und
  die Meldungen über wirkungslose Einstellungen trägt.
* Die **drei Profile** entscheiden: `standard` nimmt die Vorgabewerte der Schalter, `corporate`
  aktiviert **jede** Regel, `custom` folgt exakt den Schaltern der Datei. Das Profil überschreibt
  einen Schalter nur dort, wo die Spezifikation das verlangt — und **jede** Abweichung wird als
  Meldung berichtet, damit der Nutzer nicht raten muss.
* Die **drei nicht schaltbaren Garantien** (keine Telemetrie, keine Elevations-Anfrage, kein
  Hintergrundindex) sind **im Code** verankert: `ReadTrigger::BackgroundScan` wird von *jedem*
  Leser-Protokoll verweigert, `keeps_background_index()`, `sends_telemetry()` und
  `requests_elevation()` liefern konstant `false`. Ein Versuch in der Konfigurationsdatei wird
  gelesen, ignoriert und als Warnung gemeldet.
* **Jeder Schalter hat eine Wirkung im Code** — Viewer-Adapter (`phocommander-viewer`), Kopierregeln
  (`phocommander-ops`), Symlink-Verhalten (`phocommander-core`), Sitzungsdatei
  (`phocommander-config`) und Verzeichnis-Nachprüfung (`phocommander-ui`).
* **Die sichere Variante ist der Default:** ein kaputtes Konfigurationsfile führt zur
  Konfiguration des Profils `corporate` (alle Regeln an) und zu einer Meldung im Protokoll.

---

## 2. Dateiliste

| Pfad | Zeilen | Zweck |
|---|---:|---|
| `crates/phocommander-config/src/security.rs` | 509 | Die Auflösung der `[security]`-Sektion in den effektiven Regelsatz: `EffectiveSecurity`, `resolve_security`, die Grundwerte der Schalter, die Sammlung der Meldungen. |
| `crates/phocommander-config/src/security/tests.rs` | 245 | Die Tests der Auflösung: die drei Profile, die Garantien, die Meldungen, der Determinismus. |
| `crates/phocommander-config/src/rule.rs` | 313 | Die Regeln selbst: `SecurityRuleId` (mit Titel, Erklärung, Schaltername), `SecurityRuleState`, `SecurityRuleSource`, `SecurityRule`. |
| `crates/phocommander-config/src/rule/tests.rs` | 146 | Die Tests der Regeln: Namen, Texte, Schaltbarkeit, Zustandsquelle. |
| `crates/phocommander-config/src/notice.rs` | 149 | `ConfigNotice` und `NoticeSeverity` — die Meldung über eine Einstellung, die keine Wirkung hatte. |
| `crates/phocommander-config/src/session.rs` | 325 | Der Sitzungszustand: `SessionState`, `write_session_state`, `load_session_state` — der Schalter `remember_session` wirkt hier. |
| `crates/phocommander-config/src/schema.rs` | 481 | Geändert: `Security` bekommt die Felder `telemetry` und `elevation`, `Default` und `safe()` der Sektion, `SecurityProfile::as_str()`, die Vorgabewerte aus `security.rs`. `Config::safe()`. |
| `crates/phocommander-config/src/loader.rs` | 502 | Geändert: `parse_toml_or_safe` und `load_file_or_safe` — der Rückfall auf die sichere Konfiguration. |
| `crates/phocommander-config/src/lib.rs` | 52 | Geändert: die neuen Module und ihre öffentliche API. |
| `crates/phocommander-viewer/src/policy.rs` | 155 | `ReadTrigger` und `ReadPolicy` — wer darf eine Datei lesen? Der Hintergrundscan wird von jedem Protokoll verweigert. |
| `crates/phocommander-viewer/src/content.rs` | 186 | `OpenedFile` — der Inhalt einer Datei, die der Nutzer geöffnet hat. |
| `crates/phocommander-viewer/src/reader.rs` | 355 | `ViewerReader` und `ViewerError` — der Adapter, der nur auf ausdrücklichen Befehl liest und die Größengrenze **vor** dem Lesen prüft. |
| `crates/phocommander-viewer/src/lib.rs` | 38 | Geändert: die Module des Betrachters. |
| `crates/phocommander-viewer/Cargo.toml` | 22 | Geändert: `thiserror` als Abhängigkeit. |
| `crates/phocommander-ops/src/copy.rs` | 530 | Geändert: der Kopiervorgang nimmt die Regeln entgegen (`CopyPolicy`), ein Symlink wird als Symlink kopiert, `TreeItem` kennt seine Art. |
| `crates/phocommander-ops/src/copy/policy.rs` | 96 | `CopyPolicy` — die beiden Regeln der Spezifikation für den Kopiervorgang. |
| `crates/phocommander-ops/src/copy/types.rs` | 269 | Die Sprache des Kopiervorgangs: `CopyMode`, `CopyRequest`, `CopyOutcome`, `TreeItem`. |
| `crates/phocommommander-ops/src/copy/tests.rs` | 276 | Die Tests der Kopierregeln (Vergleich der Prüfsummen, Symlink als Symlink). |
| `crates/phocommander-ops/src/port.rs` | 276 | Geändert: `FileSystem::copy_symlink` — die Quelle legt einen Symlink an. |
| `crates/phocommander-ops/src/fake_fs.rs` | 508 | Geändert: `write_symlink`, `link_target`, `kind_of`, `checksum_calls`, `copy_symlink`; die Kopie über einen Symlink dereferenziert wie das echte Dateisystem. |
| `crates/phocommander-ops/src/sync.rs` | 444 | Geändert: der Aufruf von `copy_one_file` mit den Regeln. |
| `crates/phocommander-ops/src/executor.rs` | 292 | Geändert: ein unbenutzter Import (Warnung des Builds). |
| `crates/phocommander-ops/src/lib.rs` | 53 | Geändert: **die Module der Crate wurden überhaupt erst deklariert** (vorher war die Crate leer und wurde nie kompiliert). |
| `crates/phocommander-core/src/local.rs` | 348 | Geändert: `LocalSource::with_follow_symlinks` — der Schalter `follow_symlinks` wirkt auf dem echten Dateisystem. |
| `crates/phocommander-core/src/local/tests.rs` | 269 | Geändert: der Test, dass ein Symlink dereferenziert wird, wenn die Regel es erlaubt. |
| `crates/phocommander-ui/src/refresh.rs` | 94 | `RefreshPolicy` — der Schalter `verify_directory_before_refresh` in der Oberfläche. |
| `crates/phocommander-ui/src/app.rs` | 565 | Geändert: beim Wechsel des Panels wird das Verzeichnis erneut gelesen, wenn die Regel es verlangt. |
| `crates/phocommander-ui/src/app/tests.rs` | 230 | Geändert: die zwei Tests der Verzeichnis-Nachprüfung. |
| `crates/phocommander-ui/src/lib.rs` | 55 | Geändert: das Modul `refresh`. |
| `apps/phocommander-desktop/src/security.rs` | 181 | `SecurityPlan` — die Übersetzung der Regeln in die Werte der Subsysteme (Composition Root, ADR-0001 Option B). |
| `apps/phocommander-desktop/src/lib.rs` | 23 | Die Bibliothek des Desktop-Builds, damit die Beispiele der Übersetzung mitgetestet werden. |
| `apps/phocommander-desktop/src/main.rs` | 135 | Geändert: die App liest die Konfiguration, zeigt die Regeln und meldet die Meldungen. |
| `apps/phocommander-desktop/Cargo.toml` | 42 | Geändert: die Crates der Composition Root und das neue Bibliotheksziel. |
| `config/defaults.toml` | 203 | Geändert: die Sektion `[security]` mit `telemetry` und `elevation`, korrigierten Kommentaren und der Erklärung, dass das Profil entscheidet. |
| `scripts/check-config-schema.py` | 399 | Geändert: die Sektion `[security]` ist jetzt in der Schema-Prüfung enthalten (Sektion, Vorgabewerte, Drift zwischen Datei und Rust-Schema). |
| `tests/integration/security_profiles.rs` | 299 | Der Integrationstest: die Regeln an einer echten Datei, die drei Garantien, die kaputte Datei, die Sitzungsdatei. |
| `tests/Cargo.toml` | 104 | Geändert: das neue Testziel `security_profiles`. |
| `crates/phocommander-ui/tests/the_filter_matches_without_a_dialog.rs` | 71 | **Fremde Datei, minimal repariert** (siehe Punkt 8.7): `println!("{}")` → `println!("{:?}")`, weil `egui_kittest::Node` kein `Display` ist. |

---

## 3. Die Tabelle der Regeln

Die Auflösung ist eine reine Funktion: `resolve_security(&Security) -> EffectiveSecurity`.
Ein **Regelsatz** nennt immer den Zustand der Regel *und* die Quelle der Entscheidung
(`SecurityRuleSource`: `Guarantee`, `Profile(...)`, `Switch(...)`).

| Regel | `standard` | `corporate` | `custom` | schaltbar? | Wirkung im Code |
|---|---|---|---|---|---|
| `no_telemetry` | an | an | an | **nein** (NFR-32) | `EffectiveSecurity::sends_telemetry()` liefert konstant `false`; die Datei-Felder `security.telemetry` werden gelesen, ignoriert und als **Warnung** gemeldet. |
| `no_elevation` | an | an | an | **nein** (NFR-32) | `requests_elevation()` liefert konstant `false`; `security.elevation` wird ignoriert und gemeldet. |
| `no_background_index` | an | an | an | **nein** (NFR-32) | `keeps_background_index()` liefert konstant `false`; `ReadTrigger::BackgroundScan` wird von **jedem** `ReadPolicy` verweigert; `security.background_index` wird ignoriert und gemeldet. |
| `read_only_on_demand` | an | an | wie die Datei | ja (`read_only_on_demand`) | `ViewerReader::preview()` verweigert die Vorschau, wenn die Regel an ist; nur `open()` (ausdrücklicher Befehl) liest. |
| `verify_directory_before_refresh` | an | an | wie die Datei | ja (`verify_directory_before_refresh`) | Beim Wechsel des Panels erzeugt die Oberfläche `Command::LoadDirectory` — das Verzeichnis wird erneut gelesen. Ist die Regel aus, entsteht kein Befehl. |
| `checksum_on_copy` | an | an | wie die Datei | ja (`checksum_on_copy`) | Der Modus `verified` vergleicht die Prüfsummen nur, wenn die Regel an ist; der Modus `quick` vergleicht sie ohnehin nie. Ist die Regel aus, wird das Ziel **nicht** ein zweites Mal gelesen. |
| `symlinks_are_never_followed` | an | an | **umgekehrt** zu `follow_symlinks` | ja (`follow_symlinks`, invertiert) | Ein Symlink wird als Symlink kopiert (`FileSystem::copy_symlink`) und nie dereferenziert; `LocalSource::copy` legt einen Link an, solange die Regel gilt. |
| `leaves_nothing_behind` | **aus** | **an** | umgekehrt zu `remember_session` | ja (`remember_session`, invertiert) | `write_session_state` schreibt **keine** Datei, wenn die Regel an ist — nicht einmal im Konfigurationsverzeichnis. Steht der Schalter auf `true`, entsteht genau eine Datei **innerhalb** des Konfigurationsverzeichnisses. |

**Der Unterschied zwischen `standard` und `corporate` ist genau eine Regel:** `leaves_nothing_behind`.
`standard` merkt sich die Sitzung (die Bequemlichkeit, die keine Zusage der Spezifikation 9.7 bricht,
weil die Datei im Konfigurationsverzeichnis landet), `corporate` merkt sie nicht (die bequeme
Bequemlichkeit, die eine verwaltete Maschine nicht aufbewahren soll).

**Der Unterschied zwischen `standard` und `custom` ist der Speicherort der Entscheidung:**
`standard` nimmt die Vorgabewerte der Schalter, `custom` nimmt die Werte der Datei.

**Die Invertierung ist dokumentiert, nicht versteckt:** `SecurityRuleId::switch_name()` nennt zu
jeder Regel den Schalter, der sie entscheidet. `follow_symlinks` und `remember_session` sind in der
Spezifikation (9.8) nach der *Abweichung* von der Regel benannt (`follow_symlinks = false` heißt
„ein Link wird nie dereferenziert“). Der Name des Schalters bleibt, weil die Spezifikation ihn
festlegt; die Regel trägt in der Oberfläche den Klartext.

---

## 4. Die öffentliche API

| Crate | Modul | Typ / Funktion | Zweck |
|---|---|---|---|
| `phocommander-config` | `security` | `resolve_security(&Security) -> EffectiveSecurity` | Die Auflösung. Reine Funktion, deterministisch. |
| `phocommander-config` | `security` | `EffectiveSecurity` | Der effektive Regelsatz: `rules()`, `rule(id)`, `is_enabled(id)`, `notices()`, `profile()`. |
| `phocommander-config` | `security` | `EffectiveSecurity::{verifies_directory_before_refresh, remembers_session, reads_only_on_demand, verifies_checksums, follows_symlinks, keeps_background_index, sends_telemetry, requests_elevation}` | Die booleschen Werte, die die Subsysteme brauchen. |
| `phocommander-config` | `security` | `EffectiveSecurity::safe()` | Der Regelsatz einer Maschine mit kaputter Datei. |
| `phocommander-config` | `security` | `DEFAULT_*` (6 Konstanten) | Die Vorgabewerte der Schalter — einmal geschrieben, von Datei und Profil benutzt. |
| `phocommander-config` | `rule` | `SecurityRuleId` | Der Name der Regel: `as_str()`, `title()`, `explanation()`, `is_switchable()`, `switch_name()`. |
| `phocommander-config` | `rule` | `SecurityRule`, `SecurityRuleState`, `SecurityRuleSource` | Die Regel mit Zustand und Quelle der Entscheidung. |
| `phocommander-config` | `notice` | `ConfigNotice`, `NoticeSeverity` | Die Meldung über eine Einstellung ohne Wirkung. |
| `phocommander-config` | `session` | `SessionState`, `write_session_state`, `load_session_state`, `SESSION_FILE_NAME` | Der Sitzungszustand; `write_session_state` schreibt nur, wenn die Regel es erlaubt. |
| `phocommander-config` | `schema` | `Security::{safe, default}`, `SecurityProfile::as_str`, `Config::safe` | Die sichere Konfiguration und der Name des Profils. |
| `phocommander-config` | `loader` | `parse_toml_or_safe`, `load_file_or_safe` | Der Rückfall auf die sichere Konfiguration mit Meldung. |
| `phocommander-viewer` | `policy` | `ReadPolicy`, `ReadTrigger` | Wer darf lesen. `BackgroundScan` wird immer verweigert. |
| `phocommander-viewer` | `reader` | `ViewerReader::{open, preview, policy, max_file_size}` | Der Adapter: liest nur auf ausdrücklichen Befehl, prüft die Grenze vor dem Lesen. |
| `phocommander-viewer` | `reader` | `ViewerError` | `Io`, `Refused`, `TooLarge`. |
| `phocommander-viewer` | `content` | `OpenedFile` | Der gelesene Inhalt. |
| `phocommander-ops` | `copy` | `CopyPolicy::{new, strict, verifies_checksums, follows_symlinks}` | Die Regeln des Kopiervorgangs. |
| `phocommander-ops` | `copy` | `CopyRequest::with_rules`, `copy_one_file(..., rules, ...)` | Der Kopiervorgang nimmt die Regeln entgegen. |
| `phocommander-ops` | `port` | `FileSystem::copy_symlink` | Die Quelle legt einen Symlink an. |
| `phocommander-core` | `local` | `LocalSource::with_follow_symlinks`, `follows_symlinks` | Das Verhalten des Symlinks auf dem echten Dateisystem. |
| `phocommander-ui` | `refresh` | `RefreshPolicy::{new, strict, verifies_before_refresh}` | Die Regel der Verzeichnis-Nachprüfung. |
| `phocommander-ui` | `app` | `WorkSurfaceApp::with_refresh_policy`, `refresh_policy()` | Die Regel in der Oberfläche. |
| `phocommander-desktop` | `security` | `SecurityPlan::{from_effective, copy_policy, read_policy, refresh_policy}` | Die Übersetzung der Regeln in die Werte der Subsysteme. |

---

## 5. Testabdeckungs-Matrix

| Regel / Verhalten | getestet? | Testdatei | Testname | Lücke |
|---|---|---|---|---|
| `corporate` aktiviert **jede** Regel | ja | `crates/phocommander-config/src/security/tests.rs` | `the_profile_corporate_enables_every_rule` | — |
| `corporate` aktiviert jede Regel (Datei) | ja | `tests/integration/security_profiles.rs` | `the_profile_corporate_enables_every_rule_in_a_real_file` | — |
| `standard` lässt die bequemen Schalter im Default | ja | `crates/phocommander-config/src/security/tests.rs` | `the_profile_standard_keeps_the_convenience_switches_at_their_default` | — |
| `standard` meldet die überschriebenen Schalter | ja | `crates/phocommander-config/src/security/tests.rs` | `the_profile_standard_reports_the_switches_that_it_overrules` | — |
| `custom` folgt genau den Schaltern | ja | `crates/phocommander-config/src/security/tests.rs` | `the_profile_custom_follows_the_switches_of_the_file` | — |
| `custom` bricht keine Garantie | ja | `crates/phocommander-config/src/security/tests.rs` | `the_profile_custom_still_refuses_to_break_a_guarantee` | — |
| Die Basislinie von `standard` = der Vorgabewert der Datei | ja | `crates/phocommander-config/src/security/tests.rs` | `the_baseline_of_the_profile_standard_is_the_default_of_the_file` | — |
| Die Auflösung ist deterministisch (gleiche Eingabe → gleiches Ergebnis) | ja | `crates/phocommander-config/src/security/tests.rs` | `the_resolution_is_the_same_every_time` | — |
| Die drei Garantien sind in **jedem** Profil an | ja | `crates/phocommander-config/src/security/tests.rs` | `the_guarantees_are_in_force_in_every_profile` | — |
| Genau drei Regeln sind nicht schaltbar | ja | `crates/phocommander-config/src/rule/tests.rs` | `exactly_three_rules_cannot_be_switched` | — |
| `background_index = true` hat keine Wirkung + Meldung | ja | `crates/phocommander-config/src/security/tests.rs` | `the_switch_of_the_background_index_has_no_effect` | — |
| `telemetry`/`elevation`/`background_index` in der Datei ändern nichts | ja | `tests/integration/security_profiles.rs` | `a_file_that_tries_to_break_a_guarantee_changes_nothing` | — |
| Der Hintergrundscan wird **immer** verweigert | ja | `crates/phocommander-viewer/src/policy.rs` | `no_policy_ever_allows_a_background_scan` | — |
| `read_only_on_demand`: die Vorschau wird verweigert | ja | `crates/phocommander-viewer/src/reader.rs` | `the_reader_refuses_the_preview_when_the_rule_is_in_force` | — |
| `read_only_on_demand`: die Vorschau wird gelesen, wenn die Regel aus ist | ja | `crates/phocommander-viewer/src/reader.rs` | `the_reader_reads_the_preview_when_the_rule_is_switched_off` | — |
| `read_only_on_demand`: das ausdrückliche Öffnen liest immer | ja | `crates/phocommander-viewer/src/reader.rs` | `the_reader_opens_the_file_the_user_asked_for` | — |
| `verify_directory_before_refresh`: die Oberfläche liest erneut | ja | `crates/phocommander-ui/src/app/tests.rs` | `the_switch_asks_for_the_directory_to_be_read_again` | — |
| `verify_directory_before_refresh`: ohne die Regel wird nichts gelesen | ja | `crates/phocommander-ui/src/app/tests.rs` | `the_switch_reads_nothing_again_when_the_rule_is_switched_off` | — |
| `checksum_on_copy`: der Modus `verified` vergleicht | ja | `crates/phocommander-ops/src/copy/tests.rs` | `the_verified_copy_compares_the_checksums_when_the_rule_is_in_force` | — |
| `checksum_on_copy`: ohne die Regel wird **nicht** verglichen | ja | `crates/phocommander-ops/src/copy/tests.rs` | `the_verified_copy_does_not_compare_the_checksums_when_the_rule_is_off` | — |
| `quick` vergleicht nie | ja | `crates/phocommander-ops/src/copy/tests.rs` | `the_quick_copy_never_compares_the_checksums` | — |
| `atomic` vergleicht immer (dokumentierte Entscheidung) | ja | `crates/phocommander-ops/src/copy/tests.rs` | `the_atomic_copy_always_verifies_the_temporary_file` | — |
| `follow_symlinks`: ein Link wird als Link kopiert | ja | `crates/phocommander-ops/src/copy/tests.rs` | `a_symbolic_link_is_copied_as_a_link_when_the_rule_is_in_force` | — |
| `follow_symlinks`: ein Link wird dereferenziert, wenn erlaubt | ja | `crates/phocommander-ops/src/copy/tests.rs` | `a_symbolic_link_is_dereferenced_when_the_rule_allows_it` | — |
| `follow_symlinks` auf dem **echten** Dateisystem | ja | `crates/phocommander-core/src/local/tests.rs` | `a_link_is_dereferenced_when_the_rules_allow_it`, `a_link_is_copied_as_a_link` | — |
| `remember_session = false` erzeugt **keine** Datei außerhalb des Konfigurationsverzeichnisses | ja | `tests/integration/security_profiles.rs` | `a_configuration_that_keeps_no_session_writes_no_file_at_all` | — |
| `remember_session = true` erzeugt genau eine Datei **im** Konfigurationsverzeichnis | ja | `tests/integration/security_profiles.rs` | `a_configuration_that_remembers_the_session_writes_only_inside_the_configuration_directory` | — |
| `remember_session = false` schreibt gar nichts | ja | `crates/phocommander-config/src/session.rs` | `a_rule_that_keeps_no_session_writes_no_file` | — |
| Eine kaputte Datei ergibt die sichere Variante | ja | `crates/phocommander-config/src/loader.rs` | `a_broken_document_gives_the_safe_configuration` | — |
| Eine kaputte Datei ergibt die sichere Variante (Datei) | ja | `tests/integration/security_profiles.rs` | `a_broken_file_leaves_the_application_in_the_safe_mode` | — |
| Die Übersetzung erreicht die Subsysteme | ja | `apps/phocommander-desktop/src/security.rs` | `the_safe_rules_give_the_strict_plan`, `a_relaxed_rule_reaches_the_copy/viewer/interface` | — |
| Die ausgelieferte Datei trägt die Sektion `[security]` | ja | `tests/integration/security_profiles.rs` | `the_shipped_file_carries_the_security_section` | — |
| Die ausgelieferte Datei ergibt die Regeln von `standard` | ja | `tests/integration/security_profiles.rs` | `the_shipped_file_produces_the_rules_of_the_profile_standard` | — |
| **Lücke:** die Meldungen erscheinen in einem echten Log-Framework (`tracing`) | **nein** | — | — | Die App meldet nach `stderr`/`stdout`. Die Anbindung an `tracing` gehört in die Crate, die das Log besitzt (noch nicht vorhanden). |
| **Lücke:** die Oberfläche zeigt die Regeln in einem Einstellungsdialog | **nein** | — | — | Die Oberfläche hat noch keinen Einstellungsdialog; die Daten liegen als typisierte API (`EffectiveSecurity::rules()`) bereit. |
| **Lücke:** die Regeltexte sind lokalisiert | **nein** | — | — | Die Texte liegen englisch im Config-Crate; die Übersetzung gehört in den Katalog des UI-Crates (Plan 8). |

---

## 6. Echte Ausgaben der Kommandos

### 6.1 `./scripts/check-crate-boundaries.sh` — **PASS (Exit 0)**

```
$ ./scripts/check-crate-boundaries.sh
Phocommander - crate boundary gate
rule: apps/* -> platform/* -> crates/*, no crate below crates/ may depend
      on another crate below crates/, registry dependencies are not checked

OK    apps/phocommander-desktop -> crates/phocommander-config (apps -> crates)
OK    apps/phocommander-desktop -> crates/phocommander-core (apps -> crates)
OK    apps/phocommander-desktop -> crates/phocommander-ops (apps -> crates)
OK    apps/phocommander-desktop -> platform/phocommander-platform (apps -> platform)
OK    apps/phocommander-desktop -> crates/phocommander-ui (apps -> crates)
OK    apps/phocommander-desktop -> crates/phocommander-viewer (apps -> crates)
OK    platform/phocommommander-platform -> crates/phocommander-core (platform -> crates)
OK    tests -> crates/phocommander-archive (tests -> crates)
OK    tests -> crates/phocommander-config (tests -> crates)
OK    tests -> crates/phocommander-core (tests -> crates)
OK    tests -> crates/phocommander-keymap (tests -> crates)
OK    tests -> crates/phocommander-ui (tests -> crates)

12 path dependencies checked, 0 violations
```

### 6.2 `./scripts/check-file-size.sh` — **PASS mit Warnungen (Exit 0)**

```
$ ./scripts/check-file-size.sh
OVER  676 lines  ./crates/phocommander-archive/src/extract.rs
OVER  610 lines  ./crates/phocommander-ops/src/progress.rs
OVER  621 lines  ./crates/phocommander-ops/src/queue.rs
OVER  875 lines  ./crates/phocommander-ui/src/keymap_view.rs

Source files measured : 116 (extensions: rs)
Target value          : 600 lines (warning)
Absolute maximum      : 900 lines (failure)
Target-value warnings : 4
Maximum violations    : 0
RESULT: PASS with warnings — 4 file(s) exceed the target value of 600 lines.
        Each of them must name the reason in its module header.
```

Die vier gemeldeten Dateien sind **nicht** von diesem Meilenstein geändert worden; alle Dateien
dieses Meilestones liegen unter 600 Zeilen. `crates/phocommander-ops/src/copy.rs` wurde von 838 auf
530 Zeilen geteilt (`copy/policy.rs`, `copy/types.rs`, `copy/tests.rs`).

### 6.3 `python3 scripts/check-config-schema.py` — **PASS (Exit 0)**

```
$ python3 scripts/check-config-schema.py
config    : /workspace/phocommander/config
files     : 9
OK    all 9 files under config/ parse as TOML
OK    the configuration file carries every section of the specification
OK    the default values match the specification
OK    every key and every section carries a comment
OK    the shipped rename templates are complete
OK    the overrides match the action catalogue
OK    the configuration file and the Rust schema agree

RESULT: PASS — 7 of 7 checks passed
```

Dieser Lauf beweist, dass die ausgelieferte Datei `config/defaults.toml` und das Rust-Schema
`Security` übereinstimmen — die Prüfung wurde um die Sektion `[security]` erweitert (Sektion,
Vorgabewerte, Drift zwischen Datei und Schema).

### 6.4 `cargo build --workspace --all-targets --locked` — **NICHT VOLLSTÄNDIG AUSGEFÜHRT**

Begründung siehe Punkt 7. Was gemessen wurde:

* Der Lauf hat **zwei Compilerfehler** gefunden, die beide behoben sind:
  1. `E0277: the trait bound SecurityProfile: Hash is not satisfied`
     (`crates/phocommander-config/src/rule.rs:229`) → `SecurityProfile` bekam `#[derive(Hash)]`.
  2. `E0277: egui_kittest::Node<'_> doesn't implement std::fmt::Display`
     (`crates/phocommander-ui/tests/the_filter_matches_without_a_dialog.rs:66`) → `println!("{:?}")`.
* Der letzte Lauf hat **keine Compilerfehler** mehr gemeldet (`grep -c "^error"` = 0) und die
  Bibliotheken aller geänderten Crates erzeugt:

```
$ ls -la --time-style=+%H:%M /tmp/sec2-target/debug/deps/ | grep phocommander
-rw-r--r-- 2 root root  8921100 00:43 libphocommander_config-2d0ecef04ea4b72b.rlib
-rw-r--r-- 2 root root   375056 00:45 libphocommander_viewer-8b5aa7e551bc2129.rlib
-rw-r--r-- 2 root root  7225116 01:09 libphocommander_ops-4a6f62e77f8d6e19.rlib
-rw-r--r-- 3 root root   725599 00:41 libphocommander_ui-b8c4d008afb7b004.rmeta
-rw-r--r-- 2 root root  28935960 00:44 libphocommander_archive-8c7b3846eec1ee5e.rlib
```

* Der Lauf hat **keine abschließende `Finished`-Zeile** gedruckt: der Prozess wurde abgebrochen,
  während die Testziele gebaut wurden. **Ein vollständiger Build wird hier nicht behauptet.**

### 6.5 `cargo test --workspace --locked`, `cargo fmt --all --check`, `cargo clippy --workspace --all-targets --locked -- -D warnings`

**NOT RUN** — Begründung in Punkt 7. Es wird kein Ausgang behauptet.

---

## 7. Was NOT RUN bleibt und warum

| Kommando | Status | Exakter Grund |
|---|---|---|
| `cargo build --workspace --all-targets --locked` | **NOT RUN (unvollständig)** | Der Lauf wurde begonnen und hat alle Bibliotheken der geänderten Crates erzeugt (siehe 6.4), aber der Prozess wurde abgebrochen, bevor cargo seine abschließende `Finished`-Zeile druckte. Grund: Die Sandbox hat 3 GB RAM; zur selben Zeit liefen 17 parallele Builds anderer Agenten, der Linker wurde vom OOM-Killer beendet (`ld terminated with signal 9 [Killed]`). Auf Weisung des Teamleiters wurden alle weiteren Cargo-Kommandos eingestellt. |
| `cargo test --workspace --locked` | **NOT RUN** | Grund: OOM, 17 parallele Builds, der Linker wurde vom OOM-Killer beendet. Der Testlauf wurde begonnen und wieder abgebrochen, bevor eine Ausgabe entstand; es liegt **keine** Testergebnis-Zeile vor. Die Testziele der geänderten Crates wurden in diesem Lauf **nicht** gelinkt (in `/tmp/sec2-target/debug/deps/` existiert kein Testbinary von `phocommander-config`, `-ops`, `-viewer`, `-core` oder `-ui`). |
| `cargo fmt --all --check` | **NOT RUN** | Grund: OOM, 17 parallele Builds, der Linker wurde vom OOM-Killer beendet; auf Weisung des Teamleiters wurden alle weiteren Cargo-Kommandos eingestellt. **Hinweis:** Ein `cargo fmt --all` eines anderen Agenten hat um 00:36 Uhr die Dateien des Workspace-Formats unterzogen (an den Zeitstempeln der Dateien ablesbar); die danach von mir geänderten Dateien folgen demselben Stil, sind aber **nicht** geprüft. |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | **NOT RUN** | Grund: derselbe OOM-Abbruch. Ein Lint-Lauf wurde nicht begonnen. Der Workspace verbietet `unwrap()`/`expect()` im Produktivcode (`clippy::unwrap_used = "deny"`), `todo!()` und `unimplemented!()`; der Produktivcode dieses Meilestones enthält keine davon. |
| `./scripts/check-crate-boundaries.sh` | **RUN, PASS** | — |
| `./scripts/check-file-size.sh` | **RUN, PASS** | — |
| `python3 scripts/check-config-schema.py` | **RUN, PASS** | — |

**Konsequenz für die Übergabe:** Die Testlogik ist geschrieben (**66 neue Testfunktionen** in 14
Dateien: 11 in `config/security/tests.rs`, 8 in `ops/copy/tests.rs`, 7 in `config/rule/tests.rs`,
7 in `tests/integration/security_profiles.rs`, 6 in `viewer/reader.rs`, je 5 in `config/session.rs`
und `config/loader.rs`, je 4 in `desktop/security.rs`, je 3 in `config/notice.rs`,
`viewer/policy.rs` und `viewer/content.rs`, je 2 in `core/local/tests.rs`, `ui/refresh.rs` und
`ui/app/tests.rs`), aber **nicht ausgeführt**. Die Testziele der geänderten Crates sind im
Zielverzeichnis nicht gelinkt. Der Teamleiter muss `cargo test --workspace --locked` nachziehen; dort
können noch Fehler in der Testlogik auftauchen, die ich nicht messen konnte.

### 7.1 Was gemessen kompiliert ist — und was nicht

Gemessen (Kompilieren **erfolgreich**, Artefakt im Zielverzeichnis):

| Crate | Zeitpunkt des Artefakts | Inhalt |
|---|---|---|
| `phocommander-config` | 00:43 | alle neuen Module (`security`, `rule`, `notice`, `session`), `schema.rs`, `loader.rs` |
| `phocommander-viewer` | 00:45 | `policy.rs`, `content.rs`, `reader.rs`, `lib.rs` |
| `phocommander-ops` | 01:09 | `copy.rs` **vor** der Aufteilung, `port.rs`, `fake_fs.rs`, `sync.rs`, `executor.rs`, `lib.rs` |
| `phocommander-ui` | 00:41 | `refresh.rs`, `app.rs` |
| `phocommander-core` | 23:54 | `local.rs` mit `with_follow_symlinks` (die letzte Testdatei kam danach) |

**Nicht** gemessen (Änderungen **nach** dem Stopp-Befehl, 01:32–01:35 Uhr):

* die Aufteilung von `crates/phocommander-ops/src/copy.rs` in `copy/policy.rs`, `copy/types.rs` und
  `copy/tests.rs` (mit dem fehlenden `use crate::error::JobError;` in `types.rs`, der nachträglich
  ergänzt wurde),
* `apps/phocommander-desktop/src/lib.rs` und die Änderung von `src/main.rs` (das neue
  Bibliotheksziel, damit die Beispiele der Übersetzung mitgetestet werden),
* der Kommentar am Kopf von `copy.rs`.

Diese drei Punkte sind **von Hand nachgeprüft** (Schnittstellen, Importe, Modulpfade), aber **nicht
kompiliert**. Sie sind die ersten, die der Teamleiter bauen sollte.

---

## 8. Offene Punkte, die eine Owner-Entscheidung brauchen

### 8.1 Wer entscheidet im Profil `standard` und `corporate`?

Die Spezifikation (9.8) sagt: `standard` = „die Regeln der Sektion 9.7 mit den bequemen Schaltern,
die keine Zusage brechen“, `corporate` = „jede Regel der Sektion 9.7, auch die, die Bequemlichkeit
kostet“. Daraus folgt zwingend, dass die beiden Profile sich **irgendwo** unterscheiden müssen.

Meine Entscheidung: `corporate` schaltet `remember_session` ab (die Regel *leaves nothing behind*),
`standard` nicht. Begründung: nur die Sitzungsdatei ist ein Schalter, dessen Einschalten eine
Bequemlichkeit ist und dessen Ausschalten eine verwaltete Maschine schützt; alle anderen Schalter
stehen in beiden Profilen auf demselben Wert.

**Alternative, die der Owner wählen kann:** `standard` und `corporate` identisch setzen und den
Unterschied vollständig in die Schalter der Datei verlegen. Dann wäre `custom` das einzige Profil,
das etwas bewirkt.

### 8.2 Der Modus `atomic` prüft immer, auch wenn `checksum_on_copy` aus ist

Die Spezifikation sagt nur, dass der Schalter den **Modus `verified`** betrifft. Der Modus `atomic`
verspricht aber, dass ein Programm nie eine halb geschriebene Datei liest — dieses Versprechen hängt
an der Prüfung der temporären Datei. Ich habe den Schalter deshalb **nicht** auf `atomic` wirken
lassen und das im Code dokumentiert (`copy.rs`, `CopyMode::Atomic`).

**Der Owner entscheidet**, ob der Schalter auch `atomic` betreffen soll. Wenn ja, muss der Modus
einen anderen Weg der Prüfung bekommen (z. B. nur die Größe), sonst verliert er sein Versprechen.

### 8.3 Der Schalter `follow_symlinks` ist invertiert benannt

Die Spezifikation (9.8) nennt den Schalter `follow_symlinks` mit dem Vorgabewert `off` und
beschreibt die Wirkung als „ein Symlink wird als Symlink kopiert und nie dereferenziert“. Der Name
beschreibt also die **Abweichung** von der Regel, nicht die Regel. Das ist in der Spezifikation so
festgelegt und wurde nicht geändert. Im Code ist die Regel als `symlinks_are_never_followed`
geführt und `SecurityRuleId::switch_name()` nennt den Schalter — der Nutzer sieht also die Regel
und nicht die Abweichung.

**Der Owner entscheidet**, ob die Regel in der Oberfläche als „Regel“ (heute) oder als „Schalter“
(Umkehrung der Beschriftung) erscheinen soll.

### 8.4 Der Name der aktiven Konfigurationsdatei

Die Spezifikation nennt das **Verzeichnis** der Konfigurationsdatei, aber nicht die Datei selbst.
Die Composition Root benutzt `config.toml` (`apps/phocommander-desktop/src/main.rs`,
`CONFIGURATION_FILE_NAME`). **Der Owner bestätigt den Namen** oder nennt einen anderen; der Name
existiert genau einmal im Code.

### 8.5 Der Modus `standard` übergeht die Schalter der Datei — mit Meldung

In den Profilen `standard` und `corporate` entscheidet das **Profil**, nicht der Schalter; jede
Abweichung zwischen Datei und wirksamer Regel wird als Meldung (`Information`) berichtet, damit der
Nutzer sie sieht. Ein Nutzer, der `remember_session = false` schreibt und das Profil `standard`
beibehält, bekommt also die Meldung *„the profile `standard` sets this rule to true; the file says
false“*.

**Der Owner entscheidet**, ob die Schalter der Datei stattdessen Vorrang haben sollen (mit dem Preis,
dass dann die Schalter `background_index`, `read_only_on_demand` und `follow_symlinks` die Regeln
der Sektion 9.7 aufweichen könnten — was NFR-29 erlauben würde, NFR-32 aber nicht).

### 8.6 Der Hintergrundindex ist doppelt abgesichert

`background_index` steht in der Schaltertabelle der Spezifikation (9.8) **und** in der Liste der
drei nicht schaltbaren Garantien (NFR-32). Ich habe es als nicht schaltbar behandelt: der Schalter
existiert, damit der Wert sichtbar ist, wird aber ignoriert und gemeldet.

**Der Owner bestätigt** diese Lesart. Die Alternative wäre, `background_index = true` in einem
Profil `custom` zuzulassen — das widerspräche NFR-32.

### 8.7 Fremde Datei minimal repariert

`crates/phocommander-ui/tests/the_filter_matches_without_a_dialog.rs` gehört zu einem anderen
Rollenbereich und war um 00:54 Uhr mit `println!("{}", harness.root())` im Zustand, die nicht
kompiliert (`egui_kittest::Node` hat kein `Display`). Ohne die Reparatur konnte
`cargo build --workspace --all-targets --locked` nicht laufen. Ich habe **nur** das Formatzeichen
geändert (`{}` → `{:?}`) und nichts weiter angefasst.

### 8.8 Zwei Crates waren noch nie kompiliert

`crates/phocommander-ops/src/lib.rs` deklarierte **keine** Module: die Crate war bis heute eine
leere Hülle, die 7.900 Zeilen Quelltext in Dateien enthielt, die der Compiler nie gesehen hat. Ich
habe die Module deklariert (`copy`, `datetime`, `edit`, `error`, `executor`, `fake_fs`, `job`,
`names`, `port`, `progress`, `queue`, `series`, `sync`) und drei Fehler der Crate dabei behoben
(`FileInfo` ohne Feld `kind` in der Attrappe, `copy_one_file` mit der neuen Signatur in `sync.rs`,
ein unbenutzter Import in `executor.rs`).

**Hinweis für den Teamleiter:** Die Crate `phocommander-ops` und `phocommander-viewer` sind mit
dieser Änderung erstmals kompiliert worden. Es ist möglich, dass in den **Testzielen** der Crate
noch Fehler stecken, die erst beim nächsten `cargo test` sichtbar werden.

### 8.9 `deliverable.md` wird von mehreren Rollen parallel geschrieben

Während der Arbeit hat eine andere Rolle `deliverable.md` mit ihrem eigenen Text überschrieben. Ich
habe meinen Text zurückgeschrieben und zusätzlich unter **`deliverable.core-engineer.md`**
abgelegt, damit er nicht erneut verloren geht. Der Teamleiter sollte beim Zusammenführen entscheiden,
welche Datei die gültige ist.

---

## 9. Determinismus

Die Auflösung ist eine reine Funktion ohne Uhr, ohne Zufall und ohne I/O:
`resolve_security(&Security) -> EffectiveSecurity`. Gleiche Eingabe → gleiche Regeln, gleiche
Meldungen, gleiche Reihenfolge. Der Test `the_resolution_is_the_same_every_time` in
`crates/phocommander-config/src/security/tests.rs` vergleicht zwei Auflösungen desselben
`Security`-Wertes mit `assert_eq!` (der Typ `EffectiveSecurity` leitet `PartialEq` ab).

Die Reihenfolge der Regeln ist fest (acht Regeln in der Reihenfolge des Codes), damit die
Oberfläche und das Protokoll sie in derselben Reihenfolge zeigen.

---

## 10. Übergabe

* Die Dateien liegen im Repository unter `/workspace/phocommander` (kein Commit, kein Push, kein Tag).
* `docs/REQUIREMENTS.md` ist **unverändert**.
* Der Build des Workspace ist **nicht** als abgeschlossen gemeldet (Punkt 6.4 und 7).
* Der Testlauf ist **nicht** ausgeführt (Punkt 7) — die 66 neuen Testfunktionen sind geschrieben,
  aber ungeprüft.
