# Rendering architecture

Rendering should receive common interaction context while retaining figure-specific drawing code.

## Render context

`RenderOptions` should describe dimensions and output preferences. Interaction state should be passed separately:

```rust
pub struct RenderContext<'a> {
    pub viewport: Viewport,
    pub selection: Option<&'a Selection>,
    pub focused: bool,
    pub color: bool,
}
```

This removes figure-specific fields such as `selected_index` from general render options.

## Detail box and navigation

Every visualization reserves a detail region below the plot, but the region is an interactive component rather than a fixed block that must fit all content at once. It should support scrolling, cursor navigation, and nested data.

Figure interaction produces semantic content:

```rust
pub struct DetailContent {
    pub title: String,
    pub root: DetailNode,
}
```

A `DetailNode` can represent a value, a row, or an expandable group. The shared detail component owns borders, cursor styling, scrolling, expansion, truncation, ANSI styling, and height limits. Figure types only provide semantic data.

When detail content exceeds the available height, keep the visualization dimensions stable and scroll the detail region independently. Avoid shrinking the plot to force every detail row into view.

## Region focus

The frame contains two navigable regions:

```text
visualization
(detail region)
status / controls
```

`Tab` moves focus from visualization to detail; `Shift-Tab` reverses it. The active region should be visually indicated in the border or status line.

In visualization focus, `h/j/k/l`, `J/K`, and `H/L` retain their normal meanings. In detail focus, `j/k` navigate detail rows, `Enter` opens an expandable item, and `Backspace` collapses it or moves to its parent. This leaves the existing visualization bindings intact while making large or nested details usable.

## Emphasis

Figure renderers should classify elements using a shared concept:

```rust
pub enum Emphasis {
    Normal,
    Focused,
    Related,
    Dimmed,
}
```

The style layer maps emphasis to color or tone. `Related` is needed for graph neighbors; line and histogram renderers may only use focused and dimmed.

## Layout

The terminal frame should have a consistent structure:

```text
visualization
(detail box)
status / controls
```

The shared layout must calculate available plot height so detail content cannot overwrite the visualization or status row.
