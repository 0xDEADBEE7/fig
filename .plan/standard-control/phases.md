# Implementation phases

## Phase 1: Controls and state

- Add semantic `Action` values.
- Add centralized default `Controls`.
- Move key matching out of figure-specific session branches.
- Make `h/l` pan for histograms.
- Add `H/L` navigation for all figure types.
- Add `Reset` on `r`.
- Introduce `Selection` and `focused` state.

## Phase 2: Shared detail panel

- Define `DetailNode`, `DetailContent`, and `DetailState`.
- Add detail generation for graph and line figures.
- Adapt the histogram table to the shared detail model.
- Add independent detail scrolling and cursor navigation.
- Add `Tab` / `Shift-Tab` region focus.
- Add `Enter` expansion and `Backspace` collapse/navigation.
- Reserve a minimum detail-region height without forcing all content to fit.

## Phase 3: Focus rendering

- Add interaction context to render calls.
- Add shared emphasis/style mapping.
- Dim non-focused line series.
- Highlight and dim histogram buckets.
- Add focused, related, and dimmed graph styling.

## Phase 4: Graph focus viewport

- Cache graph layout positions for the interactive session.
- Implement viewport centering around a focused node.
- Preserve viewport dimensions while recentering.

## Phase 5: Custom mappings

- Allow a caller to provide a `Controls` instance.
- Add configuration-file or CLI remapping only after the default model is stable.

## Acceptance criteria

- Every figure responds consistently to the standard bindings.
- Histogram `h/l` always pans; only `H/L` changes buckets.
- `r` restores the initial viewport and selection state.
- Pan and zoom clear focus.
- `H/L` selects and focuses the next or previous figure item.
- Every figure displays a navigable detail region below its visualization.
- `Tab` and `Shift-Tab` transfer control between visualization and detail regions.
- Detail content can scroll independently of the visualization.
- Nested detail data can be expanded with `Enter` and collapsed with `Backspace`.
- Non-interactive rendering remains unaffected except for shared layout improvements.
