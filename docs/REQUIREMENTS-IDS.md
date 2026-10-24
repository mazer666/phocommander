# Requirement catalogue — Phocommander

This document decomposes [`docs/REQUIREMENTS.md`](REQUIREMENTS.md) into stable requirement
IDs. The specification is the binding source; this catalogue is the index into it.
Every row points at the section of the specification it was derived from, so a
reviewer can check a row against the specification without guessing.

## How to read the catalogue

| Column | Meaning |
|---|---|
| ID | `FR-xx` for functional requirements, `NFR-xx` for non-functional requirements. IDs are gapless and never reused. |
| Title | One line that states the requirement, not the solution. |
| Source | The section of `docs/REQUIREMENTS.md` the requirement was derived from, as `Section > Subsection`. |
| Milestone | The milestone that delivers the requirement, `M0` to `M9`, following the phase table of the specification. |
| Priority | `Must`, `Should` or `Could`. `Must` is binding; `Should` is binding unless a documented decision says otherwise; `Could` is a planned extension. |
| Verification | How the requirement is checked at acceptance. `automated` means a machine-checkable command or test; `manual` means a described manual test step. |

## Rules for this catalogue

1. An ID is assigned exactly once and is never reused for a different meaning.
2. A changed requirement keeps its ID and gets a note in the change log at the
   bottom of this document.
3. A requirement that is split keeps the old ID for the part that stays and gets
   a new ID for the new part; the old ID is then listed as superseded in the
   change log.
4. Every `Must` requirement of `M0`, `M1` and `M2` has an acceptance criterion
   in [`ACCEPTANCE-CRITERIA.md`](ACCEPTANCE-CRITERIA.md) with the ID of this
   catalogue.
5. The milestone column follows the phase table of the specification
   (`Roadmap > Phasen M0 bis M9` and `Projektplan > Meilensteine`).

---

## 1. Product principles

| ID | Title | Source | Milestone | Priority | Verification |
|---|---|---|---|---|---|
| FR-01 | No licence fee, no subscription, no in-app purchase, no closed feature | Produktmaximen > Kostenlos | M0 | Must | automated (CI gate) |
| FR-02 | Every function that exists in a version exists identically in every version; there is no "Pro" branch | Produktmaximen > Kostenlos | M0 | Must | manual (document review) |
| FR-03 | The source code builds the application without network access | Produktmaximen > Kostenlos | M0 | Must | automated (CI gate) |
| FR-04 | No advertising banner, no recommendation column, no message service, no forced account, no intrusive onboarding tour | Produktmaximen > Clutter-frei | M2 | Must | manual (visual inspection) |
| FR-05 | No in-app advertising, no sponsored hits in search or connection lists, no settings that are not needed | Produktmaximen > Clutter-frei | M2 | Must | manual (visual inspection) |
| FR-06 | No advertising, no tracking, no analytics, no telemetry, no usage profile, no third-party disclosure, no affiliate link | Produktmaximen > Werbefrei | M0 | Must | automated (dependency scan) |
| FR-07 | Local functions work without registration and without login | Produktmaximen > Werbefrei | M0 | Must | automated (CI gate) |
| FR-08 | The connection list contains only entries the user created themselves | Produktmaximen > Clutter-frei | M6 | Must | manual (operational test) |

## 2. Product vision

| ID | Title | Source | Milestone | Priority | Verification |
|---|---|---|---|---|---|
| FR-09 | Source and target are visible at the same time | Produktvision | M0 | Must | manual (visual inspection) |
| FR-10 | The complete operation is possible without a mouse | Produktvision | M2 | Must | manual (operational test) |
| FR-11 | The built-in viewer displays files without starting an external program | Produktvision | M1 | Must | automated (integration test) |
| FR-12 | An input line and a filter field accelerate jumping and selecting | Produktvision | M2 | Should | manual (operational test) |

## 3. Two-pane workspace

| ID | Title | Source | Milestone | Priority | Verification |
|---|---|---|---|---|---|
| FR-13 | Two independent directory windows stand next to each other by default or above one another | Funktionsumfang > Zwei-Fenster-Arbeitsfläche | M0 | Must | manual (visual inspection) |
| FR-14 | Both windows show path, file list and the columns name, size, date and type | Funktionsumfang > Zwei-Fenster-Arbeitsfläche | M0 | Must | manual (visual inspection) |
| FR-15 | The arrangement is switchable without a restart | Funktionsumfang > Zwei-Fenster-Arbeitsfläche | M1 | Must | automated (UI test) |
| FR-16 | Each window has a tab bar; each tab holds its own directory path, its own sorting and its own display settings | Funktionsumfang > Zwei-Fenster-Arbeitsfläche | M2 | Must | automated (UI test) |
| FR-17 | Tabs can be moved between the two windows | Funktionsumfang > Zwei-Fenster-Arbeitsfläche | M2 | Should | automated (UI test) |
| FR-18 | A directory tree can be switched on per window; the view type is kept when a directory is left | Funktionsumfang > Zwei-Fenster-Arbeitsfläche | M2 | Should | automated (UI test) |
| FR-19 | A third area can be switched on: a quick view that shows the content of the marked file without opening it | Funktionsumfang > Zwei-Fenster-Arbeitsfläche | M2 | Could | manual (operational test) |
| FR-20 | The workspace consists of exactly two panel objects, each with active tab, path, cursor position, sorting, selection set and view mode | Funktionsumfang > Zwei-Fenster-Arbeitsfläche | M1 | Must | automated (unit test) |
| FR-21 | Operations always act on the active panel; the transfer to the other panel is explicit | Funktionsumfang > Zwei-Fenster-Arbeitsfläche | M1 | Must | automated (integration test) |

## 4. File operations

