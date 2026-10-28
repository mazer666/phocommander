//! The default bindings of the contexts.
//!
//! WHAT: the binding of every action of the catalogue in every
//!       context, in the form of the configuration file.
//! WHY: the configuration file is the one that the user edits, and a
//!       file that is missing must not leave the application
//!       without any binding. The code therefore carries the same
//!       bindings as a default, and the file overrides them.
//!       `scripts/generate-keymap-defaults.py` writes this file from
//!       `config/keymap.toml`, and
//!       `scripts/check-keymap-consistency.py` proves that the two
//!       agree.
//!
//! THIS FILE IS GENERATED — do not change it by hand.

use std::collections::BTreeMap;

use crate::context::Context;

/// The default bindings of every context.
///
/// EXAMPLE:
/// ```no_run
/// use phocommander_ui::keymap_view::defaults::default_bindings;
///
/// let bindings = default_bindings();
/// assert!(!bindings.is_empty());
/// ```
pub fn default_bindings() -> BTreeMap<Context, BTreeMap<String, String>> {
    let mut alle: BTreeMap<Context, BTreeMap<String, String>> = BTreeMap::new();

    // keyboard
    {
        let mut tabelle: BTreeMap<String, String> = BTreeMap::new();
        tabelle.insert("panel.rename".to_string(), "F2".to_string());
        tabelle.insert("panel.view".to_string(), "F3".to_string());
        tabelle.insert("panel.edit".to_string(), "F4".to_string());
        tabelle.insert("panel.filter".to_string(), "Ctrl+F".to_string());
        tabelle.insert("panel.copy".to_string(), "F5".to_string());
        tabelle.insert("panel.move".to_string(), "F6".to_string());
        tabelle.insert("panel.mkdir".to_string(), "F7".to_string());
        tabelle.insert("panel.delete".to_string(), "F8".to_string());
        tabelle.insert("app.menu".to_string(), "F9".to_string());
        tabelle.insert("app.quit".to_string(), "F10".to_string());
        tabelle.insert("app.language.de".to_string(), "Ctrl+Shift+D".to_string());
        tabelle.insert("app.language.en".to_string(), "Ctrl+Shift+E".to_string());
        tabelle.insert("app.language.fr".to_string(), "Ctrl+Shift+F".to_string());
        tabelle.insert("panel.switch".to_string(), "Tab".to_string());
        tabelle.insert("panel.reload".to_string(), "Ctrl+R".to_string());
        tabelle.insert("panel.swap".to_string(), "Ctrl+U".to_string());
        tabelle.insert("panel.open_in_other".to_string(), "Ctrl+Enter".to_string());
        tabelle.insert("panel.tab.1".to_string(), "Ctrl+1".to_string());
        tabelle.insert("panel.tab.2".to_string(), "Ctrl+2".to_string());
        tabelle.insert("panel.tab.3".to_string(), "Ctrl+3".to_string());
        tabelle.insert("panel.tab.4".to_string(), "Ctrl+4".to_string());
        tabelle.insert("panel.tab.5".to_string(), "Ctrl+5".to_string());
        tabelle.insert("panel.tab.6".to_string(), "Ctrl+6".to_string());
        tabelle.insert("panel.tab.7".to_string(), "Ctrl+7".to_string());
        tabelle.insert("panel.tab.8".to_string(), "Ctrl+8".to_string());
        tabelle.insert("panel.tab.9".to_string(), "Ctrl+9".to_string());
        tabelle.insert("panel.next_tab".to_string(), "Ctrl+Tab".to_string());
        tabelle.insert("panel.prev_tab".to_string(), "Ctrl+Shift+Tab".to_string());
        tabelle.insert("panel.tab.new".to_string(), "Ctrl+T".to_string());
        tabelle.insert("panel.tab.close".to_string(), "Ctrl+W".to_string());
        tabelle.insert(
            "panel.tab.move_to_other".to_string(),
            "Ctrl+Shift+M".to_string(),
        );
        tabelle.insert(
            "panel.layout.toggle".to_string(),
            "Ctrl+Shift+L".to_string(),
        );
        tabelle.insert("panel.tree.toggle".to_string(), "Ctrl+B".to_string());
        tabelle.insert("panel.quick_view.toggle".to_string(), "Ctrl+Q".to_string());
        tabelle.insert("panel.job_list.toggle".to_string(), "Ctrl+J".to_string());
        tabelle.insert("selection.toggle".to_string(), "Ins".to_string());
        tabelle.insert("selection.all".to_string(), "Ctrl+A".to_string());
        tabelle.insert("selection.none".to_string(), "Ctrl+Num-".to_string());
        tabelle.insert("selection.invert".to_string(), "Ctrl+Num*".to_string());
        tabelle.insert("selection.by_pattern".to_string(), "Space".to_string());
        tabelle.insert("panel.drives".to_string(), "Alt+F1".to_string());
        tabelle.insert("panel.parent".to_string(), "Ctrl+PgUp".to_string());
        tabelle.insert("viewer.find_next".to_string(), "F3".to_string());
        tabelle.insert("viewer.find_previous".to_string(), "Shift+F3".to_string());
        tabelle.insert("rename.conflict_next".to_string(), "F3".to_string());
        tabelle.insert(
            "rename.conflict_previous".to_string(),
            "Shift+F3".to_string(),
        );
        alle.insert(Context::Keyboard, tabelle);
    }

    // pointer
    {
        let mut tabelle: BTreeMap<String, String> = BTreeMap::new();
        tabelle.insert("panel.rename".to_string(), "F2".to_string());
        tabelle.insert("panel.view".to_string(), "double-click".to_string());
        tabelle.insert("panel.edit".to_string(), "F4".to_string());
        tabelle.insert("panel.filter".to_string(), "Ctrl+F".to_string());
        tabelle.insert("panel.copy".to_string(), "F5".to_string());
        tabelle.insert("panel.move".to_string(), "F6".to_string());
        tabelle.insert("panel.mkdir".to_string(), "F7".to_string());
        tabelle.insert("panel.delete".to_string(), "F8".to_string());
        tabelle.insert("app.menu".to_string(), "F9".to_string());
        tabelle.insert("app.quit".to_string(), "F10".to_string());
        tabelle.insert("app.language.de".to_string(), "Ctrl+Shift+D".to_string());
        tabelle.insert("app.language.en".to_string(), "Ctrl+Shift+E".to_string());
        tabelle.insert("app.language.fr".to_string(), "Ctrl+Shift+F".to_string());
        tabelle.insert("panel.switch".to_string(), "Tab".to_string());
        tabelle.insert("panel.reload".to_string(), "Ctrl+R".to_string());
        tabelle.insert("panel.swap".to_string(), "Ctrl+U".to_string());
        tabelle.insert("panel.open_in_other".to_string(), "Ctrl+Enter".to_string());
        tabelle.insert("panel.tab.1".to_string(), "Ctrl+1".to_string());
        tabelle.insert("panel.tab.2".to_string(), "Ctrl+2".to_string());
        tabelle.insert("panel.tab.3".to_string(), "Ctrl+3".to_string());
        tabelle.insert("panel.tab.4".to_string(), "Ctrl+4".to_string());
        tabelle.insert("panel.tab.5".to_string(), "Ctrl+5".to_string());
        tabelle.insert("panel.tab.6".to_string(), "Ctrl+6".to_string());
        tabelle.insert("panel.tab.7".to_string(), "Ctrl+7".to_string());
        tabelle.insert("panel.tab.8".to_string(), "Ctrl+8".to_string());
        tabelle.insert("panel.tab.9".to_string(), "Ctrl+9".to_string());
        tabelle.insert("panel.next_tab".to_string(), "Ctrl+Tab".to_string());
        tabelle.insert("panel.prev_tab".to_string(), "Ctrl+Shift+Tab".to_string());
        tabelle.insert("panel.tab.new".to_string(), "Ctrl+T".to_string());
        tabelle.insert("panel.tab.close".to_string(), "Ctrl+W".to_string());
        tabelle.insert(
            "panel.tab.move_to_other".to_string(),
            "Ctrl+Shift+M".to_string(),
        );
        tabelle.insert(
            "panel.layout.toggle".to_string(),
            "Ctrl+Shift+L".to_string(),
        );
        tabelle.insert("panel.tree.toggle".to_string(), "Ctrl+B".to_string());
        tabelle.insert("panel.quick_view.toggle".to_string(), "Ctrl+Q".to_string());
        tabelle.insert("panel.job_list.toggle".to_string(), "Ctrl+J".to_string());
        tabelle.insert("selection.toggle".to_string(), "click".to_string());
        tabelle.insert("selection.all".to_string(), "Ctrl+A".to_string());
        tabelle.insert("selection.none".to_string(), "Ctrl+Num-".to_string());
        tabelle.insert("selection.invert".to_string(), "Ctrl+Num*".to_string());
        tabelle.insert("selection.by_pattern".to_string(), "Space".to_string());
        tabelle.insert("panel.drives".to_string(), "Alt+F1".to_string());
        tabelle.insert("panel.parent".to_string(), "Ctrl+PgUp".to_string());
        tabelle.insert("viewer.find_next".to_string(), "F3".to_string());
        tabelle.insert("viewer.find_previous".to_string(), "Shift+F3".to_string());
        tabelle.insert("rename.conflict_next".to_string(), "F3".to_string());
        tabelle.insert(
            "rename.conflict_previous".to_string(),
            "Shift+F3".to_string(),
        );
        alle.insert(Context::Pointer, tabelle);
    }

    // touch
    {
        let mut tabelle: BTreeMap<String, String> = BTreeMap::new();
        tabelle.insert(
            "panel.rename".to_string(),
            "palette:panel.rename".to_string(),
        );
        tabelle.insert("panel.view".to_string(), "tap".to_string());
        tabelle.insert("panel.edit".to_string(), "palette:panel.edit".to_string());
        tabelle.insert(
            "panel.filter".to_string(),
            "palette:panel.filter".to_string(),
        );
        tabelle.insert("panel.copy".to_string(), "palette:panel.copy".to_string());
        tabelle.insert("panel.move".to_string(), "palette:panel.move".to_string());
        tabelle.insert("panel.mkdir".to_string(), "palette:panel.mkdir".to_string());
        tabelle.insert(
            "panel.delete".to_string(),
            "palette:panel.delete".to_string(),
        );
        tabelle.insert("app.menu".to_string(), "palette:app.menu".to_string());
        tabelle.insert("app.quit".to_string(), "palette:app.quit".to_string());
        tabelle.insert(
            "app.language.de".to_string(),
            "palette:app.language.de".to_string(),
        );
        tabelle.insert(
            "app.language.en".to_string(),
            "palette:app.language.en".to_string(),
        );
        tabelle.insert(
            "app.language.fr".to_string(),
            "palette:app.language.fr".to_string(),
        );
        tabelle.insert("panel.switch".to_string(), "swipe-left".to_string());
        tabelle.insert(
            "panel.reload".to_string(),
            "palette:panel.reload".to_string(),
        );
        tabelle.insert("panel.swap".to_string(), "palette:panel.swap".to_string());
        tabelle.insert(
            "panel.open_in_other".to_string(),
            "palette:panel.open_in_other".to_string(),
        );
        tabelle.insert("panel.tab.1".to_string(), "palette:panel.tab.1".to_string());
        tabelle.insert("panel.tab.2".to_string(), "palette:panel.tab.2".to_string());
        tabelle.insert("panel.tab.3".to_string(), "palette:panel.tab.3".to_string());
        tabelle.insert("panel.tab.4".to_string(), "palette:panel.tab.4".to_string());
        tabelle.insert("panel.tab.5".to_string(), "palette:panel.tab.5".to_string());
        tabelle.insert("panel.tab.6".to_string(), "palette:panel.tab.6".to_string());
        tabelle.insert("panel.tab.7".to_string(), "palette:panel.tab.7".to_string());
        tabelle.insert("panel.tab.8".to_string(), "palette:panel.tab.8".to_string());
        tabelle.insert("panel.tab.9".to_string(), "palette:panel.tab.9".to_string());
        tabelle.insert(
            "panel.next_tab".to_string(),
            "palette:panel.next_tab".to_string(),
        );
        tabelle.insert(
            "panel.prev_tab".to_string(),
            "palette:panel.prev_tab".to_string(),
        );
        tabelle.insert(
            "panel.tab.new".to_string(),
            "palette:panel.tab.new".to_string(),
        );
        tabelle.insert(
            "panel.tab.close".to_string(),
            "palette:panel.tab.close".to_string(),
        );
        tabelle.insert(
            "panel.tab.move_to_other".to_string(),
            "palette:panel.tab.move_to_other".to_string(),
        );
        tabelle.insert(
            "panel.layout.toggle".to_string(),
            "palette:panel.layout.toggle".to_string(),
        );
        tabelle.insert(
            "panel.tree.toggle".to_string(),
            "palette:panel.tree.toggle".to_string(),
        );
        tabelle.insert(
            "panel.quick_view.toggle".to_string(),
            "palette:panel.quick_view.toggle".to_string(),
        );
        tabelle.insert(
            "panel.job_list.toggle".to_string(),
            "palette:panel.job_list.toggle".to_string(),
        );
        tabelle.insert("selection.toggle".to_string(), "long-press".to_string());
        tabelle.insert(
            "selection.all".to_string(),
            "palette:selection.all".to_string(),
        );
        tabelle.insert(
            "selection.none".to_string(),
            "palette:selection.none".to_string(),
        );
        tabelle.insert(
            "selection.invert".to_string(),
            "palette:selection.invert".to_string(),
        );
        tabelle.insert(
            "selection.by_pattern".to_string(),
            "palette:selection.by_pattern".to_string(),
        );
        tabelle.insert(
            "panel.drives".to_string(),
            "palette:panel.drives".to_string(),
        );
        tabelle.insert(
            "panel.parent".to_string(),
            "palette:panel.parent".to_string(),
        );
        tabelle.insert(
            "viewer.find_next".to_string(),
            "palette:viewer.find_next".to_string(),
        );
        tabelle.insert(
            "viewer.find_previous".to_string(),
            "palette:viewer.find_previous".to_string(),
        );
        tabelle.insert(
            "rename.conflict_next".to_string(),
            "palette:rename.conflict_next".to_string(),
        );
        tabelle.insert(
            "rename.conflict_previous".to_string(),
            "palette:rename.conflict_previous".to_string(),
        );
        alle.insert(Context::Touch, tabelle);
    }

    // dpad
    {
        let mut tabelle: BTreeMap<String, String> = BTreeMap::new();
        tabelle.insert(
            "panel.rename".to_string(),
            "palette:file.rename".to_string(),
        );
        tabelle.insert("panel.view".to_string(), "palette:file.view".to_string());
        tabelle.insert("panel.edit".to_string(), "palette:file.edit".to_string());
        tabelle.insert(
            "panel.filter".to_string(),
            "palette:panel.filter".to_string(),
        );
        tabelle.insert("panel.copy".to_string(), "palette:file.copy".to_string());
        tabelle.insert("panel.move".to_string(), "palette:file.move".to_string());
        tabelle.insert("panel.mkdir".to_string(), "palette:file.mkdir".to_string());
        tabelle.insert(
            "panel.delete".to_string(),
            "palette:file.delete".to_string(),
        );
        tabelle.insert("app.menu".to_string(), "palette:app.menu".to_string());
        tabelle.insert("app.quit".to_string(), "palette:app.quit".to_string());
        tabelle.insert(
            "app.language.de".to_string(),
            "palette:app.language.de".to_string(),
        );
        tabelle.insert(
            "app.language.en".to_string(),
            "palette:app.language.en".to_string(),
        );
        tabelle.insert(
            "app.language.fr".to_string(),
            "palette:app.language.fr".to_string(),
        );
        tabelle.insert(
            "panel.switch".to_string(),
            "palette:panel.switch".to_string(),
        );
        tabelle.insert(
            "panel.reload".to_string(),
            "palette:panel.reload".to_string(),
        );
        tabelle.insert("panel.swap".to_string(), "palette:panel.swap".to_string());
        tabelle.insert(
            "panel.open_in_other".to_string(),
            "palette:panel.open_in_other".to_string(),
        );
        tabelle.insert("panel.tab.1".to_string(), "palette:panel.tab.1".to_string());
        tabelle.insert("panel.tab.2".to_string(), "palette:panel.tab.2".to_string());
        tabelle.insert("panel.tab.3".to_string(), "palette:panel.tab.3".to_string());
        tabelle.insert("panel.tab.4".to_string(), "palette:panel.tab.4".to_string());
        tabelle.insert("panel.tab.5".to_string(), "palette:panel.tab.5".to_string());
        tabelle.insert("panel.tab.6".to_string(), "palette:panel.tab.6".to_string());
        tabelle.insert("panel.tab.7".to_string(), "palette:panel.tab.7".to_string());
        tabelle.insert("panel.tab.8".to_string(), "palette:panel.tab.8".to_string());
        tabelle.insert("panel.tab.9".to_string(), "palette:panel.tab.9".to_string());
        tabelle.insert(
            "panel.next_tab".to_string(),
            "palette:panel.next_tab".to_string(),
        );
        tabelle.insert(
            "panel.prev_tab".to_string(),
            "palette:panel.prev_tab".to_string(),
        );
        tabelle.insert(
            "panel.tab.new".to_string(),
            "palette:panel.tab.new".to_string(),
        );
        tabelle.insert(
            "panel.tab.close".to_string(),
            "palette:panel.tab.close".to_string(),
        );
        tabelle.insert(
            "panel.tab.move_to_other".to_string(),
            "palette:panel.tab.move_to_other".to_string(),
        );
        tabelle.insert(
            "panel.layout.toggle".to_string(),
            "palette:panel.layout.toggle".to_string(),
        );
        tabelle.insert(
            "panel.tree.toggle".to_string(),
            "palette:panel.tree.toggle".to_string(),
        );
        tabelle.insert(
            "panel.quick_view.toggle".to_string(),
            "palette:panel.quick_view.toggle".to_string(),
        );
        tabelle.insert(
            "panel.job_list.toggle".to_string(),
            "palette:panel.job_list.toggle".to_string(),
        );
        tabelle.insert(
            "selection.toggle".to_string(),
            "palette:selection.toggle".to_string(),
        );
        tabelle.insert(
            "selection.all".to_string(),
            "palette:selection.all".to_string(),
        );
        tabelle.insert(
            "selection.none".to_string(),
            "palette:selection.none".to_string(),
        );
        tabelle.insert(
            "selection.invert".to_string(),
            "palette:selection.invert".to_string(),
        );
        tabelle.insert(
            "selection.by_pattern".to_string(),
            "palette:selection.by_pattern".to_string(),
        );
        tabelle.insert(
            "panel.drives".to_string(),
            "palette:panel.drives".to_string(),
        );
        tabelle.insert(
            "panel.parent".to_string(),
            "palette:panel.parent".to_string(),
        );
        tabelle.insert(
            "viewer.find_next".to_string(),
            "palette:viewer.find_next".to_string(),
        );
        tabelle.insert(
            "viewer.find_previous".to_string(),
            "palette:viewer.find_previous".to_string(),
        );
        tabelle.insert(
            "rename.conflict_next".to_string(),
            "palette:rename.conflict_next".to_string(),
        );
        tabelle.insert(
            "rename.conflict_previous".to_string(),
            "palette:rename.conflict_previous".to_string(),
        );
        tabelle.insert("palette.open".to_string(), "Menu".to_string());
        alle.insert(Context::Dpad, tabelle);
    }

    // controller
    {
        let mut tabelle: BTreeMap<String, String> = BTreeMap::new();
        tabelle.insert(
            "panel.rename".to_string(),
            "palette:file.rename".to_string(),
        );
        tabelle.insert("panel.view".to_string(), "palette:file.view".to_string());
        tabelle.insert("panel.edit".to_string(), "palette:file.edit".to_string());
        tabelle.insert(
            "panel.filter".to_string(),
            "palette:panel.filter".to_string(),
        );
        tabelle.insert("panel.copy".to_string(), "palette:file.copy".to_string());
        tabelle.insert("panel.move".to_string(), "palette:file.move".to_string());
        tabelle.insert("panel.mkdir".to_string(), "palette:file.mkdir".to_string());
        tabelle.insert(
            "panel.delete".to_string(),
            "palette:file.delete".to_string(),
        );
        tabelle.insert("app.menu".to_string(), "palette:app.menu".to_string());
        tabelle.insert("app.quit".to_string(), "palette:app.quit".to_string());
        tabelle.insert(
            "app.language.de".to_string(),
            "palette:app.language.de".to_string(),
        );
        tabelle.insert(
            "app.language.en".to_string(),
            "palette:app.language.en".to_string(),
        );
        tabelle.insert(
            "app.language.fr".to_string(),
            "palette:app.language.fr".to_string(),
        );
        tabelle.insert(
            "panel.switch".to_string(),
            "palette:panel.switch".to_string(),
        );
        tabelle.insert(
            "panel.reload".to_string(),
            "palette:panel.reload".to_string(),
        );
        tabelle.insert("panel.swap".to_string(), "palette:panel.swap".to_string());
        tabelle.insert(
            "panel.open_in_other".to_string(),
            "palette:panel.open_in_other".to_string(),
        );
        tabelle.insert("panel.tab.1".to_string(), "palette:panel.tab.1".to_string());
        tabelle.insert("panel.tab.2".to_string(), "palette:panel.tab.2".to_string());
        tabelle.insert("panel.tab.3".to_string(), "palette:panel.tab.3".to_string());
        tabelle.insert("panel.tab.4".to_string(), "palette:panel.tab.4".to_string());
        tabelle.insert("panel.tab.5".to_string(), "palette:panel.tab.5".to_string());
        tabelle.insert("panel.tab.6".to_string(), "palette:panel.tab.6".to_string());
        tabelle.insert("panel.tab.7".to_string(), "palette:panel.tab.7".to_string());
        tabelle.insert("panel.tab.8".to_string(), "palette:panel.tab.8".to_string());
        tabelle.insert("panel.tab.9".to_string(), "palette:panel.tab.9".to_string());
        tabelle.insert(
            "panel.next_tab".to_string(),
            "palette:panel.next_tab".to_string(),
        );
        tabelle.insert(
            "panel.prev_tab".to_string(),
            "palette:panel.prev_tab".to_string(),
        );
        tabelle.insert(
            "panel.tab.new".to_string(),
            "palette:panel.tab.new".to_string(),
        );
        tabelle.insert(
            "panel.tab.close".to_string(),
            "palette:panel.tab.close".to_string(),
        );
        tabelle.insert(
            "panel.tab.move_to_other".to_string(),
            "palette:panel.tab.move_to_other".to_string(),
        );
        tabelle.insert(
            "panel.layout.toggle".to_string(),
            "palette:panel.layout.toggle".to_string(),
        );
        tabelle.insert(
            "panel.tree.toggle".to_string(),
            "palette:panel.tree.toggle".to_string(),
        );
        tabelle.insert(
            "panel.quick_view.toggle".to_string(),
            "palette:panel.quick_view.toggle".to_string(),
        );
        tabelle.insert(
            "panel.job_list.toggle".to_string(),
            "palette:panel.job_list.toggle".to_string(),
        );
        tabelle.insert("selection.toggle".to_string(), "Grip".to_string());
        tabelle.insert(
            "selection.all".to_string(),
            "palette:selection.all".to_string(),
        );
        tabelle.insert(
            "selection.none".to_string(),
            "palette:selection.none".to_string(),
        );
        tabelle.insert(
            "selection.invert".to_string(),
            "palette:selection.invert".to_string(),
        );
        tabelle.insert(
            "selection.by_pattern".to_string(),
            "palette:selection.by_pattern".to_string(),
        );
        tabelle.insert(
            "panel.drives".to_string(),
            "palette:panel.drives".to_string(),
        );
        tabelle.insert(
            "panel.parent".to_string(),
            "palette:panel.parent".to_string(),
        );
        tabelle.insert(
            "viewer.find_next".to_string(),
            "palette:viewer.find_next".to_string(),
        );
        tabelle.insert(
            "viewer.find_previous".to_string(),
            "palette:viewer.find_previous".to_string(),
        );
        tabelle.insert(
            "rename.conflict_next".to_string(),
            "palette:rename.conflict_next".to_string(),
        );
        tabelle.insert(
            "rename.conflict_previous".to_string(),
            "palette:rename.conflict_previous".to_string(),
        );
        tabelle.insert("palette.open".to_string(), "Menu".to_string());
        alle.insert(Context::Controller, tabelle);
    }
    alle
}
