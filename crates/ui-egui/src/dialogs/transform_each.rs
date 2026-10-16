//! Object → Transform → Transform Each: scale, move and rotate every selected object about its own
//! reference point, with reflection, randomness, Transform Objects, Transform Patterns, Scale
//! Corners and Scale Strokes & Effects (`object.transformEach`), previewed live on the canvas. Its Scale and Move sliders, Rotate dial
//! and reflection, reference point and Random options are the Transform effect's too.
//!
//! Fields: `scaleH`, `scaleV` (%), `moveH`, `moveV` (pt), `rotate` (°), `reflectX`, `reflectY`,
//! `random`, `reference` (0..8, the 9-point grid), `copy`, `objects`, `patterns`, `corners`,
//! `strokes` (the last two default to the preferences, which OK updates) and `preview`. It opens on
//! the values of its last OK, Copy excepted.

use serde_json::{Value, json};

use super::transform::{add_object_options, add_scale_options, object_options, scale_options, with_scale_options};
use super::{DialogSpec, form};
use crate::VectorcraftApp;
use crate::state::Dialog;
use crate::theme::Tokens;
use crate::widgets;

/// The dialog kind of Transform Each.
pub const KIND: &str = "transformEach";

const CMD: &str = "object.transformEach";

pub(super) const SPEC: DialogSpec =
    DialogSpec { heading: |_| tl!("Transform Each").into(), body, confirm, preview: true, min_width: 360.0, ..DialogSpec::FORM };

/// The fields the dialog keeps from its last OK. Copy starts off every time: it is an action, not
/// a setting.
const REMEMBERED: [&str; 9] = ["scaleH", "scaleV", "moveH", "moveV", "rotate", "reflectX", "reflectY", "random", "reference"];

/// The fields the dialog opens on: the values it was last confirmed with, else the defaults.
pub fn fields(app: &VectorcraftApp) -> Value {
    let mut f = json!({"scaleH": 100, "scaleV": 100, "moveH": 0, "moveV": 0, "rotate": 0, "reflectX": false, "reflectY": false, "random": false, "reference": 4, "copy": false});
    for k in REMEMBERED {
        if let Some(v) = app.ui.transform_each.get(k) {
            f[k] = v.clone();
        }
    }
    f
}

/// The command's parameters without the scale options.
fn params(d: &Dialog) -> Value {
    let f = |k: &str, v: f64| d.f64(k, v);
    json!({
        "scaleH": f("scaleH", 100.0), "scaleV": f("scaleV", 100.0), "moveH": f("moveH", 0.0), "moveV": f("moveV", 0.0),
        "rotate": f("rotate", 0.0), "reflectX": d.bool("reflectX"), "reflectY": d.bool("reflectY"), "random": d.bool("random"),
        "reference": f("reference", 4.0).clamp(0.0, 8.0) as u64, "copy": d.bool("copy"),
    })
}

/// Width of the label column and the value fields of [`sections`].
const LABEL_W: f32 = 72.0;
const FIELD_W: f32 = 70.0;
/// Largest scale (%) the fields take (as `object.transformEach` and the Transform effect).
const MAX_SCALE: f64 = 100_000.0;

/// The Scale, Move and Rotate sections of Transform Each and the Transform effect: sliders for
/// `scaleH` and `scaleV` (rails 0–200 %) and for `moveH` and `moveV` (rails ±1000 pt, fields in
/// `units`), and an angle dial for `rotate` (counter-clockwise). The fields take values past the
/// rails.
pub(super) fn sections(ui: &mut egui::Ui, d: &mut Dialog, units: vectorcraft_doc::Unit) {
    let rail = Tokens::get(ui.ctx()).input_border;
    let track = move |_| rail;
    let axes = |h, v| [(h, tl!("Horizontal:")), (v, tl!("Vertical:"))];
    widgets::subheader(ui, tl!("Scale"));
    for (key, label) in axes("scaleH", "scaleV") {
        widgets::label_row(ui, label, LABEL_W, |ui| {
            form::slider_rail(ui, d, key, 0.0..=200.0, &track);
            if let Some(n) = widgets::range_field(ui, ("te", key), d.f64(key, 100.0), -MAX_SCALE..=MAX_SCALE, "%", 2, FIELD_W) {
                d.fields.insert(key.into(), json!(n));
            }
        });
    }
    widgets::subheader(ui, tl!("Move"));
    for (key, label) in axes("moveH", "moveV") {
        widgets::label_row(ui, label, LABEL_W, |ui| {
            form::slider_rail(ui, d, key, -1000.0..=1000.0, &track);
            form::length(ui, d, key, units, FIELD_W);
        });
    }
    widgets::subheader(ui, tl!("Rotate"));
    widgets::label_row(ui, tl!("Angle:"), LABEL_W, |ui| {
        let angle = d.f64("rotate", 0.0);
        let dialed = widgets::angle_dial(ui, "te-dial", angle, 28.0);
        if let Some(a) = widgets::plain_field(ui, ("te", "rotate"), angle, "°", 2, FIELD_W).or(dialed) {
            d.fields.insert("rotate".into(), json!(a));
        }
    });
}

/// The Reflect X, Reflect Y, reference point and Random options of Transform Each and the
/// Transform effect.
pub(super) fn reflect_options(ui: &mut egui::Ui, d: &mut Dialog) {
    form::check(ui, d, "reflectX", tl!("Reflect X"));
    form::check(ui, d, "reflectY", tl!("Reflect Y"));
    ui.horizontal(|ui| {
        let cur = d.f64("reference", 4.0).clamp(0.0, 8.0) as usize;
        if let Some(i) = widgets::reference_point(ui, cur) {
            d.fields.insert("reference".into(), json!(i));
        }
        widgets::dim_label(ui, tl!("Reference Point"));
    });
    form::check(ui, d, "random", tl!("Random"));
}