| ID | Title | Source | Milestone | Priority | Verification |
|---|---|---|---|---|---|
| FR-22 | Copy, move, rename, delete and create directory are the basic operations | Funktionsumfang > Dateioperationen | M1 | Must | automated (integration test) |
| FR-23 | Every operation runs through a job queue in the background so that the surface stays usable | Funktionsumfang > Dateioperationen | M1 | Must | automated (integration test) |
| FR-24 | The user can continue working while a copy is running | Funktionsumfang > Dateioperationen | M1 | Must | automated (integration test) |
| FR-25 | Conflict behaviour is explicit and configurable: overwrite, skip, keep both, ask or cancel | Funktionsumfang > Dateioperationen | M1 | Must | automated (unit test) |
| FR-26 | Overwriting is not the default; the default conflict policy is "ask" | Funktionsumfang > Dateioperationen | M1 | Must | automated (config parse) |
| FR-27 | An undo area for operations is provided and is explicitly labelled as limited: undo of a copy means "delete the copy" | Funktionsumfang > Dateioperationen | M1 | Should | manual (operational test) |
| FR-28 | A job has type, status, progress, rate, cancel flag and error list; the queue is threadsafe and the surface reads a snapshot | Systemarchitektur > Auftragsverarbeitung | M1 | Must | automated (unit test) |
| FR-29 | Several jobs run in parallel up to a configurable limit, by default the number of available cores; one panel half does not block while the other works | Systemarchitektur > Auftragsverarbeitung | M1 | Must | automated (integration test) |

## 5. Multiple rename

| ID | Title | Source | Milestone | Priority | Verification |
|---|---|---|---|---|---|
| FR-30 | The multiple rename dialog has three zones: the list of selected files, a table of original name and new name, and the rule definition | Funktionsumfang > Mehrfaches Umbenennen | M1 | Must | automated (UI test) |
| FR-31 | The rule evaluation combines placeholders, replace and counter in this order | Funktionsumfang > Mehrfaches Umbenennen | M1 | Must | automated (unit test) |
| FR-32 | The placeholders name, ext, parent, n, date, time, size, hash, rand, case, lower, upper and title are available | Funktionsumfang > Mehrfaches Umbenennen | M1 | Must | automated (unit test) |
| FR-33 | The replace rule offers search text and replacement text, optionally as a regular expression | Funktionsumfang > Mehrfaches Umbenennen | M1 | Should | automated (unit test) |
| FR-34 | The counter rule offers start value, step, digit count with leading zeros and the order of the counter sequence | Funktionsumfang > Mehrfaches Umbenennen | M1 | Must | automated (unit test) |
| FR-35 | The dialog shows a live preview of the result | Funktionsumfang > Mehrfaches Umbenennen | M1 | Must | automated (UI test) |
| FR-36 | Conflicts are marked in colour and listed in a conflict jump list traversable with F3 and Shift+F3 | Funktionsumfang > Mehrfaches Umbenennen | M1 | Must | automated (UI test) |
| FR-37 | Rules can be saved as named templates and applied again | Funktionsumfang > Mehrfaches Umbenennen | M1 | Must | automated (integration test) |
| FR-38 | The dialog recognises season and episode number in an existing file name using the pattern SxxEyy and offers the rest as episode title | Funktionsumfang > Mehrfaches Umbenennen | M1 | Must | automated (unit test) |
| FR-39 | If no pattern is found, the dialog asks for season and episode and remembers the last input as the pre-selection for the next file | Funktionsumfang > Mehrfaches Umbenennen | M1 | Must | automated (unit test) |
| FR-40 | The series name comes from the parent directory by default, is always displayed and is overridable | Funktionsumfang > Mehrfaches Umbenennen | M1 | Must | automated (unit test) |
| FR-41 | The target format is serie, season and episode zero padded to two digits; the width grows automatically to three when a three-digit episode is in the selection | Funktionsumfang > Mehrfaches Umbenennen | M1 | Must | automated (unit test) |
| FR-42 | The episode title is taken over unchanged; characters that Windows forbids are reported before the rename and a replacement is suggested instead of being applied silently | Funktionsumfang > Mehrfaches Umbenennen | M1 | Must | automated (unit test) |
| FR-43 | A preset set of templates ships with the application: series, camera, podcast, invoice and music; each is a TOML file in the configuration directory and is user-adjustable | Funktionsumfang > Mehrfaches Umbenennen | M1 | Must | automated (config parse) |
| FR-44 | Before renaming, the dialog shows a preview of the new names of all selected files side by side, with the recognised fields as a column of their own | Funktionsumfang > Mehrfaches Umbenennen | M1 | Must | automated (UI test) |

## 6. File viewer

| ID | Title | Source | Milestone | Priority | Verification |
|---|---|---|---|---|---|
| FR-45 | The viewer opens in an additional area without leaving the application | Funktionsumfang > Dateibetrachter | M1 | Must | automated (integration test) |
| FR-46 | The viewer shows text and source code with charset detection, UTF-8 and UTF-16 and line-end handling | Funktionsumfang > Dateibetrachter | M1 | Must | automated (unit test) |
| FR-47 | The viewer shows a hex dump with offset, hex, ASCII and OR group | Funktionsumfang > Dateibetrachter | M1 | Must | automated (unit test) |
| FR-48 | The viewer shows images in the formats PNG, JPEG, GIF, WebP, BMP and TIFF | Funktionsumfang > Dateibetrachter | M1 | Must | automated (unit test) |
| FR-49 | The viewer lists the content of an archive and extracts a single file | Funktionsumfang > Dateibetrachter | M1 | Must | automated (integration test) |
| FR-50 | The viewer shows metadata: size, type, timestamp and checksum | Funktionsumfang > Dateibetrachter | M1 | Must | automated (unit test) |
| FR-51 | The viewer shows PDF documents with text layer and page navigation | Funktionsumfang > Dateibetrachter | M2 | Should | automated (integration test) |
| FR-52 | The viewer highlights source code syntax | Funktionsumfang > Dateibetrachter | M2 | Should | automated (integration test) |
| FR-53 | The viewer shows audio and video with metadata, still image and playback | Funktionsumfang > Dateibetrachter | M9 | Could | manual (operational test) |
| FR-54 | The viewer shows office documents with metadata only, without a full text representation | Funktionsumfang > Dateibetrachter | M9 | Could | manual (operational test) |
| FR-55 | The viewer is read-only | Funktionsumfang > Dateibetrachter | M1 | Must | automated (unit test) |
| FR-56 | The About dialog names the actually supported formats honestly instead of an inflated number | Funktionsumfang > Dateibetrachter | M1 | Should | manual (document review) |
| FR-57 | F3 searches forward in the viewer and Shift+F3 searches backward | Systemarchitektur > Tastatursteuerung | M1 | Must | automated (UI test) |
| FR-58 | A separate text editor is a later expansion stage | Funktionsumfang > Dateibetrachter | M9 | Could | manual (operational test) |

