use serde_json::Value;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Color {
    Default,
    Dim,
    Bright,
    Red,
    Green,
    Blue,
    Yellow,
    Rgb(u8, u8, u8),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Style {
    pub color: Color,
    pub size: u8,
    pub override_focus: bool,
    pub show_label: Option<bool>,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            color: Color::Default,
            size: 1,
            override_focus: false,
            show_label: None,
        }
    }
}
pub fn style(fig: &Value, enabled: bool) -> Style {
    if !enabled || !fig.is_object() {
        return Style::default();
    }
    Style {
        color: fig
            .get("color")
            .and_then(parse_color)
            .unwrap_or(Color::Default),
        size: fig
            .get("size")
            .and_then(Value::as_u64)
            .unwrap_or(1)
            .clamp(1, 3) as u8,
        override_focus: fig
            .get("override-focus")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        show_label: fig.get("show-label").and_then(Value::as_bool),
    }
}

fn parse_color(value: &Value) -> Option<Color> {
    let text = value.as_str()?.to_ascii_lowercase();
    Some(match text.as_str() {
        "dim" => Color::Dim,
        "bright" => Color::Bright,
        "red" => Color::Red,
        "green" => Color::Green,
        "blue" => Color::Blue,
        "yellow" => Color::Yellow,
        text if text.len() == 7 && text.starts_with('#') => Color::Rgb(
            u8::from_str_radix(&text[1..3], 16).ok()?,
            u8::from_str_radix(&text[3..5], 16).ok()?,
            u8::from_str_radix(&text[5..7], 16).ok()?,
        ),
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_fig_render_options() {
        let value = serde_json::json!({"fig": {"color": "#12abef", "size": 2, "override-focus": true, "show-label": false}});
        assert_eq!(style(&value["fig"], true).size, 2);
        assert!(style(&value["fig"], true).override_focus);
        assert_eq!(style(&value["fig"], true).show_label, Some(false));
    }
}
