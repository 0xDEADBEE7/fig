use super::*;

#[test]
fn dimmed_series_emits_terminal_controls_not_literal_escapes() {
    let series = [
        Series {
            label: "selected".into(),
            points: vec![],
        },
        Series {
            label: "dimmed".into(),
            points: vec![],
        },
    ];
    let output = draw(
        &[vec![(0, 0), (10, 10)], vec![(0, 10), (10, 0)]],
        &series,
        12,
        6,
        Some(0),
        Plane::new(false, false),
    )
    .join("\n");
    assert!(output.contains('\x1b'));
    assert!(!output.contains(r"\x1b"));
}

#[test]
fn selected_line_stays_visible_at_an_overlap() {
    let mut canvas = background(Plane::new(false, false), 2, 1);
    draw_series(&mut canvas, &[(0, 0), (3, 3)], Tone::Normal);
    draw_series(&mut canvas, &[(0, 3), (3, 0)], Tone::Dim);
    assert_eq!(canvas[0][0].tone, Tone::Normal);
}

#[test]
fn plotted_dots_replace_grid_and_axis_characters() {
    let mut canvas = background(Plane::new(false, false), 1, 1);
    canvas[0][0].text = Some('─');

    put_dot(&mut canvas, 0, 0, Tone::Normal);

    assert_eq!(canvas[0][0].text, None);
    assert_eq!(render_row(canvas.remove(0)), "⠁");
}
