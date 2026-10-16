//! Shape size dialogs (a click with a shape tool): Rectangle, Rounded Rectangle, Ellipse, Polygon,
//! Star, Line Segment Tool Options and the two grids (size and reference point, divider counts and
//! skews, switches; a double-click on a grid tool opens its dialog to set the tool's options only).

use serde_json::{Value, json};

use vectorcraft_tools::draw2::GRID_ORIGINS;

use super::{DialogSpec, form, run_and_close};
use crate::state::Dialog;
use crate::{VectorcraftApp, widgets};

pub(super) const SPEC: DialogSpec = DialogSpec { heading: |d| title(&d.kind).into(), body, confirm, ..DialogSpec::FORM };

fn title(kind: &str) -> &'static str {
    match kind {
        "rectangle" => tl!("Rectangle"),
        "roundedRectangle" => tl!("Rounded Rectangle"),
        "ellipse" => tl!("Ellipse"),
        "polygon" => tl!("Polygon"),
        "star" => tl!("Star"),
        "rectangularGrid" => tl!("Rectangular Grid"),
        "polarGrid" => tl!("Polar Grid"),
        _ => tl!("Line Segment Tool Options"),
    }
}

/// The grids' divider counts and skews: (rows / concentric, columns / radial).
fn grid_keys(kind: &str) -> ([&'static str; 2], [&'static str; 2]) {
    if kind == "polarGrid" { (["concentric", "radial"], ["concentricSkew", "radialSkew"]) } else { (["rows", "columns"], ["rowSkew", "columnSkew"]) }
}

/// The grid's reference point: its index in [`GRID_ORIGINS`] (unknown: the first).
fn grid_origin(d: &Dialog) -> usize {
    let origin = d.str("origin");
    GRID_ORIGINS.iter().position(|o| *o == origin).unwrap_or(0)
}

/// The form's fields, then a grid's reference point and switches.
fn body(app: &mut VectorcraftApp, ui: &mut egui::Ui, d: &mut Dialog) -> bool {
    form::grid(ui, d, app.session.general_unit());
    let own = match d.kind.as_str() {
        "rectangularGrid" => ("frame", tl!("Use Outside Rectangle As Frame")),
        "polarGrid" => ("compound", tl!("Create Compound Path From Ellipses")),
        _ => return false,
    };
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        if let Some(i) = widgets::corner_point(ui, grid_origin(d)) {
            d.fields.insert("origin".into(), json!(GRID_ORIGINS.get(i).copied().unwrap_or(GRID_ORIGINS[0])));
        }
        widgets::dim_label(ui, tl!("Reference Point"));
    });
    ui.add_space(8.0);
    for (key, label) in [own, ("fill", tl!("Fill Grid"))] {
        form::check(ui, d, key, label);
    }
    false
}

fn confirm(app: &mut VectorcraftApp, d: &Dialog) -> Result<Value, String> {
    if let kind @ ("rectangularGrid" | "polarGrid") = d.kind.as_str() {
        return confirm_grid(app, d, kind);
    }
    let (x, y) = (d.f64("x", 0.0), d.f64("y", 0.0));
    let (id, params) = match d.kind.as_str() {
        "ellipse" => ("shape.ellipse", json!({"x": x, "y": y, "width": d.f64("width", 100.0), "height": d.f64("height", 100.0)})),
        "polygon" => ("shape.polygon", json!({"cx": x, "cy": y, "radius": d.f64("radius", 50.0), "sides": d.f64("sides", 6.0) as u64})),
        "star" => (
            "shape.star",
            json!({"cx": x, "cy": y, "radius1": d.f64("radius1", 50.0), "radius2": d.f64("radius2", 25.0), "points": d.f64("points", 5.0) as u64}),
        ),
        "lineSegment" => {
            let (l, a) = (d.f64("length", 100.0), d.f64("angle", 0.0).to_radians());
            ("shape.line", json!({"x1": x, "y1": y, "x2": x + l * a.cos(), "y2": y - l * a.sin()}))
        }
        _ => (
            "shape.rectangle",
            json!({"x": x, "y": y, "width": d.f64("width", 100.0), "height": d.f64("height", 100.0), "radius": d.f64("radius", 0.0)}),
        ),
    };
    run_at(app, id, params, x, y)
}

/// Run shape command `id` for a click at (`x`, `y`), on the active plane while the perspective grid
/// shows (sized in plane units from the click), and close the dialog.
fn run_at(app: &mut VectorcraftApp, id: &str, params: Value, x: f64, y: f64) -> Result<Value, String> {
    match vectorcraft_engine::perspective_click(&app.session, id, &params, vectorcraft_geom::Point::new(x, y)) {
        Some((id, params)) => run_and_close(app, &id, params),
        None => run_and_close(app, id, params),
    }
}

/// A grid dialog's OK: draws the grid with its reference point at the click, then keeps the values
/// as the tool's options. Opened from the tool (no click point), it only sets the options.
fn confirm_grid(app: &mut VectorcraftApp, d: &Dialog, kind: &str) -> Result<Value, String> {
    // The switches as they are; the typed fields as numbers.
    let (mut p, (counts, skews)) = (form::params(d), grid_keys(kind));
    let (width, height) = (d.f64("width", 100.0), d.f64("height", 100.0));
    p["width"] = json!(width);
    p["height"] = json!(height);
    for (count, skew) in counts.into_iter().zip(skews) {
        p[count] = json!((d.f64(count, 5.0) as u64).min(999));
        p[skew] = json!(d.f64(skew, 0.0));
    }
    let origin = grid_origin(d);
    p["origin"] = json!(GRID_ORIGINS.get(origin).copied().unwrap_or(GRID_ORIGINS[0]));
    let mut options = p.clone();
    if let Some(o) = options.as_object_mut() {
        o.remove("x");
        o.remove("y");
    }
    let set = json!({"tool": kind, "values": options});
    let (Some(x), Some(y)) = (d.fields.get("x").and_then(Value::as_f64), d.fields.get("y").and_then(Value::as_f64)) else {
        return run_and_close(app, "tool.setOption", set);
    };
    // The click is the chosen corner (the reference points are in reading order).
    let x0 = if origin % 2 == 1 { x - width } else { x };
    let y0 = if origin >= 2 { y - height } else { y };
    if let Some(o) = p.as_object_mut() {
        o.remove("origin");
        o.insert("x".into(), json!(x0));
        o.insert("y".into(), json!(y0));
    }
    let result = run_at(app, &format!("shape.{kind}"), p, x0, y0)?;
    app.run("tool.setOption", set)?;
    Ok(result)
}