## 7. Archive handling

| ID | Title | Source | Milestone | Priority | Verification |
|---|---|---|---|---|---|
| FR-59 | Archives appear as normal entries in the file list | Funktionsumfang > Archivbehandlung | M1 | Must | automated (integration test) |
| FR-60 | A double click on an archive opens it as a virtual directory whose files can be listed, viewed and copied into the other pane without prior extraction | Funktionsumfang > Archivbehandlung | M1 | Must | automated (integration test) |
| FR-61 | Supported formats are ZIP, TAR with GZip, BZip2, XZ and Zstd, and 7z; reading and writing are possible, archives are created encrypted | Funktionsumfang > Archivbehandlung | M1 | Must | automated (integration test) |
| FR-62 | RAR, WIM, CAB, ARJ and LZH stay out of scope because they need proprietary or licence-bound libraries | Funktionsumfang > Archivbehandlung | M1 | Must | automated (unit test) |
| FR-63 | All extractions run through a hardened path resolution | Funktionsumfang > Archivbehandlung | M1 | Must | automated (integration test) |

## 8. Search and synchronisation

| ID | Title | Source | Milestone | Priority | Verification |
|---|---|---|---|---|---|
| FR-64 | The file search searches by name, size, date and content; text, hex and binary search are possible and Unicode text is handled correctly | Funktionsumfang > Suche und Synchronisation | M6 | Must | automated (integration test) |
| FR-65 | The search operators AND, OR and NOT are available, as is the search in archives | Funktionsumfang > Suche und Synchronisation | M6 | Should | automated (integration test) |
| FR-66 | The content search runs in a background process with a growing hit list; a hit can be taken over into a panel | Funktionsumfang > Suche und Synchronisation | M6 | Should | automated (integration test) |
| FR-67 | The directory comparison places two directories side by side and reports differences in content, date and size | Funktionsumfang > Suche und Synchronisation | M2 | Should | automated (integration test) |
| FR-68 | The synchronisation transfers differences in both directions and holds profile options that include or exclude directories | Funktionsumfang > Suche und Synchronisation | M6 | Should | automated (integration test) |
| FR-69 | A plan is shown before the execution, not only a dialog with "OK" | Funktionsumfang > Suche und Synchronisation | M6 | Must | automated (UI test) |

## 9. Favourites, file drop and tags

| ID | Title | Source | Milestone | Priority | Verification |
|---|---|---|---|---|---|
| FR-70 | Folder favourites and application favourites are callable from the icon bar or a keybinding | Funktionsumfang > Favoriten, Dateiablage und Tags | M2 | Should | automated (UI test) |
| FR-71 | The file drop collects frequently needed files in one place, independent of where they were found | Funktionsumfang > Favoriten, Dateiablage und Tags | M2 | Could | manual (operational test) |
| FR-72 | Tags on files and folders are a later expansion stage | Funktionsumfang > Favoriten, Dateiablage und Tags | M9 | Could | manual (operational test) |

## 10. Input line and tools

| ID | Title | Source | Milestone | Priority | Verification |
|---|---|---|---|---|---|
| FR-73 | An input line at the lower edge accepts paths and shell commands | Funktionsumfang > Eingabezeile und Werkzeuge | M2 | Must | automated (UI test) |
| FR-74 | A command line history with Ctrl+Down and a command output window are part of the scope | Funktionsumfang > Eingabezeile und Werkzeuge | M2 | Could | manual (operational test) |
| FR-75 | The menu bar follows the platform conventions: in the system menu bar on macOS, in the window on Windows and Linux; on macOS the standard roles sit in the expected places | Funktionsumfang > Eingabezeile und Werkzeuge | M2 | Must | manual (visual inspection) |

## 11. Input models and operating contexts

