# Controls

Keyboard translation belongs in one control layer. The terminal session should convert key events into semantic actions and must not contain figure-specific navigation logic.

## Actions

```rust
pub enum Action {
    PanHorizontal(f64),
    PanVertical(f64),
    Zoom(f64),
    Previous,
    Next,
    FocusNextRegion,
    FocusPreviousRegion,
    Activate,
    Back,
    Reset,
    Quit,
}
```

The directional values are interpreted by the viewport; figure implementations should not need to know key names.

## Default map

```text
h -> PanHorizontal(-1)       l -> PanHorizontal(1)
j -> PanVertical(-1)         k -> PanVertical(1)
J -> Zoom(out)               K -> Zoom(in)
H -> Previous                L -> Next
Tab -> FocusNextRegion       Shift-Tab -> FocusPreviousRegion
Enter -> Activate            Backspace -> Back
r -> Reset                   q -> Quit
```

## Region-aware dispatch

The active region determines the meaning of otherwise shared navigation keys:

- `Visualization`: panning, zooming, and `H/L` selection are active.
- `Detail`: `j/k` move through visible detail items; `Enter` opens a nested item; `Backspace` returns to the parent.
- `Tab` and `Shift-Tab` change the active region without changing selection or focus.

A detail component may expose a tree or menu model. It should consume semantic navigation events and report whether an action was handled, allowing the session to fall back to region traversal where appropriate.

## Remapping

Use a `Controls` value containing a map from terminal keys to `Action`. The default map is constructed by `Controls::default()`; alternate maps can be supplied to the session later.

Remapping should be an input concern only. Renderers and figure interaction implementations consume actions, not keys. Region focus should also be represented semantically so custom bindings can target it without coupling to terminal key codes.
