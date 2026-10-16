//! Window → Workspace: saved panel/bar layouts (Essentials, Essentials Classic, Painting, …),
//! Reset, New Workspace…, Manage Workspaces…. A workspace is a snapshot of `UiState` layout
//! flags; UI brightness and everything else is untouched when switching.

use std::collections::BTreeMap;
use std::sync::atomic::Ordering;

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::state::{Dialog, DockTab, FloatingFlyout, FloatingPanels, UiState};
use crate::theme::Tokens;
use crate::{VectorcraftApp, widgets};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Workspace {
    pub name: String,
    pub control_bar: bool,
    pub rulers: bool,
    pub toolbar: bool,
    pub toolbar_double: bool,
    pub toolbar_advanced: bool,
    pub task_bar: bool,
    pub dock: bool,
    pub dock_tab: DockTab,
    /// The dock's tabbed group is collapsed to icons (built-in workspaces: expanded).
    pub dock_collapsed: bool,
    pub open_panel: Option<String>,
    pub status_bar: bool,
    /// Tool groups torn off the toolbar into floating strips (built-in workspaces: none).
    pub floating_flyouts: Vec<FloatingFlyout>,
    /// Panels dragged out of the dock, where they float (built-in workspaces: none).
    pub floating_panels: Vec<FloatingPanels>,
    /// Custom docking tree, including floating tab groups.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub docking: Option<craft_ui::docking::Layout<String>>,
    #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub docking_hidden: std::collections::BTreeMap<String, craft_ui::docking::Location<String>>,
    /// Panels whose saved return destination is the iconic rail.
    #[serde(default, skip_serializing_if = "std::collections::BTreeSet::is_empty")]
    pub docking_icons: std::collections::BTreeSet<String>,
    /// Where the Tools panel floats (built-in workspaces: docked).
    pub toolbar_pos: Option<[f32; 2]>,
    /// Dock column widths in points, by column key (see [`dock_panel_id`]). Empty (built-in
    /// workspaces, workspaces saved without them): applying keeps the current widths.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub dock_widths: BTreeMap<String, f32>,
}

impl Default for Workspace {
    fn default() -> Self {
        let ui = UiState::default();
        capture(&ui, "Essentials")
    }
}

pub const ESSENTIALS: &str = "Essentials";

/// Built-in workspaces, in Illustrator's menu order (Essentials first).
pub fn builtins() -> Vec<Workspace> {
    let base = Workspace::default();
    let w = |name: &str, f: &dyn Fn(&mut Workspace)| {
        let mut x = Workspace { name: name.into(), ..base.clone() };
        f(&mut x);
        x
    };
    vec![
        w(ESSENTIALS, &|_| {}),
        w("Essentials Classic", &|x| {
            x.control_bar = true;
            x.rulers = true;
            x.toolbar_advanced = true;
            x.task_bar = false;
            x.open_panel = Some("color".into());
        }),
        w("Automation", &|x| {
            x.control_bar = true;
            x.toolbar_advanced = true;
            x.dock_tab = DockTab::Layers;
            x.open_panel = Some("actions".into());
        }),
        w("Layout", &|x| {
            x.control_bar = true;
            x.rulers = true;
            x.toolbar_advanced = true;
            x.dock_tab = DockTab::Layers;
            x.open_panel = Some("align".into());
        }),
        w("Painting", &|x| {
            x.control_bar = true;
            x.toolbar_advanced = true;
            x.toolbar_double = true;
            x.open_panel = Some("brushes".into());
        }),
        w("Printing and Proofing", &|x| {
            x.control_bar = true;
            x.rulers = true;
            x.toolbar_advanced = true;
            x.open_panel = Some("info".into());
        }),
        w("Tracing", &|x| {
            x.control_bar = true;
            x.toolbar_advanced = true;
            x.dock_tab = DockTab::Layers;
            x.task_bar = false;
        }),
        w("Typography", &|x| {
            x.control_bar = true;
            x.rulers = true;
            x.toolbar_advanced = true;
            x.open_panel = Some("character".into());
        }),
        w("Web", &|x| {
            x.control_bar = true;
            x.rulers = true;
            x.toolbar_advanced = true;
            x.open_panel = Some("symbols".into());
        }),
    ]
}