| ID | Title | Source | Milestone | Priority | Verification |
|---|---|---|---|---|---|
| FR-76 | Every operable function is a named action with a stable action ID; the input layer translates physical input into action IDs | Eingabemodelle und Bedienkontexte > Das Aktionsmodell | M0 | Must | automated (config parse) |
| FR-77 | The surface layer knows action IDs only and does not know whether an action was triggered by key, click, swipe, D-pad key or controller trigger | Eingabemodelle und Bedienkontexte > Das Aktionsmodell | M1 | Must | automated (unit test) |
| FR-78 | The action catalogue lives in config/keymap.toml and is referenced by the configuration file; docs/KEYMAP.md documents every action with default, context and platform variant | Eingabemodelle und Bedienkontexte > Das Aktionsmodell | M0 | Must | automated (config parse) |
| FR-79 | The default binding is stored separately per operating context | Eingabemodelle und Bedienkontexte > Bedienkontexte | M0 | Must | automated (config parse) |
| FR-80 | Five operating contexts are supported: keyboard, pointer, touch, dpad and controller | Eingabemodelle und Bedienkontexte > Bedienkontexte | M4 | Must | automated (integration test) |
| FR-81 | The layouts differ structurally per context: two panes visible for keyboard and pointer, stacked for touch, one pane plus overlay for dpad, spatial arrangement with ray pointer for controller | Eingabemodelle und Bedienkontexte > Adaptive Layouts je Kontext | M4 | Must | manual (visual inspection) |
| FR-82 | In the touch context the row height is at least 44 points, dialogs appear as a sheet from below and hover effects are absent | Eingabemodelle und Bedienkontexte > Adaptive Layouts je Kontext | M4 | Must | manual (visual inspection) |
| FR-83 | In the dpad context one pane is visible, a second pane appears as an overlay when an action needs a target, the type is scaled for a viewing distance of about three metres and the operating hints are visible | Eingabemodelle und Bedienkontexte > Adaptive Layouts je Kontext | M8 | Must | manual (visual inspection) |
| FR-84 | The action palette is a searchable list of all actions available in the current context; it opens with the menu or play/pause key in the dpad context and is fully operable with the direction keys and OK | Eingabemodelle und Bedienkontexte > Die Aktionspalette als Ersatz für F-Tasten | M8 | Must | automated (UI test) |
| FR-85 | The palette filters by context: in the dpad context only actions that can be executed without a mouse and without a modifier key appear | Eingabemodelle und Bedienkontexte > Die Aktionspalette als Ersatz für F-Tasten | M8 | Must | automated (UI test) |
| FR-86 | The action palette is available in every context, on the desktop as "find actions" | Eingabemodelle und Bedienkontexte > Die Aktionspalette als Ersatz für F-Tasten | M8 | Should | automated (UI test) |
| FR-87 | Focus and selection are led as separate states in the dpad and controller contexts; navigating moves the focus without changing the selection, selecting changes the selection without losing the focus | Eingabemodelle und Bedienkontexte > Fokus und Auswahl als getrennte Zustände | M8 | Must | automated (unit test) |
| FR-88 | The dpad context offers an explicit selection mode in which OK toggles the marking and the direction keys move the focus; the mode is visibly indicated and can be left at any time | Eingabemodelle und Bedienkontexte > Fokus und Auswahl als getrennte Zustände | M8 | Must | automated (UI test) |
| FR-89 | Focus is a visible and permanently recognisable state on platforms without a pointer | Eingabemodelle und Bedienkontexte > Fokus und Auswahl als getrennte Zustände | M4 | Must | manual (visual inspection) |
| FR-90 | Every action that is reachable in the keyboard context must be reachable in the dpad context through the action palette; the check is documented in a test protocol | Eingabemodelle und Bedienkontexte > Fire TV und Google TV | M8 | Must | manual (operational test) |
| FR-91 | The application does not depend on a menu key of the remote control; back navigates back and leaves the application without endless loops | Eingabemodelle und Bedienkontexte > Fire TV und Google TV | M8 | Must | automated (UI test) |
| FR-92 | Gamepad events do not come from winit; the controller binding on the desktop is a separate task that uses a platform-native library | Eingabemodelle und Bedienkontexte > Das Aktionsmodell | M7 | Must | automated (integration test) |
| FR-93 | The context "auto" detects the input device at run time and selects keyboard, pointer, touch, dpad or controller | Systemarchitektur > Konfigurationsdatei | M4 | Should | automated (integration test) |
| FR-94 | Meta Quest is operated as a flat panel on the Android version through the controller context; there is no immersive VR port | Eingabemodelle und Bedienkontexte > Meta Quest als Android-Konfiguration | M7 | Should | manual (operational test) |
| FR-95 | The limitation of the panel mode — no keyboard — is stated in the documentation | Eingabemodelle und Bedienkontexte > Meta Quest als Android-Konfiguration | M7 | Must | manual (document review) |

## 12. Network and cloud access

| ID | Title | Source | Milestone | Priority | Verification |
|---|---|---|---|---|---|
| FR-96 | Network access is a source in the same abstraction; SMB, WebDAV, SFTP and S3 deliver the same folder content as the local file system | Netzwerk- und Cloud-Zugriffe > Quellen statt Funktionen | M6 | Must | automated (integration test) |
| FR-97 | The operation is identical for all sources: copy, move, rename, delete, view, search | Netzwerk- und Cloud-Zugriffe > Quellen statt Funktionen | M6 | Must | automated (integration test) |
| FR-98 | The protocols SFTP, FTP and FTPS, SMB 2 and 3, WebDAV, S3-compatible and OAuth 2.0 are supported | Netzwerk- und Cloud-Zugriffe > Protokollunterstützung | M6 | Must | automated (integration test) |
| FR-99 | The WebDAV client is written in the project itself instead of using an LGPL library | Netzwerk- und Cloud-Zugriffe > Protokollunterstützung | M6 | Must | manual (document review) |
| FR-100 | SMB1 is not implemented | Netzwerk- und Cloud-Zugriffe > SMB-Sicherheitsregeln | M6 | Must | automated (integration test) |
| FR-101 | NTLMv1 is not used; where NTLM is needed it is NTLMv2 | Netzwerk- und Cloud-Zugriffe > SMB-Sicherheitsregeln | M6 | Must | automated (integration test) |
| FR-102 | SMB signing is required when the server supports it | Netzwerk- und Cloud-Zugriffe > SMB-Sicherheitsregeln | M6 | Must | automated (integration test) |
| FR-103 | SMB transport encryption is preferred and the credentials never lie in the configuration file | Netzwerk- und Cloud-Zugriffe > SMB-Sicherheitsregeln | M6 | Must | automated (integration test) |
| FR-104 | A connection attempt against a server that offers only SMB1 or only NTLMv1 fails with an understandable message | Netzwerk- und Cloud-Zugriffe > SMB-Sicherheitsregeln | M6 | Must | automated (integration test) |
| FR-105 | Phocommander runs no own intermediary server; every login runs directly between the application and the provider and the refresh token lies in the keychain | Netzwerk- und Cloud-Zugriffe > Cloud-Zugriff | M7 | Must | automated (dependency scan) |
| FR-106 | Credentials lie in the keychain of the operating system; on platforms without a keychain the application falls back to the app sandbox and points out that the data is then not protected by the operating system | Netzwerk- und Cloud-Zugriffe > Verbindungsdaten und Protokollierung | M6 | Must | manual (operational test) |
| FR-107 | Connections are managed in a list the user created; connection names, server addresses and user names are personal data and do not appear in the log | Netzwerk- und Cloud-Zugriffe > Verbindungsdaten und Protokollierung | M6 | Must | automated (integration test) |
| FR-108 | The log names protocol, timestamp, result and duration, never the server and never the user | Netzwerk- und Cloud-Zugriffe > Verbindungsdaten und Protokollierung | M6 | Must | automated (integration test) |
| FR-109 | For HTTPS sources the server certificate is checked strictly, without an opt-out; a self-signed certificate can be stored deliberately with a PIN-protected exception for exactly that host | Nicht-funktionale Anforderungen > Sicherheit | M6 | Must | automated (integration test) |
| FR-110 | The client reads in a fixed budget, aborts and reports the error when a server exceeds the expected response size | Nicht-funktionale Anforderungen > Sicherheit | M6 | Must | automated (integration test) |
| FR-111 | An older protocol version is not silently accepted: SMB1 and NTLMv1 are rejected, a WebDAV server without TLS is rejected when HTTPS is not reachable | Nicht-funktionale Anforderungen > Sicherheit | M6 | Must | automated (integration test) |
| FR-112 | Directory names injected by a server are normalised, checked for invalid characters and shortened in the surface without losing the real name | Nicht-funktionale Anforderungen > Sicherheit | M6 | Must | automated (unit test) |
| FR-113 | Every network operation has a time limit; a server that does not finish a response leads to an abort with an understandable message | Nicht-funktionale Anforderungen > Sicherheit | M6 | Must | automated (integration test) |
| FR-114 | The behaviour on connection problems is configurable: "retry" tries again, "fail" aborts; the timeouts apply per connection establishment | Systemarchitektur > Konfigurationsdatei | M6 | Must | automated (config parse) |

