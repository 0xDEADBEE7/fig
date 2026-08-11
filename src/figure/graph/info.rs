use crate::models::Graph;

pub fn information(
    graph: &Graph,
    focus: Option<usize>,
    width: usize,
    height: usize,
) -> Vec<String> {
    let Some(index) = focus else {
        return centered(
            "No node selected. Press H or L to select one.",
            width,
            height,
        );
    };
    let node = &graph.nodes[index];
    let mut lines = vec![
        graph.node_label(index).to_owned(),
        format!("id: {}", node.id),
    ];
    if !node.metadata.is_null() {
        lines.push("metadata:".to_owned());
        lines.extend(
            serde_json::to_string_pretty(&node.metadata)
                .expect("JSON values can always be serialized")
                .lines()
                .map(str::to_owned),
        );
    }
    lines.extend([
        String::new(),
        "H/L: previous/next node    b/q/x: back".to_owned(),
    ]);
    lines
        .into_iter()
        .map(|line| line.chars().take(width).collect())
        .chain(std::iter::repeat(String::new()))
        .take(height)
        .collect()
}

fn centered(text: &str, width: usize, height: usize) -> Vec<String> {
    (0..height)
        .map(|row| {
            if row == height / 2 {
                format!("{:^width$}", text, width = width)
            } else {
                String::new()
            }
        })
        .collect()
}