pub fn is_builtin(name: &str) -> bool {
    builtins().iter().any(|w| w.name == name)
}

/// Snapshot the layout flags of `ui` as a workspace called `name`.
pub fn capture(ui: &UiState, name: &str) -> Workspace {
    Workspace {
        name: name.into(),
        control_bar: ui.control_bar,
        rulers: ui.view.rulers,
        toolbar: ui.toolbar,
        toolbar_double: ui.toolbar_double,
        toolbar_advanced: ui.toolbar_advanced,
        task_bar: ui.task_bar,
        dock: ui.dock,
        dock_tab: ui.dock_tab,
        dock_collapsed: ui.dock_collapsed,
        open_panel: ui.open_panel.clone(),
        status_bar: ui.status_bar,
        floating_flyouts: ui.floating_flyouts.clone(),
        floating_panels: ui.floating_panels.clone(),
        docking: ui.docking.clone(),
        docking_hidden: ui.docking_hidden.clone(),
        docking_icons: ui.docking_icons.clone(),
        toolbar_pos: ui.toolbar_pos,
        dock_widths: ui.dock_widths.clone(),
    }
}

/// Apply a workspace's layout flags (brightness, prefs, shortcuts etc. untouched).
pub fn apply(ui: &mut UiState, w: &Workspace) {
    ui.control_bar = w.control_bar;
    ui.view.rulers = w.rulers;
    ui.toolbar = w.toolbar;
    ui.toolbar_double = w.toolbar_double;
    ui.toolbar_advanced = w.toolbar_advanced;
    ui.task_bar = w.task_bar;
    ui.dock = w.dock;
    ui.tab_hid = None;
    ui.dock_tab = w.dock_tab;
    ui.dock_collapsed = w.dock_collapsed;
    ui.open_panel = w.open_panel.clone();
    ui.status_bar = w.status_bar;
    ui.floating_flyouts = w.floating_flyouts.clone();
    ui.floating_panels = w.floating_panels.clone();
    ui.docking_generation = ui.docking_generation.wrapping_add(1);
    ui.docking = w.docking.clone().filter(crate::panel_docking::valid);
    ui.docking_hidden = w.docking_hidden.clone();
    ui.docking_icons =
        w.docking_icons.iter().filter(|id| crate::state::ICON_PANELS.iter().any(|entry| entry.0 == id.as_str())).take(64).cloned().collect();
    ui.toolbar_pos = w.toolbar_pos;
    FloatingPanels::sanitize(&mut ui.floating_panels, &mut ui.toolbar_pos);
    ui.flyout = None;
    ui.dock_widths_pending =
        w.dock_widths.iter().filter(|(_, width)| width.is_finite() && **width > 0.0).take(64).map(|(k, width)| (k.clone(), *width)).collect();
    ui.dock_widths.extend(ui.dock_widths_pending.iter().map(|(k, width)| (k.clone(), *width)));
    ui.workspace = w.name.clone();
}

/// The dock's tabbed panel group (the legacy dock).
pub const DOCK_GROUP: &str = "dock";
/// The shared docking tree's column.
pub const SHARED_DOCK: &str = "shared-panel-dock";
const COLUMN_PREFIX: &str = "dock-column:";

/// The width key of the docked column headed by panel `first`.
pub fn column_key(first: &str) -> String {
    format!("{COLUMN_PREFIX}{first}")
}

/// The panel heading the docked column of width key `key`, if it is a [`column_key`].
pub fn column_first(key: &str) -> Option<&str> {
    key.strip_prefix(COLUMN_PREFIX)
}

/// The egui panel id of a dock width key: [`DOCK_GROUP`], [`SHARED_DOCK`] or a [`column_key`].
pub fn dock_panel_id(key: &str) -> egui::Id {
    match column_first(key) {
        Some(first) => egui::Id::new(("dock-column", first)),
        None => egui::Id::new(key),
    }
}