## 13. System architecture

| ID | Title | Source | Milestone | Priority | Verification |
|---|---|---|---|---|---|
| FR-115 | The workspace uses flat crates, apps for the binaries and xtask for build helpers | Systemarchitektur > Crate-Struktur | M0 | Must | automated (CI gate) |
| FR-116 | The dependency direction is apps to platform to crates; inside crates no crate depends on a sibling of the same level | Systemarchitektur > Crate-Struktur | M0 | Must | automated (CI gate) |
| FR-117 | The crate boundary rule is checked mechanically by a script that exits with code 1 on a violation, and a negative test proves that the gate has teeth | Systemarchitektur > Crate-Struktur | M0 | Must | automated (CI gate) |
| FR-118 | The core is free of UI dependencies: no windows, no crates of the UI stack, no platform-specific APIs | Systemarchitektur > Crate-Struktur | M0 | Must | automated (CI gate) |
| FR-119 | The directory content is held in a state object outside the egui context; only the visible slice is drawn | Systemarchitektur > Benutzerschnittstelle und Plattformanpassung | M1 | Must | automated (unit test) |
| FR-120 | Viewer and archive adapters are registered through a trait; a new adapter is a new implementation plus one registration line | Systemarchitektur > Viewer- und Archivregistrierung | M1 | Must | automated (unit test) |
| FR-121 | There is exactly one central configuration file in TOML format; its location follows the operating system conventions | Systemarchitektur > Konfigurationsdatei | M0 | Must | automated (config parse) |
| FR-122 | The configuration file is fully commented because it is the first contact point for users and contributors | Systemarchitektur > Konfigurationsdatei | M0 | Must | automated (config parse) |
| FR-123 | The configuration file carries a schema_version; on incompatible changes the old file is migrated and the original is kept as a .bak file | Systemarchitektur > Konfigurationsdatei | M0 | Must | automated (unit test) |
| FR-124 | Import and export of a configuration are provided so that settings can be transferred between machines | Systemarchitektur > Konfigurationsdatei | M2 | Could | manual (operational test) |
| FR-125 | The repository structure follows the crate structure; the workspace is a single Cargo workspace with shared dependency versions and a pinned toolchain | Repository- und Dokumentationsorganisation > Ablagestruktur | M0 | Must | automated (CI gate) |

## 14. Platform matrix

| ID | Title | Source | Milestone | Priority | Verification |
|---|---|---|---|---|---|
| FR-126 | macOS is delivered without app sandbox, with notarisation, hardened runtime and a first-run page that explains the required permissions | Plattform-Matrix > macOS | M3 | Must | manual (operational test) |
| FR-127 | The use of full disk access is neither forced nor circumvented; the application handles the refusal | Plattform-Matrix > macOS | M3 | Must | manual (operational test) |
| FR-128 | iPadOS is not a file system browser; it is a viewer and organiser for root directories the user shares explicitly | Plattform-Matrix > iPadOS | M5 | Must | manual (operational test) |
| FR-129 | A structure for permanent access is stored as security-scoped bookmarks; the user can revoke the access in the settings and the application handles the error cleanly | Plattform-Matrix > iPadOS | M5 | Must | manual (operational test) |
| FR-130 | The two-pane surface stays sensible on the iPad in split view; touch operates the selection and the keyboard remains the main control | Plattform-Matrix > iPadOS | M5 | Should | manual (visual inspection) |
| FR-131 | The Android version works in SAF mode without the special permission and offers full access as a clearly marked optional extension | Plattform-Matrix > Android | M4 | Must | manual (operational test) |
| FR-132 | Android declares MANAGE_EXTERNAL_STORAGE in the manifest, explains the core functionality and asks the user to enable the permission | Plattform-Matrix > Android | M4 | Must | manual (document review) |
| FR-133 | The Windows manifest sets longPathAware and path length errors are phrased understandably | Plattform-Matrix > Windows, Linux und BSD | M3 | Must | automated (integration test) |
| FR-134 | The case in which the explorer does not show a file that Phocommander shows is caught and reported understandably | Plattform-Matrix > Windows, Linux und BSD | M3 | Should | manual (operational test) |
| FR-135 | Linux and BSD share the code path; Wayland is preferred and X11 is the fallback | Plattform-Matrix > Windows, Linux und BSD | M3 | Must | automated (CI gate) |
| FR-136 | The build on FreeBSD and NetBSD is checked in the CI; the functional tests are carried out manually | Plattform-Matrix > Windows, Linux und BSD | M3 | Must | automated (CI gate) |
| FR-137 | The TV build declares no touchscreen requirement, sets CATEGORY_LEANBACK_LAUNCHER and carries a 320 by 180 pixel banner | Eingabemodelle und Bedienkontexte > Fire TV und Google TV | M8 | Must | manual (document review) |
| FR-138 | The pointer is never the primary means of operation on the TV | Eingabemodelle und Bedienkontexte > Fire TV und Google TV | M8 | Must | manual (visual inspection) |

