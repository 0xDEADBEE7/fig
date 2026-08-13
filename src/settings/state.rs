use serde::{Deserialize, Serialize};

use crate::figure::Plane;

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
#[serde(default)]
pub struct PlaneSettings {
    pub axis_lines: bool,
    pub grid_lines: bool,
}

impl Default for PlaneSettings {
    fn default() -> Self {
        Self {
            axis_lines: true,
            grid_lines: true,
        }
    }
}

impl PlaneSettings {
    pub fn from_plane(plane: Plane) -> Self {
        Self {
            axis_lines: plane.show_axes,
            grid_lines: plane.show_grid,
        }
    }

    pub fn apply(self, plane: &mut Plane) {
        plane.show_axes = self.axis_lines;
        plane.show_grid = self.grid_lines;
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
#[serde(default)]
pub struct Settings {
    pub plane: PlaneSettings,
}

pub fn settings_from_value(value: &serde_json::Value) -> Settings {
    value
        .get("fig")
        .and_then(|fig| fig.get("menu"))
        .cloned()
        .and_then(|menu| serde_json::from_value(menu).ok())
        .unwrap_or_default()
}