/// Note a dock column's width as drawn this frame.
pub fn note_dock_width(ui: &mut UiState, key: &str, width: f32) {
    match ui.dock_widths.get_mut(key) {
        Some(w) => *w = width,
        None => {
            ui.dock_widths.insert(key.to_string(), width);
        }
    }
}

/// Give the dock columns the widths an applied workspace saved, once. A resizable egui panel
/// takes its size from its state in egui memory, so the user can drag its edge afterwards.
pub fn restore_dock_widths(ui: &mut UiState, ctx: &egui::Context) {
    if ui.dock_widths_pending.is_empty() {
        return;
    }
    for (key, width) in std::mem::take(&mut ui.dock_widths_pending) {
        if key == SHARED_DOCK && ui.dock_collapsed {
            // Collapsed, the shared dock is icon-wide and overwrites its width: it takes the saved
            // width when it expands.
            ui.dock_widths_pending.insert(key, width);
            continue;
        }
        let id = dock_panel_id(&key);
        ctx.data_mut(|d| {
            let old = d.get_persisted::<egui::containers::panel::PanelState>(id).map_or(egui::Rect::ZERO, |s| s.outer_rect);
            // Only the size counts: the panel lays itself out again from the right edge.
            let outer_rect = egui::Rect::from_min_max(egui::pos2(old.right() - width, old.top()), old.max);
            d.insert_persisted(id, egui::containers::panel::PanelState { outer_rect });
        });
    }
}

/// A workspace by name (user workspaces shadow nothing: names are unique across both lists).
pub fn find(ui: &UiState, name: &str) -> Option<Workspace> {
    ui.custom_workspaces
        .iter()
        .find(|w| w.name.eq_ignore_ascii_case(name))
        .cloned()
        .or_else(|| builtins().into_iter().find(|w| w.name.eq_ignore_ascii_case(name)))
}

/// Every workspace name: built-ins then user workspaces.
pub fn names(ui: &UiState) -> Vec<String> {
    builtins().into_iter().map(|w| w.name).chain(ui.custom_workspaces.iter().map(|w| w.name.clone())).collect()
}

/// Save the current layout as `name` (replacing a user workspace of that name).
pub fn save_as(ui: &mut UiState, name: &str) -> Result<(), String> {
    let name = name.trim();
    if name.is_empty() {
        return Err("workspace name can't be empty".into());
    }
    if is_builtin(name) {
        return Err(format!("`{name}` is a built-in workspace; choose another name"));
    }
    let w = capture(ui, name);
    match ui.custom_workspaces.iter_mut().find(|x| x.name == name) {
        Some(x) => *x = w,
        None => ui.custom_workspaces.push(w),
    }
    ui.workspace = name.into();
    Ok(())
}

pub fn delete(ui: &mut UiState, name: &str) -> Result<(), String> {
    let before = ui.custom_workspaces.len();
    ui.custom_workspaces.retain(|w| w.name != name);
    if ui.custom_workspaces.len() == before {
        return Err(if is_builtin(name) { format!("`{name}` is built in and can't be deleted") } else { format!("no workspace `{name}`") });
    }
    if ui.workspace == name {
        ui.workspace = ESSENTIALS.into();
    }
    Ok(())
}

pub fn rename(ui: &mut UiState, from: &str, to: &str) -> Result<(), String> {
    let to = to.trim();
    if to.is_empty() || is_builtin(to) || ui.custom_workspaces.iter().any(|w| w.name == to) {
        return Err(format!("`{to}` is not an available name"));
    }
    let w = ui.custom_workspaces.iter_mut().find(|w| w.name == from).ok_or_else(|| format!("no user workspace `{from}`"))?;
    w.name = to.into();
    if ui.workspace == from {
        ui.workspace = to.into();
    }
    Ok(())
}

// ---------- menu support (menu_tree has no app access) ----------

