mod state;

pub use state::{PlaneSettings, Settings, settings_from_value};
pub fn items() -> Vec<crate::modal::Item<Settings>> {
    vec![
        crate::modal::Item {
            label: "axis-lines",
            value: |settings| yes(settings.plane.axis_lines).to_owned(),
            activate: |settings| settings.plane.axis_lines = !settings.plane.axis_lines,
        },
        crate::modal::Item {
            label: "grid-lines",
            value: |settings| yes(settings.plane.grid_lines).to_owned(),
            activate: |settings| settings.plane.grid_lines = !settings.plane.grid_lines,
        },
    ]
}

fn yes(value: bool) -> &'static str {
    if value { "on" } else { "off" }
}
