# Figure behavior

Each figure implements selection, detail, focus styling, and optional focus viewport behavior.

## Histogram

- Selectable item: bucket.
- `H` / `L`: previous / next bucket.
- Detail: bucket label, total, and values by series.
- Focus: emphasize the selected bucket and dim other buckets.
- Focus viewport: unchanged.

The existing histogram table becomes the common detail box rather than a histogram-only layout concept.

## Line figure

- Selectable item: line series.
- `H` / `L`: previous / next series.
- Detail: series label, point count, and useful range/statistics.
- Focus: emphasize the selected series and dim all other series.
- Focus viewport: unchanged.

Series colors and legend entries should use the same emphasis state as the plotted lines.

## Graph

- Selectable item: node.
- `H` / `L`: previous / next node.
- Detail: node ID, label, degree, and connected node IDs.
- Focus: emphasize the selected node, preserve stronger emphasis for direct neighbors and incident edges, and dim unrelated nodes and labels.
- Focus viewport: recenter on the focused node while preserving the current viewport dimensions.

Graph layout positions should be calculated once per session and reused across redraws. Otherwise force-layout variation can make focus movement appear unstable.

## Focus recentering

When a node gains focus, center the viewport on its layout position. Do not recenter during panning or zooming. Focus is lost on those actions and regained only through `H` / `L` navigation.
