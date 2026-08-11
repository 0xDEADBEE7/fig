# Standard visualization controls

This plan defines a reusable interaction model for all figure types.

## Goals

- Standardize keyboard controls across figures.
- Separate viewport, selection, focus, and rendering concerns.
- Give every view a shared, navigable detail region.
- Support figure-specific navigation and focus behavior.
- Leave room for custom key remapping.

## Documents

- [Controls](controls.md): actions, default bindings, remapping, and region focus.
- [Interaction state](state.md): viewport, selection, focus, region, and detail navigation.
- [Rendering](rendering.md): detail panels, emphasis, and render context.
- [Figure behavior](figures.md): histogram, line, and graph semantics.
- [Implementation phases](phases.md): incremental delivery plan.

## Default bindings

| Key | Action |
|---|---|
| `h` / `l` | Pan horizontally in the visualization |
| `j` / `k` | Pan vertically in the visualization; move through detail content when detail has focus |
| `J` / `K` | Zoom out / in in the visualization |
| `H` / `L` | Previous / next selection in the visualization |
| `Tab` / `Shift-Tab` | Focus next / previous region |
| `Enter` | Open or expand the focused detail item |
| `Backspace` | Collapse detail item or return to its parent |
| `r` | Reset visualization and interaction state |
| `q` | Quit |

`Esc` may remain an alias for quit as a convenience, but `q` is the standard quit binding. `Tab` is the default region-switching binding because it is familiar for moving between interface regions and does not conflict with the visualization controls. `Shift-Tab` reverses the cycle.