## 15. Test and quality strategy

| ID | Title | Source | Milestone | Priority | Verification |
|---|---|---|---|---|---|
| FR-139 | Every public function has at least one unit test | Test- und Qualitätsstrategie > Testpyramide | M0 | Must | automated (CI gate) |
| FR-140 | Integration tests cover the crate boundaries: copy through a temporary directory, extraction of an archive with traversal entries, loading of a configuration file with an unknown field | Test- und Qualitätsstrategie > Testpyramide | M1 | Must | automated (integration test) |
| FR-141 | Property tests check invariants that cannot be enumerated: path resolution never leaves the destination, the rename rule is deterministic, the key resolution decides deterministically | Test- und Qualitätsstrategie > Testpyramide | M1 | Must | automated (property test) |
| FR-142 | Fuzz tests check the parsers: archive header, image header, charset detection; a fuzz run belongs to the nightly CI | Test- und Qualitätsstrategie > Testpyramide | M1 | Must | automated (fuzz test) |
| FR-143 | UI tests check the keyboard operation deterministically, without pixel-wise screenshots | Test- und Qualitätsstrategie > Testpyramide | M1 | Must | automated (UI test) |
| FR-144 | Performance measurements with criterion are part of the gate; the values of the performance requirement are the thresholds | Test- und Qualitätsstrategie > Testpyramide | M1 | Must | automated (benchmark) |
| FR-145 | The CI runs as a matrix over Linux, Windows and macOS and additionally checks the cross-compilation to aarch64-linux-android and aarch64-apple-ios and the build for FreeBSD | Test- und Qualitätsstrategie > Continuous Integration | M0 | Must | automated (CI gate) |
| FR-146 | Every CI run covers cargo fmt --check, cargo clippy --all-targets with -D warnings, cargo test --workspace, cargo-deny, cargo-audit and cargo build --release | Test- und Qualitätsstrategie > Continuous Integration | M0 | Must | automated (CI gate) |
| FR-147 | The tool versions are pinned because the same configuration is read differently per version | Test- und Qualitätsstrategie > Continuous Integration | M0 | Must | automated (CI gate) |
| FR-148 | For every hardening — path traversal, decompression bomb, header limit — a negative test exists that actually performs the attack | Test- und Qualitätsstrategie > Continuous Integration | M1 | Must | automated (integration test) |
| FR-149 | The gate must not be green when a rule is violated | Test- und Qualitätsstrategie > Continuous Integration | M0 | Must | automated (CI gate) |
| FR-150 | A milestone counts as reached only when its acceptance criteria are met and the acceptance is documented under docs/acceptance | Projektplan > Grundregeln des Ablaufs | M0 | Must | manual (document review) |
| FR-151 | The core comes before the surface; a surface function without core support is not started | Projektplan > Grundregeln des Ablaufs | M0 | Must | manual (document review) |
| FR-152 | If a basic assumption does not hold, the affected milestone is stopped, the assumption is corrected in an ADR or in the specification and the work is repeated | Projektplan > Rücksprungregel | M0 | Must | manual (document review) |

## 16. Documentation and repository organisation

| ID | Title | Source | Milestone | Priority | Verification |
|---|---|---|---|---|---|
| FR-153 | The documentation landscape contains the architecture, the ADRs, the user guide in both languages, the security documents, the glossary, the acceptances, the reviews, the guides and the templates | Repository- und Dokumentationsorganisation > Ablagestruktur | M0 | Must | manual (document review) |
| FR-154 | The ADRs follow the pattern status, date, context, decision, consequences, alternatives; the first series covers the framework choice, the crate boundaries, the error hierarchy and the security strategy for archive extraction | Repository- und Dokumentationsorganisation > Ablagestruktur | M0 | Must | manual (document review) |
| FR-155 | AGENTS.md describes the working rules for AI-assisted contributions | Repository- und Dokumentationsorganisation > Ablagestruktur | M0 | Must | manual (document review) |
| FR-156 | CONTRIBUTING.md summarises the rules, SECURITY.md describes the threat model and the reporting path, PRIVACY.md holds the zero-telemetry pledge | Repository- und Dokumentationsorganisation > Beitragsregeln und Governance | M0 | Must | manual (document review) |
| FR-157 | CODE_OF_CONDUCT.md follows the Contributor Covenant | Repository- und Dokumentationsorganisation > Beitragsregeln und Governance | M0 | Must | manual (document review) |
| FR-158 | The repository stands under the dual licence MIT OR Apache-2.0; both files lie in the repository root and every source file carries a short SPDX header | Repository- und Dokumentationsorganisation > Lizenzierung | M0 | Must | automated (CI gate) |

## 17. Configuration, keymap and requirement catalogue

