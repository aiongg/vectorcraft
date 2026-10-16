//! Actions panel menu › Action Options… and Set Options…: rename an action or an action set. The
//! sets are saved with the UI preferences, so the new name persists.
//!
//! Fields: `set`, `action` (null for Set Options…) and `name`.

use serde_json::{Value, json};

use super::{DialogSpec, form};
use crate::state::Dialog;
use crate::{VectorcraftApp, widgets};

pub(super) const KIND: &str = "actionOptions";
const LABEL: f32 = 60.0;

pub(super) const SPEC: DialogSpec = DialogSpec { heading, body, confirm, min_width: 320.0, ..DialogSpec::FORM };

/// The menu item's label without its ellipsis.
fn heading(d: &Dialog) -> String {
    let item = if action(d).is_some() { tl!("Action Options…") } else { tl!("Set Options…") };
    item.trim_end_matches('…').to_string()
}

fn index(d: &Dialog, key: &str) -> Option<usize> {
    d.fields.get(key).and_then(Value::as_u64).and_then(|i| usize::try_from(i).ok())
}

fn action(d: &Dialog) -> Option<usize> {
    index(d, "action")
}

/// Open Action Options for action `action` of set `set`, or Set Options for set `set` when
/// `action` is None.
pub fn open(app: &mut VectorcraftApp, set: usize, action: Option<usize>) -> Result<Value, String> {
    let s = app.ui.action_sets.get(set).ok_or("no such action")?;
    let name = match action {
        Some(a) => s.actions.get(a).ok_or("no such action")?.name.clone(),
        None => s.name.clone(),
    };
    app.ui.dialog = Some(Dialog::new(KIND, json!({"set": set, "action": action, "name": name})));
    Ok(Value::Null)
}

/// Rename action `action` of set `set`, or the set itself when `action` is None.
pub fn rename(app: &mut VectorcraftApp, set: usize, action: Option<usize>, name: &str) -> Result<Value, String> {
    let name = name.trim();
    if name.is_empty() {
        return Err("name is empty".into());
    }
    let s = app.ui.action_sets.get_mut(set).ok_or("no such action")?;
    let slot = match action {
        Some(a) => &mut s.actions.get_mut(a).ok_or("no such action")?.name,
        None => &mut s.name,
    };
    *slot = name.to_string();
    Ok(Value::Null)
}

fn body(_: &mut VectorcraftApp, ui: &mut egui::Ui, d: &mut Dialog) -> bool {
    widgets::label_row(ui, tl!("Name:"), LABEL, |ui| {
        form::text(ui, d, "name", 220.0);
    });
    false
}

fn confirm(app: &mut VectorcraftApp, d: &Dialog) -> Result<Value, String> {
    let set = index(d, "set").ok_or("no such action")?;
    let r = rename(app, set, action(d), &d.str("name"))?;
    app.ui.dialog = None;
    Ok(r)
}
