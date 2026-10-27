# Keymap — the complete binding reference

Phocommander is operated in five input contexts. **The binding is to actions, not to
devices**: every operable function has a stable action ID, the input layer translates
a key press, a click, a swipe or a controller trigger into that ID, and the interface
never asks where the input came from.

This document is the reference for that catalogue:

- The catalogue itself lives in [`config/keymap.toml`](../config/keymap.toml) — one
  file, one section per context, one line per action.
- The defaults for the central settings live in
  [`config/defaults.toml`](../config/defaults.toml); its `[keymap]` section overrides
  single assignments for one installation.
- The binding is described in the specification, sections **6.1 The action model** and
  **6.3 The action palette**, in [`docs/REQUIREMENTS.md`](REQUIREMENTS.md). This page follows
  the catalogue of the specification and adds what the specification states in prose.

## The rules behind the table

**Function keys.** `F2` to `F10` carry the main operations — the Norton Commander
tradition that has been the de facto standard since the 1980s. `F3` means "view" in
the panel *and* "search forward" in the viewer *and* "jump to the next conflict" in
the multiple-rename dialog. The key is the same; the action depends on the surface
that has the focus.

**The primary modifier.** `Ctrl` in this document means `Cmd` on macOS and iPadOS.
On Windows, Linux and BSD it is `Ctrl`. The *actions* never differ between
platforms — only the modifier key does.

