# Interaction state

The session owns shared interaction state while each figure supplies figure-specific selection behavior.

## State model

```rust
pub struct InteractionState {
    pub viewport: Viewport,
    pub selection: Option<Selection>,
    pub focused: bool,
    pub region: Region,
    pub detail: DetailState,
}

pub enum Region {
    Visualization,
    Detail,
}

pub enum Selection {
    GraphNode(usize),
    LineSeries(usize),
    HistogramBucket(usize),
}
```

Indices are internal stable identities. Labels and IDs are display data and must not be used as identity.

`DetailState` should hold the current detail tree, cursor, scroll offset, and expansion path. Selection remains figure-level state; detail navigation should not silently change the selected item unless an explicit detail action requests it.

## Region focus

The default region cycle is:

```text
Visualization -> Detail -> Visualization
```

`Tab` advances through regions and `Shift-Tab` reverses. If a region has no navigable content, traversal skips it. Region focus is independent from figure focus: the visualization can be focused while the detail describes a selected item, and moving into the detail region does not lose visualization focus.

## Detail navigation

Detail content should be modeled as a navigable tree rather than only preformatted text:

```rust
pub struct DetailState {
    pub root: DetailNode,
    pub path: Vec<usize>,
    pub cursor: usize,
    pub scroll: usize,
}
```

`j/k` move the cursor or scroll within the active detail view. `Enter` expands or enters a nested node, while `Backspace` collapses or returns to the parent. The detail renderer may choose a compact row view or a nested menu presentation depending on available height.

## Transitions

| Event | Viewport | Selection | Focus |
|---|---|---|---|
| Initial render | Initial bounds | First item | Lost |
| Pan | Changed | Retained | Lost |
| Zoom | Changed | Retained | Lost |
| `H` / `L` | Figure-defined | Previous/next | Gained |
| `Tab` / `Shift-Tab` | Region changes | Retained | Retained |
| `j` / `k` in detail | Detail cursor/scroll changes | Retained | Retained |
| `Enter` / `Backspace` in detail | Detail expansion changes | Retained | Retained |

## Figure interaction interface

```rust
pub trait FigureInteraction {
    fn first_selection(&self) -> Selection;
    fn next_selection(&self, current: &Selection, direction: NavigationDirection) -> Selection;
    fn detail(&self, selection: &Selection) -> DetailContent;
    fn focus_viewport(&self, selection: &Selection, current: Viewport) -> Viewport;
}
```

`focus_viewport` may return the current viewport when no recentering is appropriate.