crate::shortcut_editor::ui_mirror!(fn names_store() -> Vec<&'static str>);

/// Mirror the user workspace names for the (static) menu tree.
pub fn sync(ui: &UiState) {
    let same = names_store()
        .read()
        .map(|v| v.len() == ui.custom_workspaces.len() && v.iter().zip(&ui.custom_workspaces).all(|(a, b)| *a == b.name))
        .unwrap_or(true);
    if !same {
        if let Ok(mut v) = names_store().write() {
            *v = ui.custom_workspaces.iter().map(|w| crate::shortcut_editor::intern(&w.name)).collect();
        }
        crate::shortcut_editor::GENERATION.fetch_add(1, Ordering::Relaxed);
    }
}

/// Window → Workspace submenu items.
pub fn menu_items() -> Vec<crate::menus::Item> {
    use crate::menus::Item;
    let mut v: Vec<Item> =
        builtins().into_iter().map(|w| Item::Cmd(crate::shortcut_editor::intern(&w.name), "window.workspace", json!({"name": w.name}))).collect();
    let custom = names_store().read().map(|v| v.clone()).unwrap_or_default();
    if !custom.is_empty() {
        v.push(Item::Sep);
        v.extend(custom.into_iter().map(|n| Item::Cmd(n, "window.workspace", json!({"name": n}))));
    }
    v.push(Item::Sep);
    v.push(Item::Cmd("Reset Essentials", "window.workspace.reset", Value::Null));
    v.push(Item::Cmd("New Workspace…", "window.workspace.new", Value::Null));
    v.push(Item::Cmd("Manage Workspaces…", "window.workspace.manage", Value::Null));
    v
}

/// The app-bar workspace switcher's menu.
pub fn popup(app: &mut VectorcraftApp, ui: &mut egui::Ui) {
    ui.set_min_width(210.0);
    ui.set_max_width(240.0);
    let mut clicked: Option<(&'static str, Value)> = None;
    for it in menu_items() {
        match it {
            crate::menus::Item::Sep => {
                ui.separator();
            }
            crate::menus::Item::Cmd(label, id, p) => {
                // As the Window menu draws it: custom workspace names (and Reset <name>) as they are.
                let label = crate::menus::display_label(app, id, label);
                let text = match crate::menus::checked(app, id, &p) {
                    Some(true) => format!("✓  {label}"),
                    Some(false) => format!("     {label}"),
                    None => format!("     {label}"),
                };
                if ui.add(egui::Button::new(text).min_size(egui::vec2(220.0, 0.0))).clicked() {
                    clicked = Some((id, if p.is_null() { json!({}) } else { p }));
                    ui.close();
                }
            }
            _ => {}
        }
    }
    if let Some((id, p)) = clicked {
        crate::menus::invoke(app, id, p);
    }
}

// ---------- commands ----------

pub fn run_command(app: &mut VectorcraftApp, id: &str, p: &Value) -> Option<Result<Value, String>> {
    let s = |k: &str| p.get(k).and_then(Value::as_str).map(str::to_string);
    let r = match id {
        "window.workspace" => match s("name").and_then(|n| find(&app.ui, &n)) {
            Some(w) => {
                apply(&mut app.ui, &w);
                Ok(json!({"workspace": w.name}))
            }
            None => Err(format!("unknown workspace (one of {})", names(&app.ui).join(", "))),
        },
        "window.workspace.reset" => {
            let w = find(&app.ui, &app.ui.workspace.clone()).unwrap_or_default();
            apply(&mut app.ui, &w);
            Ok(json!({"workspace": w.name}))
        }
        "window.workspace.new" => match s("name") {
            Some(n) => save_as(&mut app.ui, &n).map(|_| json!({"workspace": n})),
            None => {
                let n = (1..).map(|i| format!("Workspace {i}")).find(|n| find(&app.ui, n).is_none()).unwrap_or_default();
                app.ui.dialog = Some(Dialog::new("newWorkspace", json!({"name": n})));
                Ok(Value::Null)
            }
        },
        "window.workspace.manage" => {
            app.ui.dialog = Some(Dialog::new("manageWorkspaces", json!({"selected": "", "name": ""})));
            Ok(Value::Null)
        }
        "window.workspace.delete" => delete(&mut app.ui, &s("name").unwrap_or_default()).map(|_| Value::Null),
        "window.workspace.rename" => rename(&mut app.ui, &s("name").unwrap_or_default(), &s("to").unwrap_or_default()).map(|_| Value::Null),
        "window.workspace.list" => Ok(json!({"current": app.ui.workspace, "workspaces": names(&app.ui), "custom": app.ui.custom_workspaces})),
        _ => return None,
    };
    Some(r)
}

/// OK in New Workspace / Manage Workspaces.
pub fn confirm(app: &mut VectorcraftApp) -> Result<Value, String> {
    let Some(d) = app.ui.dialog.clone() else { return Err("no dialog open".into()) };
    let r = match d.kind.as_str() {
        "newWorkspace" => save_as(&mut app.ui, &d.str("name")).map(|_| Value::Null),
        _ => Ok(Value::Null),
    };
    if r.is_ok() {
        app.ui.dialog = None;
    }
    r
}

pub fn show(app: &mut VectorcraftApp, ctx: &egui::Context) {
    let Some(mut d) = app.ui.dialog.clone() else { return };
    let t = Tokens::get(ctx);
    let (mut ok, mut cancel) = (false, false);
    let mut action: Option<(&str, Value)> = None;
    let manage = d.kind == "manageWorkspaces";
    crate::dialogs::modal::show(ctx, "Workspace", egui::Id::new("dialog-workspace"), -40.0, 22, |ui| {
        ui.set_width(340.0);
        crate::dialogs::modal::heading(ui, if manage { tl!("Manage Workspaces") } else { tl!("New Workspace") });
        ui.add_space(12.0);
        if manage {
            egui::Frame::NONE.fill(t.input).stroke(egui::Stroke::new(1.0, t.input_border)).inner_margin(egui::Margin::same(4)).show(ui, |ui| {
                ui.set_min_height(140.0);
                ui.set_width(ui.available_width());
                for n in names(&app.ui) {
                    let builtin = is_builtin(&n);
                    let shown = crate::panels::label_or_name(&n, builtin);
                    let text = if builtin { egui::RichText::new(shown).color(t.text_dim) } else { egui::RichText::new(shown).color(t.text) };
                    if ui.selectable_label(d.str("selected") == n, text).clicked() && !builtin {
                        d.fields.insert("selected".into(), json!(n));
                        d.fields.insert("name".into(), json!(n));
                    }
                }
            });
            ui.add_space(8.0);
            let sel = d.str("selected");
            ui.horizontal(|ui| {
                let mut name = d.str("name");
                if ui.add(egui::TextEdit::singleline(&mut name).desired_width(180.0)).changed() {
                    d.fields.insert("name".into(), json!(name));
                }
                if ui.button(tl!("New")).on_hover_text(tl!("Save the current layout under this name")).clicked() {
                    action = Some(("window.workspace.new", json!({"name": d.str("name")})));
                }
                ui.add_enabled_ui(!sel.is_empty(), |ui| {
                    if ui.button(tl!("Rename")).clicked() {
                        action = Some(("window.workspace.rename", json!({"name": sel, "to": d.str("name")})));
                    }
                    if ui.button(tl!("Delete")).clicked() {
                        action = Some(("window.workspace.delete", json!({"name": sel})));
                    }
                });
            });
        } else {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(tl!("Name:")).color(t.text_dim));
                let mut name = d.str("name");
                let r = ui.add(egui::TextEdit::singleline(&mut name).desired_width(240.0));
                if r.changed() {
                    d.fields.insert("name".into(), json!(name));
                }
            });
            ui.label(egui::RichText::new(tl!("Saves the current bars, toolbar and panel layout.")).color(t.text_dim).size(11.0));
        }
        ui.add_space(16.0);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if widgets::primary_button(ui, tl!("OK")).clicked() {
                ok = true;
            }
            ui.add_space(8.0);
            if !manage && widgets::secondary_button(ui, tl!("Cancel")).clicked() {
                cancel = true;
            }
        });
    });
    if !manage && ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
        ok = true;
    }
    app.ui.dialog = Some(d);
    if let Some((id, p)) = action {
        match run_command(app, id, &p) {
            Some(Err(e)) => app.status(e),
            _ => {
                if let Some(d) = app.ui.dialog.as_mut() {
                    d.fields.insert("selected".into(), json!(""));
                }
            }
        }
    }
    if cancel {
        app.ui.dialog = None;
    } else if ok && let Err(e) = confirm(app) {
        app.status(e);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn essentials_is_the_default_layout() {
        let ui = UiState::default();
        assert_eq!(capture(&ui, ESSENTIALS), builtins()[0]);
        assert_eq!(ui.workspace, ESSENTIALS);
    }

    #[test]
    fn apply_classic_then_reset_to_essentials_keeps_brightness() {
        let mut ui = UiState { brightness: crate::theme::Brightness::Light, ..Default::default() };
        let w = find(&ui, "Essentials Classic").unwrap();
        apply(&mut ui, &w);
        assert!(ui.control_bar && ui.view.rulers && ui.toolbar_advanced);
        assert_eq!(ui.open_panel.as_deref(), Some("color"));
        assert_eq!(ui.workspace, "Essentials Classic");
        let w = find(&ui, "essentials").unwrap();
        apply(&mut ui, &w);
        assert!(!ui.control_bar && !ui.view.rulers);
        assert_eq!(ui.brightness, crate::theme::Brightness::Light);
    }

    #[test]
    fn save_apply_round_trip() {
        let mut ui = UiState { control_bar: true, dock_tab: DockTab::Layers, open_panel: Some("stroke".into()), ..Default::default() };
        save_as(&mut ui, "Mine").unwrap();
        let w = find(&ui, ESSENTIALS).unwrap();
        apply(&mut ui, &w);
        assert!(!ui.control_bar);
        let w = find(&ui, "Mine").unwrap();
        apply(&mut ui, &w);
        assert!(ui.control_bar && ui.dock_tab == DockTab::Layers && ui.open_panel.as_deref() == Some("stroke"));
        // Survives the UiState JSON round trip.
        let back: UiState = serde_json::from_value(serde_json::to_value(&ui).unwrap()).unwrap();
        assert_eq!(back.custom_workspaces, ui.custom_workspaces);
        assert_eq!(back.workspace, "Mine");
    }

    #[test]
    fn workspace_saved_without_dock_widths_loads_and_keeps_the_current_widths() {
        let old: Workspace = serde_json::from_value(json!({"name": "Old", "dock": true, "openPanel": "stroke"})).unwrap();
        assert!(old.dock_widths.is_empty());
        let mut ui = UiState { custom_workspaces: vec![old], ..Default::default() };
        ui.dock_widths.insert(DOCK_GROUP.into(), 410.0);
        let w = find(&ui, "Old").unwrap();
        apply(&mut ui, &w);
        assert!(ui.dock_widths_pending.is_empty());
        assert_eq!(ui.dock_widths.get(DOCK_GROUP), Some(&410.0));
        // Saved again, the widths go with it.
        save_as(&mut ui, "New").unwrap();
        let back: Workspace = serde_json::from_value(serde_json::to_value(find(&ui, "New").unwrap()).unwrap()).unwrap();
        assert_eq!(back.dock_widths.get(DOCK_GROUP), Some(&410.0));
    }

    #[test]
    fn builtin_names_are_protected() {
        let mut ui = UiState::default();
        assert!(save_as(&mut ui, "Painting").is_err());
        assert!(save_as(&mut ui, "  ").is_err());
        assert!(delete(&mut ui, "Essentials").is_err());
        save_as(&mut ui, "A").unwrap();
        assert!(rename(&mut ui, "A", "Web").is_err());
        rename(&mut ui, "A", "B").unwrap();
        assert_eq!(ui.workspace, "B");
        delete(&mut ui, "B").unwrap();
        assert_eq!(ui.workspace, ESSENTIALS);
    }

    #[test]
    fn all_requested_workspaces_exist() {
        let ui = UiState::default();
        for n in ["Essentials", "Essentials Classic", "Painting", "Typography", "Layout", "Tracing", "Web", "Automation"] {
            assert!(find(&ui, n).is_some(), "{n}");
        }
    }
}
