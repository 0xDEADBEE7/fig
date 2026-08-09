//! Terminal-friendly figures rendered from JSON.

mod layout;
mod line;
mod model;
mod render;

pub use model::{DataPoint, Edge, Figure, Graph, LineFigure, LineSeries, Node};
pub use render::{RenderOptions, render};

/// Parse and validate a figure from tagged JSON.
pub fn from_json(input: &str) -> anyhow::Result<Figure> {
    let figure: Figure = serde_json::from_str(input)?;
    figure.validate()?;
    Ok(figure)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_tagged_line_figure() {
        let figure =
            from_json(r#"{"type":"line","series":[{"label":"a","points":[{"x":1,"y":2}]}]}"#)
                .unwrap();
        assert!(matches!(figure, Figure::Line(_)));
    }

    #[test]
    fn requires_a_figure_type() {
        let error = from_json(r#"{"nodes":[],"edges":[]}"#).unwrap_err();
        assert!(error.to_string().contains("type"));
    }
}