| ID | Title | Source | Milestone | Priority | Verification |
|---|---|---|---|---|---|
| FR-159 | The central configuration file contains the sections schema_version, general, appearance, panels, keymap, viewer, archives, privacy, advanced, input and remote with the specified default values | Systemarchitektur > Konfigurationsdatei | M0 | Must | automated (config parse) |
| FR-160 | Every section and every single switch of the configuration file carries a comment that explains what it does | Systemarchitektur > Konfigurationsdatei | M0 | Must | automated (config parse) |
| FR-161 | Every action has a stable action ID in dotted notation and exactly one binding per operating context | Eingabemodelle und Bedienkontexte > Das Aktionsmodell | M0 | Must | automated (config parse) |
| FR-162 | The binding table of the keyboard context matches the key table of the specification | Systemarchitektur > Tastatursteuerung | M0 | Must | automated (config parse) |
| FR-163 | In the dpad and controller contexts the actions refer to the action palette because those devices have no function keys | Eingabemodelle und Bedienkontexte > Die Aktionspalette als Ersatz für F-Tasten | M0 | Must | automated (config parse) |
| FR-164 | The shipped rename templates series, camera, podcast, invoice and music exist as TOML files in the configuration directory | Funktionsumfang > Mehrfaches Umbenennen | M0 | Must | automated (config parse) |
| FR-165 | The requirement catalogue decomposes the specification into gapless FR and NFR IDs, each row naming a concrete section of the specification | Test- und Qualitätsstrategie > Abnahmekriterien | M0 | Must | automated (config parse) |
| FR-166 | Every FR of M0, M1 and M2 has an acceptance criterion with an exact command or an exact operation sequence and a measurable pass condition | Abnahmeregelungen > Aufbau einer Abnahme | M0 | Must | automated (config parse) |
| FR-167 | The acceptance template carries the sections functionality, technique, performance, guidance, security and privacy, interoperability, known limitations and decision | Abnahmeregelungen > Aufbau einer Abnahme | M0 | Must | manual (document review) |

---

## Non-functional requirements

### 18. Security

| ID | Title | Source | Milestone | Priority | Verification |
|---|---|---|---|---|---|
| NFR-01 | The trust boundary runs between the process and the file system; inputs from the file system and from archives are hostile by assumption | Nicht-funktionale Anforderungen > Sicherheit | M1 | Must | automated (integration test) |
| NFR-02 | No extract function of an archive crate is used; the entries are iterated and written by the project itself, every target path is canonicalised and checked against the target directory, and no symlink is created from archive material | Nicht-funktionale Anforderungen > Sicherheit | M1 | Must | automated (integration test) |
| NFR-03 | The minimum version of the zip crate is 9.0.5 | Nicht-funktionale Anforderungen > Sicherheit | M1 | Must | automated (CI gate) |
| NFR-04 | The extraction has a limit for the total size, a limit for the number of entries and a limit for the size of a single entry; the operation aborts when a limit is exceeded | Nicht-funktionale Anforderungen > Sicherheit | M1 | Must | automated (integration test) |
| NFR-05 | Where the check of a target and the access to it can fall apart, the access runs descriptor-based or the check is repeated immediately before the access; the remaining race is named in the threat model | Nicht-funktionale Anforderungen > Sicherheit | M1 | Must | manual (document review) |
| NFR-06 | When copying recursively, symlinks are copied as symlinks and not dereferenced; when deleting, they are unlinked and not deleted recursively | Nicht-funktionale Anforderungen > Sicherheit | M1 | Must | automated (integration test) |
| NFR-07 | Secure deletion follows NIST SP 800-88 Rev. 2 with the levels Clear, Purge and Destroy; Clear is the standard level and is called "secure deletion" | Nicht-funktionale Anforderungen > Sicherheit | M1 | Must | automated (integration test) |
| NFR-08 | The limitation of overwriting deletion on flash media is stated in the surface and not only in the manual | Nicht-funktionale Anforderungen > Sicherheit | M1 | Must | manual (visual inspection) |
| NFR-09 | The trash is the standard path for deletion; secure deletion is a deliberate additional option | Nicht-funktionale Anforderungen > Sicherheit | M1 | Must | automated (integration test) |
| NFR-10 | Every remote source is hostile; names, size information, timestamps and contents delivered by the server are treated as unverified | Nicht-funktionale Anforderungen > Sicherheit | M6 | Must | automated (integration test) |
| NFR-11 | Only libraries with a permissive licence are used; crates with a copyleft effect are excluded | Nicht-funktionale Anforderungen > Sicherheit | M0 | Must | automated (CI gate) |
| NFR-12 | The CI checks every dependency with cargo-deny and cargo-audit in the pinned tool versions | Nicht-funktionale Anforderungen > Sicherheit | M0 | Must | automated (CI gate) |

### 19. Privacy

| ID | Title | Source | Milestone | Priority | Verification |
|---|---|---|---|---|---|
| NFR-13 | No telemetry, no analytics, no usage counter, no advertising and no cloud connection | Nicht-funktionale Anforderungen > Datenschutz | M0 | Must | automated (dependency scan) |
| NFR-14 | No usage profile, no crash report and no content index is stored; the application writes only what the user configures | Nicht-funktionale Anforderungen > Datenschutz | M0 | Must | automated (dependency scan) |
| NFR-15 | Connection data for SFTP and FTP is not stored in cleartext in the configuration file but in the keychain of the operating system; on platforms without a keychain the application falls back to the app sandbox and points out the missing protection | Nicht-funktionale Anforderungen > Datenschutz | M6 | Must | automated (integration test) |
| NFR-16 | Log data contains no user name, no server name and no IP address; personal fields are masked when logging is needed | Nicht-funktionale Anforderungen > Datenschutz | M6 | Must | automated (integration test) |
| NFR-17 | The log level is configurable and defaults to "warn" | Systemarchitektur > Konfigurationsdatei | M0 | Should | automated (config parse) |

