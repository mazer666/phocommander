# Multiple rename

> English | [Deutsch](multi-rename.de.md)

The multiple-rename dialog is the reason to use Phocommander at all: you mark twenty
files, describe the rule once, and the dialog renames all twenty — with a preview,
with conflict detection and without silent surprises.

**Availability:** The dialog arrives with **M1**, together with the named templates.
The scope is described in [`docs/REQUIREMENTS.md`](../REQUIREMENTS.md), the key
bindings in [`docs/KEYMAP.md`](../KEYMAP.md).

---

## 1. Opening the dialog

1. Mark the files you want to rename — with `Ins`, `Ctrl+A`, or by pattern with the
   space bar.
2. Press `F2`.

With a single marked file you get the simple rename field. With several files the
multiple-rename dialog opens.

---

## 2. How the dialog is laid out

| Area | Content |
|---|---|
| **Left** | The list of selected files |
| **Right** | The table of *original name* and *new name* — with a column of its own for the recognised fields |
| **Bottom** | The rule definition: placeholders, replacement, counter |

The column with the recognised fields is the actual purpose of the dialog: before you
click, you see whether the detection worked. With a series whose file names are
inconsistent, that is exactly the point at which you would otherwise rename twenty
files wrongly.

---

## 3. How a rule is evaluated

The three parts of the rule are evaluated **in this order**:

### 3.1 Placeholders

Placeholders are building blocks that are replaced by a value when the rename runs.

| Placeholder | Replaced by |
|---|---|
| `[name]` | Base name of the file without the extension |
| `[ext]` | File extension |
| `[parent]` | Name of the parent directory |
| `[n]` | Running counter |
| `[date]` | Date |
| `[time]` | Time |
| `[size]` | File size |
| `[hash]` | Short checksum |
| `[rand]` | Random value |
| `[case]` / `[lower]` / `[upper]` / `[title]` | The text in upper case, lower case, capitalised or title case |
| `[series]` | Name of the series |
| `[season]` | Season of the series |
| `[episode]` | Number of the episode |
| `[episode_title]` | Title of the episode |

The width of a number comes from the **counter** and from the fields of the series,
not from the placeholder. A counter with three digits writes `[n]` as `001`; a season
with two digits writes `[season]` as `02`. There is no format such as `{n:03}` — a
name that carries one is not a placeholder and stays as it is.

### 3.2 Replace

Search text and replacement text, optionally switchable to **regular expressions**.
Without regex the text is matched literally.

### 3.3 Counter

Start value, step, digit count with leading zeros, and the order in which the files
receive the counter — for example in the order of the selection, by file name or by
date.

---

## 4. Preview and conflicts

The **live preview** shows the new name of every marked file while you edit the rule.

If the dialog computes a name that already exists in the target directory, the entry
is **colour-marked** and added to a separate **conflict-jump list**:

| Key | Effect |
|---|---|
| `F3` | Jump to the next conflict |
| `Shift+F3` | Jump to the previous conflict |

That way you see every conflict in one pass without leaving the list.

---

## 5. Templates

Rules can be saved as **named templates** and applied again. Every template is a file in the
configuration directory and can be adjusted by you.

The named templates carry **curly braces**, the dialog carries **square brackets**. The two
dialects are separate on purpose: the file of the templates is read by you, and its braces make
the placeholders stand out from the brackets that a file name may carry. The conversion happens
when a template is stored and when it is applied to the dialog.

| Template | Target format |
|---|---|
| **Series episode** | `{series} - S{season}E{episode} - {episode_title}.{ext}` — the season with two digits and the episode with two digits |
| **Camera** | `{n}_{series}_{title}.{ext}` — the counter is written with three digits |
| **Podcast** | `{date}_{title}.{ext}` — the date as `YYYY-MM-DD` |
| **Invoice** | `{parent}_{n}_{name}.{ext}` — the parent directory, the counter and the name of the file |
| **Music** | `{parent}_{n}_{name}.{ext}` — the parent directory as the album, the counter as the track number |

The templates live in the `templates/` subdirectory of the configuration directory. You
create your own template by saving a rule in the dialog and giving it a name.

---

## 6. Series episodes — the pattern in detail

The most common case: naming episodes of a series after the pattern
`Series name - S01E01 - Episode title.Format`.

### The example

You have the folder `The Expanse` with the files

```
Duldung.avi
Leviathan.avi
Homegrown.avi
```

The dialog finds no `SxxEyy` pattern in `Duldung.avi`. It therefore asks for the
season and the episode number and **remembers what you entered** — the next file is
pre-filled with the same values. So you enter season `1` and episode `1` once,
confirm the pre-fill for `Leviathan.avi`, and raise the episode to `2`.

The **series name** comes from the parent directory: the folder is called
`The Expanse`, so the series name is `The Expanse`. That is the default — but it is
always shown and can be overridden, because for series filed in collection folders it
is otherwise wrong.

The result:

```
Duldung.avi     →  The Expanse - S01E01 - Duldung.avi
Leviathan.avi   →  The Expanse - S01E02 - Leviathan.avi
Homegrown.avi   →  The Expanse - S01E03 - Homegrown.avi
```

### Two digits versus three

`S01E01` and not `S1E1`: season and episode number are zero-padded to two digits. If
the selection contains an episode with a three-digit number, the dialog raises the
width automatically to three — otherwise the files of one run would not be named
consistently:

```
Season 1, episode 100  →  The Expanse - S01E100 - … .avi
```

If the season number is three digits as well, it follows the same rule.

### Files that are already named

If a file name already contains an `SxxEyy` pattern — say `The.Expanse.S01E03.avi` —
the dialog takes over season and episode as the pre-fill and offers the rest of the
name as the episode title. That saves typing for episodes that are already filed in
order.

### Characters that are not allowed

The episode title is taken over **unchanged**. A colon, a slash or a question mark are
not allowed on Windows and fail there.

The dialog points this out **before** the rename runs and suggests a replacement:

```
The Expanse - S01E01 - Chapter 1: The Big One.avi
                              ^  colon — not allowed on Windows
suggestion:  The Expanse - S01E01 - Chapter 1 - The Big One.avi
```

Nothing is changed silently — you decide.

---

## 7. When something is not right

| Symptom | Cause and remedy |
|---|---|
| The episode number does not increase | The rule has no counter. Add a start value and a step under "Counter". |
| The series name is the folder name, not the series | The series name comes from the parent directory by default. It can be overridden in the dialog. |
| The preview stays empty | The rule is missing the placeholder that produces the name — usually `{name}` or `{title}`. |
| A name is shown in red | The name already exists in the target directory. `F3` jumps to the next conflict, `Shift+F3` to the previous one. |
| The detection does not fire | The pattern `SxxEyy` is matched case-insensitively. A name like `Season 1, Episode 2` does not contain the pattern and is asked for. |

---

## Read next

- [Getting started](getting-started.en.md) — start, navigate, copy a file
- [Key reference](../KEYMAP.md) — every action, every context
- [Glossary](../glossary.md) — placeholder, conflict jump, virtual directory