**Rebinding.** Every binding is configurable. The default catalogue is a template; the
active file lives in the platform's configuration directory and overrides single
assignments. See [Changing the binding](#changing-the-binding).

**Devices without function keys.** A TV remote control and a gamepad have neither
`F` keys nor modifier keys. There, the **action palette** takes over: a searchable
list of every action available in the current context. See
[The contexts `pointer`, `touch`, `dpad` and `controller`](#the-contexts-pointer-touch-dpad-and-controller).

---

## The bindings

Milestones in the last column follow the milestone table in
[`docs/REQUIREMENTS.md`](REQUIREMENTS.md). Where the specification names a binding
but does not assign it to a milestone, that is written down instead of a guessed
milestone.

| Action | Default | Context | Platform variant | Note |
|---|---|---|---|---|
| Rename — also for multiple files — `panel.rename` | `F2` | `keyboard`, palette | — | With more than one entry selected, the multiple-rename dialog opens instead of the single rename field. M1 |
| Open the viewer `panel.view` | `F3` | `keyboard`, palette | — | The viewer is read-only. M1 |
| Open the editor `panel.edit` | `F4` | `keyboard`, palette | — | **Planned.** The editor is an M9 feature; the binding is reserved so that it does not move later. |
| Filter the entries `panel.filter` | `Ctrl+F` | `keyboard`, palette | `Cmd+F` on macOS/iPadOS | Filters the entries of the active pane while the user types. The field is a line inside the pane and not a dialog, the list follows every key stroke, and the number of the entries that pass the filter is shown. The pattern language: `*` matches any number of characters, `?` exactly one, a leading `~` ignores the case, and a plain text matches anywhere in the name. The specification names the filter in FR-74a to FR-74f and names no key for it; this document and `config/keymap.toml` name `Ctrl+F`. M1 |
| Copy `panel.copy` | `F5` | `keyboard`, palette | — | Copies into the *other* pane. Overwrite is never the default — the conflict policy decides. M1 |
| Move `panel.move` | `F6` | `keyboard`, palette | — | Moves into the other pane. M1 |
| Create directory `panel.mkdir` | `F7` | `keyboard`, palette | — | In the active pane. M1 |
| Delete `panel.delete` | `F8` | `keyboard`, palette | — | The conflict policy applies; the trash is the default path, "secure delete" is an explicit option. M1 |
| Application menu `app.menu` | `F9` | `keyboard`, palette | — | The menu bar follows the platform convention: the system menu bar on macOS, in-window on Windows, Linux and BSD. M2 |
| Quit `app.quit` | `F10` | `keyboard`, palette | — | M0 |
| Switch the active pane `panel.switch` | `Tab` | `keyboard`, palette | — | The two panes alternate. M0 |
| Reload the directory listing `panel.reload` | `Ctrl+R` | `keyboard`, palette | `Cmd+R` on macOS/iPadOS | Re-reads the directory of the active pane. The specification names the binding but assigns it to no milestone. |
| Swap the two panes `panel.swap` | `Ctrl+U` | `keyboard`, palette | `Cmd+U` on macOS/iPadOS | Exchanges source and target. M1 |
| Open the selection in the other pane `panel.open_in_other` | `Ctrl+Enter` | `keyboard`, palette | `Cmd+Enter` on macOS/iPadOS | The explicit hand-over to the other pane — the counterpart of the drag-and-drop variant. M1 |
| Switch to tab 1–9 `panel.tab.1` … `panel.tab.9` | `Ctrl+1` … `Ctrl+9` | `keyboard`, palette | `Cmd+1` … `Cmd+9` on macOS/iPadOS | Tabs arrive with M2. |
| Switch to the next tab `panel.next_tab` | `Ctrl+Tab` | `keyboard`, palette | `Cmd+Tab` on macOS/iPadOS | M2 |
| Switch to the previous tab `panel.prev_tab` | `Ctrl+Shift+Tab` | `keyboard`, palette | `Cmd+Shift+Tab` on macOS/iPadOS | **Not in the specification.** The symmetric completion of `Ctrl+Tab`; `config/keymap.toml` marks it as the first binding to revisit if it proves wrong. M2 with the tabs. |
| Open a new tab `panel.tab.new` | `Ctrl+T` | `keyboard`, palette | `Cmd+T` on macOS/iPadOS | **Not in the specification.** The specification names the function of the tabs in FR-12 and names no key for it. M2 |
| Close the tab that is shown `panel.tab.close` | `Ctrl+W` | `keyboard`, palette | `Cmd+W` on macOS/iPadOS | **Not in the specification.** The last tab of a panel is never closed — a panel without a tab would have nothing to show. M2 |
| Move the tab into the other panel `panel.tab.move_to_other` | `Ctrl+Shift+M` | `keyboard`, palette | `Cmd+Shift+M` on macOS/iPadOS | **Not in the specification.** The tab keeps its path, its order, its cursor, its marks and its filter when it moves (FR-12, FR-13). M2 |
| Switch the arrangement of the panels `panel.layout.toggle` | `Ctrl+Shift+L` | `keyboard`, palette | `Cmd+Shift+L` on macOS/iPadOS | **Not in the specification.** The panels stand next to each other or below each other, and the state of the panels does not change with the arrangement (FR-11). M2 |
| Show or hide the tree of the directories `panel.tree.toggle` | `Ctrl+B` | `keyboard`, palette | `Cmd+B` on macOS/iPadOS | **Not in the specification.** The tree belongs to one panel. It shows the directories the user has opened and the directories below the current path — it never reads the tree recursively (FR-14). M2 |
| Show or hide the quick view `panel.quick_view.toggle` | `Ctrl+Q` | `keyboard`, palette | `Cmd+Q` on macOS/iPadOS | **Not in the specification.** The quick view shows the content of the marked file without opening it, and it reads exactly one file (FR-15). M2 |
| Show or hide the list of the jobs `panel.job_list.toggle` | `Ctrl+J` | `keyboard`, palette | `Cmd+J` on macOS/iPadOS | **Not in the specification.** The list of the jobs shows the queue of the operations (FR-22g). M2 |
| Toggle the selection `selection.toggle` | `Ins` | `keyboard`, palette | — | Toggles the marking of the entry under the cursor. M1 |
| Select all `selection.all` | `Ctrl+A` | `keyboard`, palette | `Cmd+A` on macOS/iPadOS | M1 |
| Clear the selection `selection.none` | `Ctrl+Num-` | `keyboard`, palette | `Cmd+Num-` on macOS/iPadOS | M1 |
| Invert the selection `selection.invert` | `Ctrl+Num*` | `keyboard`, palette | `Cmd+Num*` on macOS/iPadOS | M1 |
| Select by pattern `selection.by_pattern` | `Space` | `keyboard`, palette | — | Opens the pattern input; the selection follows the pattern. M1 |
| Drive list `panel.drives` | `Alt+F1` | `keyboard`, palette | `Alt`/`Option` on all platforms | `Alt+F1` opens the drive list of the active pane, `Alt+F2` the one of the other pane. The specification names the binding but assigns it to no milestone. |
| Parent directory `panel.parent` | `Ctrl+PgUp` | `keyboard`, palette | `Cmd+PgUp` on macOS/iPadOS | Goes up one level in the active pane. The specification names the binding but assigns it to no milestone. |
| Find next in the viewer `viewer.find_next` | `F3` | viewer | — | The same key as "view" — the surface with the focus decides. Comes with the viewer (M1). |
| Find previous in the viewer `viewer.find_previous` | `Shift+F3` | viewer | — | The backward variant of the viewer search. Comes with the viewer (M1). |
| Jump to the next conflict `rename.conflict_next` | `F3` | multiple-rename dialog | — | The conflict list of the rename dialog is walked with `F3` / `Shift+F3`. M1 |
| Jump to the previous conflict `rename.conflict_previous` | `Shift+F3` | multiple-rename dialog | — | The backward variant of the conflict jump. M1 |
| Open the action palette `palette.open` | `Menu` — on remotes without a Menu key: `Play/Pause` | `dpad`, `controller` | — | On the desktop the palette is the "search for an action" dialog. The application must not depend on a menu key — see the specification, section *Fire TV und Google TV*. M8 |

### Why `F3` means three different things

The same physical key carries three actions, and that is deliberate:

- In the **panel**, `F3` opens the viewer.
- In the **viewer**, `F3` searches forward, `Shift+F3` backward.
- In the **multiple-rename dialog**, `F3` jumps to the next name conflict, `Shift+F3`
  to the previous one.

The rule is simple: a key is resolved against the surface that has the focus. That
keeps the muscle memory of the classic two-pane managers intact while the dialogs
reuse the row that is already under the user's hand.

---

## Changing the binding

The catalogue is a TOML file. The shipped template is
[`config/keymap.toml`](../config/keymap.toml); the file you edit is the copy in the
platform's configuration directory:

| Platform | Location |
|---|---|
| macOS | `~/Library/Application Support/Phocommander/keymap.toml` |
| Windows | `%APPDATA%\Phocommander\keymap.toml` |
| Linux, BSD | `$XDG_CONFIG_HOME/phocommander/keymap.toml` (by default `~/.config/phocommander/keymap.toml`) |
| Android, iPadOS | The application sandbox of the app |

### Override a single binding in the central configuration file

The central configuration file (`config/defaults.toml` template, copied to the same
directory) carries a `[keymap]` section that replaces individual assignments:

```toml
# The catalogue itself is `keymap.toml`; this section only replaces individual
# assignments of the platform default for this installation.
# "primary" is Cmd on macOS and iPadOS, Ctrl on Windows, Linux and BSD.
[keymap]
"panel.copy" = "F5"
"panel.move" = "F6"

# A different key for one action:
# "panel.view" = "Ctrl+B"
```

### Change a binding per context

Every action has its own default per context, so that the same program is operated
correctly on a laptop, on a tablet and on a television:

```toml
[input]
# Which operating context is active.
# "auto" detects the input device at run time:
# keyboard or mouse -> keyboard/pointer, touch -> touch,
# remote control -> dpad, controller -> controller.
context = "auto"

[input.keyboard]
"panel.copy" = "F5"
"panel.move" = "F6"

[input.dpad]
# "palette:file.copy" means: the action is reachable through the action palette
# under the entry "file.copy", because the remote control has no F keys.
"panel.copy" = "palette:file.copy"
"palette.open" = "Menu"

[input.controller]
# Trigger = execute the action, grip = toggle the selection.
"panel.copy" = "palette:file.copy"
"selection.toggle" = "Grip"
```

### The notation

| Written as | Means |
|---|---|
| `F5`, `Ins`, `Space`, `Menu` | A physical key. |
| `Ctrl+R`, `Ctrl+Shift+Tab`, `Ctrl+Num-`, `Ctrl+Num*` | A key combination. `Ctrl` is the *primary* modifier: `Cmd` on macOS and iPadOS, `Ctrl` on Windows, Linux and BSD. |
| `Shift+F3` | A key combination with a second modifier. |
| `palette:<group>.<entry>` | The action is reachable through the action palette under that entry. Used in the contexts that have no function keys. |
| `Grip`, `Trigger`, `Thumbstick`, `A`, `B`, `X`, `Y`, `DpadUp`, `DpadDown`, `DpadLeft`, `DpadRight` | A button of a gamepad or of a VR controller. |
| `click`, `double-click`, `right-click`, `drag` | An event of the mouse or the trackpad. The right button opens the menu of the entry and binds therefore no action of the catalogue. |
| `wheel-up`, `wheel-down` | The scroll wheel of the mouse or the trackpad. |
| `tap`, `long-press` | A gesture of a finger. |
| `swipe-left`, `swipe-right`, `swipe-up`, `swipe-down` | A swipe of a finger. |
| `two-finger-up`, `two-finger-down` | A scroll with two fingers. |
| `pinch-in`, `pinch-out` | Two fingers that change the distance between them. |

### Rules for a rebinding

- An **action ID** is never reused for a different meaning, and a new function always
  gets a new ID. The same applies to the binding values.
- A binding that needs a modifier key cannot be reached in the `dpad` context. It
  belongs in the action palette there, or it is not available on a remote control.
- Changing a binding means changing three places in the same commit:
  `config/keymap.toml`, this document, and — if the action itself is new — the
  action catalogue in `crates/phocommander-keymap/`.

---

## The contexts `pointer`, `touch`, `dpad` and `controller`

The specification names five contexts (section 6.2). The keyboard is the one
that carries the complete set of the function keys; the other four reach the
same actions by other means.

| Context | Device | How it reaches the actions |
|---|---|---|
| `keyboard` | Keyboard | The function keys `F2` to `F10` and the key combinations. |
| `pointer` | Mouse, trackpad | The same keys for the actions that a hand does with the keyboard, and the mouse for the actions that a hand does with the mouse: a **click** marks the selection, a **double click** opens the file. |
| `touch` | Finger | The **action palette** takes the place of the function keys, exactly as on the remote control of the television: a **tap** opens the file, a **long press** marks the selection, a **swipe** switches to the second panel. |
| `dpad` | Remote control | The action palette as well, opened from the menu or the play/pause key. |
| `controller` | Gamepad, VR controller | The action palette as well; the focus sits on the ray pointer. |

Every context holds **every** action of the catalogue. The binding differs, the
action does not — that is the promise of FR-101.

## The contexts `dpad` and `controller`

A TV remote control has a D-pad, an OK button, Back, Home and Play/Pause. A gamepad
has a trigger, a grip, a thumbstick and four face buttons. Neither has function
keys, and neither has modifier keys — so the F-key row cannot exist on these
devices.

The **action palette** replaces it:

- It is a searchable list of every action that is available in the current context.
- It opens with the Menu key — or with Play/Pause on remotes that have no Menu key,
  because an application must not depend on a menu key.
- It is operated with the direction keys and OK: the direction keys move the focus,
  OK executes.
- It **filters by context**. In the `dpad` context only actions that can be executed
  without a mouse and without a modifier key appear. An action that needs a held
  Shift key is not reachable there, so it is not shown.
- It is available in every context — on the desktop it is the "search for an
  action" dialog.

The binding in the template makes that explicit:

```toml
[input.dpad]
"panel.copy" = "palette:file.copy"
"palette.open" = "Menu"

[input.controller]
"panel.copy" = "palette:file.copy"
"selection.toggle" = "Grip"
```

The `controller` context has exactly one action outside the palette: the grip
toggles the marking of the entry under the focus, because a multi-select needs a
dedicated control when no modifier key exists.

### Focus and selection are separate states

In the `dpad` and `controller` contexts there is no hover, so focus is a visible,
permanent state rather than a by-product of the mouse position:

- The **focus** is the row the next action applies to.
- The **selection** is the set of marked files an action applies to.
- Navigating moves the focus and leaves the selection alone. Marking changes the
  selection and leaves the focus alone.

The `dpad` context therefore offers an explicit selection mode in which OK toggles
the marking and the direction keys move the focus. The mode is visible and can be
left at any time.

---

## Consistency with the specification

The table above is checked mechanically against the specification and the catalogue:

```bash
python3 scripts/check-keymap-consistency.py
```

The script compares three sources — the keyboard table of the specification, the
`[input.keyboard]` section of `config/keymap.toml` and the table of this document —
and exits with code 1 on a deviation that is not documented in the repository.

The result at the time of writing: **no undocumented deviation**. Two documented
deviations and two notation differences remain:

| # | Finding | Where it is documented |
|---|---|---|
| 1 | `Ctrl+Shift+Tab` for the previous tab exists in the catalogue and in this document, but not in the specification's table — the specification names only the forward direction `Ctrl+Tab`. | `config/keymap.toml` marks it as "the symmetric completion and the first point to adjust if it turns out to be wrong"; this document repeats the note. |
| 2 | `Alt+F2` opens the drive list of the *other* pane. The catalogue binds the action `panel.drives` to `Alt+F1` and describes `Alt+F2` in the comment; the specification names both keys for one action. | `config/keymap.toml`, the comment above `panel.drives`. |
| 3 | The viewer search (`F3` / `Shift+F3`) and the conflict jump (`F3` / `Shift+F3`) are rows of this table, but rows of the *prose* in the specification, not of its table. | The paragraph below the keyboard table in the specification. |
| 4 | The specification writes the space bar as `Leertaste`, the catalogue and this document as `Space`. | The notation table in this document. |

Everything else — the F-key row, the modifier combinations, the selection keys, the
drive list and the parent directory — matches the specification and the catalogue
one to one.
