# Acceptance of the milestone M4 — the seam of the input

**State:** 27.10.2026 · **Examiner:** Lead instance · **Result:** **in work** — the input seam is
accepted, the layout of the finger and the Android build are open

---

## 1. What this milestone covers

M4 makes the application **operable without a keyboard**. The specification names five contexts in
FR-101 — the keyboard, the pointer, the finger, the remote control and the gamepad — and it
requires that all five reach the **same actions**. The milestone builds the seam at which these
five inputs meet:

| Part | Content | State |
|---|---|---|
| The events | `PointerEvent`, `Gesture`, `Direction`, `PinchDirection`, `ControllerButton` | **done** |
| The bindings | `Binding::Pointer`, `Binding::Gesture`, `Binding::Controller` and their notation | **done** |
| The recognition | The translator out of the events of the interface framework: click, double click, right click, drag, scroll, tap, long press, swipe, pinch | **done** |
| The table | Five contexts in `config/keymap.toml` and as the defaults in the code | **done** |
| The interface of the finger | Stacked panels, rows of 44 points, sheets instead of dialogs, no hover | **open** |
| The Android build | Project structure, `cdylib`, SAF, NDK | **open** — the sandbox has no NDK |

## 2. The measured state

| What | How it was measured | Result |
|---|---|---|
| The tests of the workspace | `cargo test --workspace --locked` | **1710 green, 0 red, 0 skipped** |
| The formatting | `cargo fmt --all -- --check` | **without a deviation** |
| The warnings of the linter | `cargo clippy --workspace --all-targets --locked -- -D warnings` | **without a warning** |
| The consistency gates | `scripts/check-*.py`, `scripts/check-*.sh` | **15, all PASS** |
| The gates prove themselves | `scripts/check-gates-negative.sh` | **39 gates in 42 runs**, each turns red and green again |

The number of the tests has risen from 1277 at the acceptance of M3 to **1703**: 426 new tests
for the events, the notation, the recognition of the gestures, the five contexts, the defaults,
the overriding by the file of the user, the two dialects of the placeholders, the layout of the
finger and the change of the language.

## 3. What this milestone found

**The input knew only the keyboard.** `InputEvent` and `Binding` carried only `Key` and `Palette`.
The table of the bindings could therefore name **neither a click nor a tap** — although the
specification demands exactly that in FR-101, and although the interface already knew
`Context::Pointer` and `Context::Touch`. The two contexts were there and carried nothing.

**There were no default bindings in the code at all.** The bindings stood exclusively in
`config/keymap.toml`. But that file is **never written** — it never comes into being on the user's
machine, and `read_keymap()` returned `None`. The interface therefore had **not a single binding**:
no `F5`, no `Tab`, nothing. The gap showed itself while the defaults were written, because the
default of the controller context was `Grip` — and `Grip` is no key. The file had been unreadable
all along, and nobody had noticed, because nobody read it.

**The swipe went out twice.** The gesture went out when the finger passed the distance, and once
more when it lifted. The test of the gesture found it.

**The formatter and the generator contradicted each other.** The generated file of the defaults was
broken up by `cargo fmt`, and the generator compared the bytes. A comparison of the bytes reports a
deviation that is not one. The comparison now reads the **content** — the bindings — and leaves
the spelling alone.

**The check of the bindings knew three contexts only.** It saw `dpad` and `controller` and measured
on them whether every action is bound. The contexts `pointer` and `touch` were unchecked — and
would have stayed empty, without a gate noticing.

**The formatter of `benches/interaction.rs` had never run.** The commit that brought the
benchmarks was made after the last `cargo fmt`. The gate was not green on that commit.

## 4. The decisions that this milestone carries

| Decision | Why |
|---|---|
| **The code carries the defaults, the file overrides them.** | A file that is missing must not leave the application without any binding. `scripts/generate-keymap-defaults.py` produces the defaults out of `config/keymap.toml`, and `scripts/check-keymap-consistency.py` proves that the two agree. |
| **The file of the user overrides only the binding that it names.** | A file with a broken entry does not take the whole set of the bindings away. |
| **The context decides, not the origin of the event.** | The translation produces every signal that the interface sees. The table of the context decides which signal triggers an action. The keyboard triggers no tap, and the finger no key — that follows from the table alone. |
| **The interface recognises the gestures itself.** | The framework of the interface delivers the raw events of a finger, but no gesture. The long press does not arise from an event but from the time the finger rests — the interface therefore asks once per frame. |
| **The buttons of the gamepad are a kind of binding of their own.** | The grip, the trigger and the thumbstick are no keys of a keyboard, and the remote control of the television names exactly these signals. |
| **The two dialects of the placeholders.** | The specification decides in E13: square brackets in the dialog, curly braces in the named templates. The conversion happens at the border of the file of the templates — when a template is stored and when it is applied to the dialog. |

## 5. What is still open

| # | What | Why |
|---|---|---|
| 1 | The layout of the finger | The panels are stacked, the rows are 44 points high, the dialogs come as a sheet from the bottom, and there is no hover. The metric of the row height is set per context already, the arrangement of the controls is not. |
| 2 | The Android build | The sandbox has no Android NDK. The project structure, the `cdylib` and the connection to the Storage Access Framework can be written without an NDK, the build cannot. |
| 3 | The key bindings of the television and of the gamepad in the framework | The events of the controller are named and bound; the framework still has to deliver them. For the remote control of the television the action palette is the intended way, and it exists already. |
| 4 | The trigger of the change of the language | Unchanged and open — the configuration file is read once per second. |
| 5 | The acceptance of M1 and M3 | Unchanged and open — both need real machines. |

## 6. The places of evidence

| Subject | File |
|---|---|
| The events of the pointer and of the finger | `crates/phocommander-ui/src/keymap_view/gesture.rs` |
| The buttons of the gamepad | `crates/phocommander-ui/src/keymap_view/controller.rs` |
| The defaults of the bindings | `crates/phocommander-ui/src/keymap_view/defaults.rs` |
| The recognition of the gestures | `crates/phocommander-ui/src/input/pointer.rs` |
| The processing in the frame | `crates/phocommander-ui/src/app/input.rs` |
| The two dialects of the placeholders | `crates/phocommander-ui/src/rename/dialect.rs` |
| The table of the five contexts | `config/keymap.toml` |
| The production of the defaults | `scripts/generate-keymap-defaults.py` |
| The check of the agreement | `scripts/check-keymap-consistency.py` |
| The description of the bindings | `docs/KEYMAP.md` |
