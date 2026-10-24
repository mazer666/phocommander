# Getting started with Phocommander

> English | [Deutsch](getting-started.de.md)

Phocommander is a two-pane file manager with a built-in viewer. This page shows you how
to start the program, how to move around in it, which ten keys you need first and how
to copy a file.

**Availability:** Keyboard operation and the two panes are available from milestone
**M1**, copying as well. The menu bar and the tabs arrive with **M2**. The complete key
catalogue is in [`docs/KEYMAP.md`](../KEYMAP.md), the binding specification in
[`docs/REQUIREMENTS.md`](../REQUIREMENTS.md).

---

## 1. Starting the program

Phocommander is a Rust program without an installation wizard. After the start you
see the work surface directly — no landing page, no sign-in, no account.

| Platform | How to start |
|---|---|
| macOS | `cargo run -p phocommander-desktop` from the repository, or the finished `.app` from a release |
| Windows, Linux, BSD | `cargo run -p phocommander-desktop` from the repository, or the package of your distribution |
| Android, iPadOS | Not available yet — planned for M4 (Android) and M5 (iPadOS) |

> **State in M0:** the program starts, names the host platform, the primary modifier
> key, the configuration directory and the configuration schema version, and exits
> again. It does not open a window yet — an empty surface would be worse than a
> program that says what it is. The work surface arrives with M1.

On the first start, one pane opens with your home directory and the other with the same
directory. Both panes are equal: which one the active action applies to is decided by
the focus.

> **Two panes, one source, one target.** The left side is the source and the right
> side the target — but the panes are not fixed in that role. Both can be source or
> target. The hand-over to the other pane is always an explicit act (`Ctrl+Enter`), so
> that no file ends up in the wrong directory by accident.

---

## 2. Navigating

| Key | Effect |
|---|---|
| `↑` `↓` | One line up or down |
| `PgUp` `PgDn` | One page further |
| `Home` `End` | Start or end of the list |
| `Ctrl+PgUp` | One level up |
| `Tab` | Switch the active pane |
| `Ctrl+Enter` | Open the selection in the other pane |
| `Ctrl+R` | Re-read the directory |
| `Ctrl+U` | Swap the two panes |

In the directory tree of a pane you move with the arrow keys. The view mode is kept
when you leave a directory — the same directory type is always displayed the same way.

**The input line** sits along the bottom edge. You type a path there and press Enter to
jump straight there — faster than clicking through the hierarchy. A command-line
history is reachable with `Ctrl+Down`.

> The directory tree, the input line and the filter field are part of **M2**. Before
> that, navigate with the keys in the table above.

---

## 3. Your first ten keys

| Key | Effect |
|---|---|
| `F2` | Rename — with several files marked, the multiple-rename dialog opens |
| `F3` | Open the viewer |
| `F4` | Editor — planned for M9, the binding is reserved |
| `F5` | Copy into the other pane |
| `F6` | Move into the other pane |
| `F7` | Create a directory |
| `F8` | Delete |
| `F9` | Menu |
| `F10` | Quit |
| `Tab` | Switch the active pane |
| `Ctrl+Enter` | Open the selection in the other pane |

The complete list with context and platform variant is in
[`docs/KEYMAP.md`](../KEYMAP.md).

---

## 4. Copying a file

This is the first operation worth practising — it shows the two-pane principle within
a minute.

1. **Choose the source pane.** Press `Tab` until the pane holding the file you want to
   copy is the active one. The active pane has the visible focus frame.
2. **Mark the file.** Move the cursor with the arrow keys onto the file and press
   `Ins`. The file is marked.
3. **Choose the target.** Press `Ctrl+Enter` — the file is opened in the other pane.
   The target pane is now the directory you copy into. You can also navigate in the
   target pane without opening the file.
4. **Copy.** Press `F5`.

The operation runs as a **job** in the background: the interface stays usable, the job
shows progress and a rate, and it can be cancelled.

### What happens on a conflict

If a file with the same name already exists in the target, the conflict policy decides
— **overwriting is never the default**:

| Policy | Effect |
|---|---|
| `ask` (default) | You are asked |
| `never` | The existing file stays, the job reports the conflict |
| `always` | The existing file is overwritten |
| `keep_both` | Both are kept, the new file is renamed |

The policy lives in the configuration file under `[panels] overwrite_policy`.

---

## 5. Marking several files

| Key | Effect |
|---|---|
| `Ins` | Toggle the marking of the entry under the cursor |
| `Ctrl+A` | Mark all entries |
| `Ctrl+Num-` | Clear the marking |
| `Ctrl+Num*` | Invert the marking |
| `Space` | Mark by pattern — type the pattern afterwards |

`F5`, `F6` and `F8` apply to the marking. If nothing is marked, the action applies to
the entry under the cursor.

---

## 6. Viewing a file

Press `F3`. The viewer opens in an additional area without starting an external
program. In the first tier it shows:

- text and source code (with character set detection, UTF-8 and UTF-16),
- a hex dump (offset, hex, ASCII),
- images (PNG, JPEG, GIF, WebP, BMP, TIFF),
- the contents of an archive,
- metadata (size, type, timestamp, checksum).

PDF and syntax highlighting follow with **M2**, audio and video with **M9**. The
viewer is **read-only** — it does not modify a file.

In the viewer you search forward with `F3` and backward with `Shift+F3`.

---

## 7. Where your settings live

Phocommander writes a single configuration file in TOML format to the location your
operating system prescribes:

| Platform | Location |
|---|---|
| macOS | `~/Library/Application Support/Phocommander/` |
| Windows | `%APPDATA%\Phocommander\` |
| Linux, BSD | `$XDG_CONFIG_HOME/phocommander/` (default: `~/.config/phocommander/`) |
| Android, iPadOS | The sandbox of the app |

It holds the settings, the key bindings, the favourites and — if you want that — the
recently opened directories. More about that in [`PRIVACY.md`](../../PRIVACY.md).

---

## 8. If something does not work

| Symptom | Cause and remedy |
|---|---|
| A key does nothing | The binding was changed. Check the `[keymap]` section of the configuration file and the table in [`docs/KEYMAP.md`](../KEYMAP.md). |
| `Ctrl` does nothing on macOS | On macOS and iPadOS, `Cmd` is the primary modifier. `Ctrl+Enter` is `Cmd+Enter` there. |
| A file ended up in the wrong directory | `Ctrl+U` swaps the panes, `Ctrl+Enter` opens in the other pane. Check which pane is active before you copy. |
| The trash accepts nothing | A mounted drive or a network share has no trash. Choose a different method when deleting. |
| The viewer shows a binary file as text | `F3` does not recognise the content. The hex dump always shows the raw bytes. |

Further reading: the [glossary](../glossary.md) and the
[specification](../REQUIREMENTS.md).

---

## Read next

- [Multiple rename](multi-rename.en.md) — series, placeholders, conflict jump
- [Key reference](../KEYMAP.md) — every action, every context
- [Glossary](../glossary.md) — the terms that carry a specific meaning in this project