### 20. Performance

| ID | Title | Source | Milestone | Priority | Verification |
|---|---|---|---|---|---|
| NFR-18 | A directory list with up to 10,000 entries is drawn in under 100 ms | Nicht-funktionale Anforderungen > Leistung | M1 | Must | automated (benchmark) |
| NFR-19 | A directory change with a loaded listing is under 16 ms | Nicht-funktionale Anforderungen > Leistung | M1 | Must | automated (benchmark) |
| NFR-20 | A key press reaches a visible reaction in under 50 ms | Nicht-funktionale Anforderungen > Leistung | M1 | Must | automated (benchmark) |
| NFR-21 | The start-up to the operable surface is under 800 ms on macOS, warm | Nicht-funktionale Anforderungen > Leistung | M1 | Must | automated (benchmark) |
| NFR-22 | The memory demand in idle with two panes open is under 120 MB | Nicht-funktionale Anforderungen > Leistung | M1 | Must | automated (benchmark) |
| NFR-23 | A sequential file copy reaches at least 80 % of the sequential read and write bandwidth | Nicht-funktionale Anforderungen > Leistung | M1 | Must | automated (benchmark) |
| NFR-24 | A search over 100,000 file names takes under 30 s | Nicht-funktionale Anforderungen > Leistung | M6 | Must | automated (benchmark) |
| NFR-25 | An archive listing with 5,000 entries takes under 500 ms | Nicht-funktionale Anforderungen > Leistung | M1 | Must | automated (benchmark) |
| NFR-26 | Background operations are mandatorily asynchronous; no process that can take longer than 50 ms blocks the surface | Nicht-funktionale Anforderungen > Leistung | M1 | Must | automated (integration test) |
| NFR-27 | Every operation reports progress with a rate and is cancellable | Nicht-funktionale Anforderungen > Leistung | M1 | Must | automated (integration test) |

### 21. Accessibility

| ID | Title | Source | Milestone | Priority | Verification |
|---|---|---|---|---|---|
| NFR-28 | The operation is completely possible by keyboard, with a visible focus marking and a deterministic focus order | Nicht-funktionale Anforderungen > Barrierefreiheit | M1 | Must | automated (UI test) |
| NFR-29 | The contrast ratio is at least 4.5:1 for text and 3:1 for focus frames | Nicht-funktionale Anforderungen > Barrierefreiheit | M2 | Must | automated (UI test) |
| NFR-30 | prefers-reduced-motion is respected | Nicht-funktionale Anforderungen > Barrierefreiheit | M2 | Should | manual (visual inspection) |
| NFR-31 | Screen reader support works via AccessKit | Nicht-funktionale Anforderungen > Barrierefreiheit | M2 | Should | manual (operational test) |

### 22. Code quality

| ID | Title | Source | Milestone | Priority | Verification |
|---|---|---|---|---|---|
| NFR-32 | Every public function carries a doc comment in the pattern WHAT, WHY, EXAMPLE; every module begins with a header comment that names its responsibility and its boundary; every constant carries a name that explains its purpose; every public function has at least one unit test | Nicht-funktionale Anforderungen > Codequalität und Einsteigerfreundlichkeit | M0 | Must | automated (CI gate) |
| NFR-33 | No source file exceeds 400 lines (soft) or 550 lines (hard); unsafe_code, unwrap_used and expect_used are forbidden in the workspace; missing_docs is a warning and becomes an error for the public API of the core crates | Nicht-funktionale Anforderungen > Codequalität und Einsteigerfreundlichkeit | M0 | Must | automated (CI gate) |
| NFR-34 | What a beginner cannot follow is rewritten and not left out; deep nesting is avoided with early returns | Nicht-funktionale Anforderungen > Codequalität und Einsteigerfreundlichkeit | M0 | Must | manual (document review) |

### 23. Internationalisation

| ID | Title | Source | Milestone | Priority | Verification |
|---|---|---|---|---|---|
| NFR-35 | All surface texts run through a translation key; no string is hardcoded in the code; German and English are fully covered; date, number and size formats follow the system language | Nicht-funktionale Anforderungen > Internationalisierung | M2 | Must | automated (CI gate) |
| NFR-36 | The code is in English, the documentation is bilingual in German and English, the surface texts are internationalisable with German as default | Nicht-funktionale Anforderungen > Codequalität und Einsteigerfreundlichkeit | M0 | Must | manual (document review) |

### 24. Process and gates

| ID | Title | Source | Milestone | Priority | Verification |
|---|---|---|---|---|---|
| NFR-37 | No gate is weakened to become green: no lint is commented out with an allow attribute, no check is removed from the CI, no test is disabled | Abnahmeregelungen > Die Abnahme-Gates | M0 | Must | manual (document review) |
| NFR-38 | Every evidence is a command output, a test name with a green result or an extract from an acceptance documentation; "it works" is not evidence | Abnahmeregelungen > Belegregeln | M0 | Must | manual (document review) |
| NFR-39 | Tests that could not be executed are reported as NOT RUN with the exact reason | Abnahmeregelungen > Belegregeln | M0 | Must | manual (document review) |
| NFR-40 | A CI check confirms that no crate of the telemetry or analytics categories is referenced, and a test proves that the process start opens no network connection | Produktmaximen > Kostenlos | M0 | Must | automated (CI gate) |
| NFR-41 | The milestone gates are: formatting, static analysis, licence check, vulnerability check, crate boundaries, test run, platform build, performance, security proof and, before M8, the operator reachability | Abnahmeregelungen > Die Abnahme-Gates | M0 | Must | automated (CI gate) |

---

## Change log

| Date | Change |
|---|---|
| 24.10.2026 | Initial catalogue created from `docs/REQUIREMENTS.md` version 1.1. FR-01 to FR-167 and NFR-01 to NFR-41, no ID reused. |