fn body(app: &mut VectorcraftApp, ui: &mut egui::Ui, d: &mut Dialog) -> bool {
    let units = app.session.general_unit();
    ui.horizontal_top(|ui| {
        ui.vertical(|ui| sections(ui, d, units));
        ui.add_space(16.0);
        ui.vertical(|ui| {
            widgets::subheader(ui, tl!("Options"));
            ui.add_space(4.0);
            object_options(app, ui, d);
            scale_options(app, ui, d);
            reflect_options(ui, d);
            form::check(ui, d, "copy", tl!("Copy"));
        });
    });
    let mut p = params(d);
    add_object_options(app, d, &mut p);
    add_scale_options(app, d, &mut p);
    form::preview(app, ui, d, "Transform Each", CMD, p);
    false
}

fn confirm(app: &mut VectorcraftApp, d: &Dialog) -> Result<Value, String> {
    let mut p = params(d);
    add_object_options(app, d, &mut p);
    let p = with_scale_options(app, d, p)?;
    let r = form::commit_preview(app, CMD, p)?;
    let kept = REMEMBERED.into_iter().filter_map(|k| Some((k.to_string(), d.fields.get(k)?.clone())));
    app.ui.transform_each = Value::Object(kept.collect());
    Ok(r)
}

#[cfg(test)]
mod tests {
    use super::*;
    use vectorcraft_doc::NodeId;
    use vectorcraft_engine::Session;

    #[test]
    fn ok_transforms_each_and_keeps_the_scale_options() {
        let mut app = VectorcraftApp::new(Session::new(), Default::default());
        app.run("file.new", json!({"width": 300, "height": 300})).unwrap();
        let id = app.run("shape.rectangle", json!({"x": 0, "y": 0, "width": 50, "height": 50})).unwrap()["id"].as_u64().unwrap();
        app.run("select.all", json!({})).unwrap();
        app.run("prefs.set", json!({"key": "scaleStrokes", "value": true})).unwrap();
        app.run("ui.menuDialog", json!({ "command": CMD })).unwrap();
        assert_eq!(app.ui.dialog.as_ref().map(|d| d.kind.as_str()), Some(KIND));
        // One drawn frame: the checkboxes start from the preferences.
        let ctx = egui::Context::default();
        crate::theme::install_fonts(&ctx);
        let mut out = ctx.run_ui(egui::RawInput::default(), |ui| super::super::show(&mut app, ui.ctx()));
        out.textures_delta.clear();
        let d = app.ui.dialog.as_mut().unwrap();
        assert_eq!(d.fields["strokes"], json!(true));
        d.fields.insert("scaleH".into(), json!(200));
        d.fields.insert("scaleV".into(), json!(200));
        d.fields.insert("strokes".into(), json!(false));
        super::super::confirm(&mut app).unwrap();
        assert!(app.ui.dialog.is_none());
        assert!(!app.session.prefs.scale_strokes, "OK keeps the option as the preference");
        let n = app.session.doc().unwrap().doc.node(NodeId(id)).unwrap().clone();
        assert!((n.geometric_bounds().unwrap().width() - 100.0).abs() < 1e-6);
        assert_eq!(n.appearance.stroke_width(), 1.0, "strokes kept their weight");
        let (cmd, p) = app.session.journal.last().unwrap();
        assert_eq!((cmd.as_str(), &p["strokes"], &p["objects"]), (CMD, &json!(false), &json!(true)), "#1448: Transform Objects goes along");
    }

    /// The dialog opens on the values it was last confirmed with, Copy excepted (#1732).
    #[test]
    fn reopens_on_the_last_confirmed_values() {
        let mut app = VectorcraftApp::new(Session::new(), Default::default());
        app.run("file.new", json!({"width": 300, "height": 300})).unwrap();
        app.run("shape.rectangle", json!({"x": 0, "y": 0, "width": 50, "height": 50})).unwrap();
        app.run("select.all", json!({})).unwrap();
        app.run("ui.menuDialog", json!({ "command": CMD })).unwrap();
        let d = app.ui.dialog.as_mut().unwrap();
        for (k, v) in [
            ("scaleH", json!(150)),
            ("moveV", json!(12)),
            ("rotate", json!(30)),
            ("reflectX", json!(true)),
            ("reference", json!(0)),
            ("copy", json!(true)),
        ] {
            d.fields.insert(k.into(), v);
        }
        super::super::confirm(&mut app).unwrap();
        assert!(app.ui.dialog.is_none());
        app.run("ui.menuDialog", json!({ "command": CMD })).unwrap();
        let d = app.ui.dialog.as_ref().unwrap();
        assert_eq!(d.kind, KIND);
        assert_eq!((d.f64("scaleH", 0.0), d.f64("scaleV", 0.0), d.f64("moveV", 0.0), d.f64("rotate", 0.0)), (150.0, 100.0, 12.0, 30.0));
        assert_eq!((d.bool("reflectX"), d.f64("reference", 4.0), d.bool("copy")), (true, 0.0, false));
        let saved: crate::state::UiState = serde_json::from_slice(&serde_json::to_vec(&app.ui).unwrap()).unwrap();
        assert_eq!(saved.transform_each["scaleH"], json!(150), "kept with the other UI state");
    }
}
