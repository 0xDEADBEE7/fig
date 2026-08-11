# fig

`fig` renders structured JSON figures as terminal-friendly Unicode graphics.
It currently supports force-directed network graphs, multi-series line charts, and stacked histograms.

## Usage

```sh
cargo run -- examples/simple.json
cargo run -- examples/line.json --width 80 --height 24
cat examples/line.json | cargo run -- -
```

Install it locally with `cargo install --path .`, then run `fig`. ANSI colors are
selected automatically for line series when output is a terminal; pass `--no-color`
for plain output.

## Figure types

Every input has a required `type` discriminator.

### Graph

```json
{
  "type": "graph",
  "nodes": [
    { "id": "api", "label": "API" },
    { "id": "db", "label": "Database" }
  ],
  "edges": [{ "from": "api", "to": "db" }]
}
```

`label` defaults to `id`. Edges may use `source` and `target` in place of
`from` and `to`.

### Line

```json
{
  "type": "line",
  "x_label": "Time",
  "y_label": "Value",
  "series": [
    {
      "label": "observed",
      "points": [{ "x": 0, "y": 1 }, { "x": 1, "y": 4 }]
    },
    {
      "label": "forecast",
      "points": [{ "x": 0, "y": 3 }, { "x": 1, "y": 2 }]
    }
  ]
}
```

Points are joined in the order supplied. All series share automatically computed
x and y bounds and are drawn together with distinct colors and a legend.

Dense series are automatically reduced to the available plot width using
Largest-Triangle-Three-Buckets sampling, which retains endpoints and visually
significant peaks instead of simply dropping every nth point. Use `--x-min` and
`--x-max` to select a viewport (the basis for zooming and panning):

```sh
python3 examples/generate_sine.py | cargo run -- - --width 80
python3 examples/generate_sine.py | cargo run -- - --x-min 6 --x-max 12
```

### Histogram

Histograms use ordered buckets and explicitly declared stacked series. Missing values
are treated as zero; bucket labels determine left-to-right order and series order
determines bottom-to-top stack and legend order.

```json
{
  "type": "histogram",
  "x_label": "Latency",
  "y_label": "Count",
  "series": [
    { "label": "success" },
    { "label": "failure" }
  ],
  "buckets": [
    { "label": "0–10 ms", "values": { "success": 42, "failure": 3 } },
    { "label": "10–25 ms", "values": { "success": 67 } }
  ]
}
```

Histogram values must be finite and non-negative. Values use the series labels as
keys; unknown labels, duplicate series labels, empty buckets, and empty series are
rejected.

## Interactive figures

Start a redraw-in-place session with any supported JSON figure (interactive mode needs standard input for keyboard events):

```sh
python3 examples/generate_sine.py > /tmp/sine.json
cargo run -- /tmp/sine.json --interactive
cargo run -- /tmp/sine.json --interactive --width 60 --height 18
```

Use `h`/`l` to pan left/right, `j`/`k` to pan down/up, `J`/`K` to zoom out/in, `r` to reset, and `q` or Escape to quit. On histograms, `h`/`l` select
the previous/next bucket instead; the selected bucket stays visible and its values
appear in the data table. Panning is intentionally unbounded: the viewport may move
beyond the data and show blank space, with data clipped at the viewport edges. The
session uses the terminal's alternate screen and redraws after key and resize events,
leaving normal scrollback intact.
In interactive mode, `--width` and `--height` are hard maximums for the entire
session; the canvas also shrinks if the terminal is smaller. Line wrapping is
disabled while the session runs so neither the figure nor status row can scroll
and corrupt the next frame.



# Examples

```bash
2.25e5   │                   ⢸⠁                                          ⢀⡰⠒⠊
         │                   ⢸                                         ⣠⠋⠁   ⢀⠄
         │                   ⡸                                     ⢀⡠⠊⠉   ⣀⣀⠤⠃
         │                   ⡇                                    ⡜⠁    ⡠⡜⠂
         │                   ⡇                        ⢠⡀        ⢠⠴⠁  ⡔⠒⡩⠊⠁
         │                  ⢀⠇                   ⢀⣀⡠⠤⡶⠏        ⢠⠃ ⡠⠊⠉ ⡰⠁
         │             ⢀⡤⠤⠖⠚⠉                  ⢀⠤⠎ ⡰⠚       ⣀⠔⠒⣉⠤⠜⢀⡠⠔⠒⠁
         │           ⢀⠤⠎                    ⡠⠔⠉⠁  ⢰⠁    ⣀⣀⠎⠉⢀⡠⠋⢀⡠⠼⠁
         │        ⢠⠤⠒⠁                     ⢰⠁   ⣀⣀⡎   ⡔⠉ ⢀⣀⠎⠁⢰⠊⠁
         │        ⢸                     ⡀ ⡠⠎⢀⣀⠤⠼  ⢀⠤⠒⠉ ⡠⠊⠁ ⣀⠤⠊     ⢀⡠⠄
         │        ⢸                  ⡤⠶⠾⠉⢉⣀⠎⠁    ⡤⢺⠤⠔⠒⠉  ⢀⡰⠁    ⢀⣀⠔⠁
         │        ⢸              ⣀⠴⢩⠽⢁⣀⡠⠋⠁    ⢀⡶⠾⠤⠚    ⢀⠔⠁  ⢀⣠⠔⠚⠁
         │        ⢸         ⣀⠤⣒⣒⠞⠒⣊⠝⠒⠁      ⣀⠎⠉     ⣀⠤⠒⠁ ⡠⠔⠊⠁
         │        ⢸         ⡷⠊⢀⡠⠒⠊       ⣤⡲⠝⠊    ⢀⡰⠉  ⢀⡠⠜⢀⠤      ⢀⡠⠔⠒⠊⠉
         │        ⢸     ⢀⡔⣒⣉⡧⠒⠁        ⣤⡾⠋   ⢀⡠⠒⠊⠁  ⢠⠤⡣⠔⠒⠊    ⣀⠤⠒⠁
         │        ⢸  ⢀⡰⢊⡡⠔⠁⡇         ⡴⠞⠁ ⢠⣔⠤⠊⠁⢀⠤⠔⠒⢉⡩⠝⠉   ⢀⡠⢒⠊⠉
         │      ⢠⠒⡞⠊⠉⠁⢀⡜  ⢸⣀⣀⣀⣀⣠⠋⠉⢩⠭⠟⠁⢀⣀⡤⡞⢀⠤⣒⣉⠕⠒⠊⠉⠁ ⢀⣀⠤⠔⢊⣁⣰⡡⠄
         │      ⡜ ⡇ ⣀⣠⠃⢰⠒⢹⠋    ⣀⣀⣮⡃ ⣠⠒⣞⣒⢞⠞⠓⠉  ⢀⣀⡠⣒⣒⡩⠥⣔⠝⠛⠉⠁
         │    ⢠⠒⠁ ⡧⠜  ⢀⡼⠤⠇  ⣤⣶⠟⡭⠋⡡⢴⣚⡖⠛⠓⠉⠁ ⢀⣠⣔⡲⠕⣒⠭⠔⠒⠒⠉
         │   ⢀⣸  ⢰⡇ ⡤⢤⠚⠃⡤⢤⣔⣾⢿⠤⢚⢤⢴⣳⣋⡜  ⣀⣀⡰⠚⠓⣁⡠⠒⠉⣀⠤⠄
         │   ⣼⠥  ⡎⣇⣸⠤⠓⡲⢚⣽⣿⡾⣆⡮⢔⣽⣓⠕⣁⢤⠤⠶⠋⢀⣠⣒⣊⡹⠒⠒⠊⠉
         │   ⣿⣠⣤⣴⡻⡿⡳⡲⠭⡜⣏⣿⣯⣽⣶⣿⣻⠯⠒⠊⣉⣱⠖⠚⠛⠛
         │ ⢠⣶⣿⣧⣼⣀⣴⣿⣿⣿⣟⣿⣯⣟⣿⠽⣾⠽⠓  ⣠⡻
         │ ⢸⢸⣷⣿⣿⣿⢻⣟⣉⣴⡿⡟⠋⢉⠵⠊⠁⡠⢤⠮⠭⠜⠁
         │ ⢸⣸⣿⣿⣿⣴⣿⡿⣿⢷⣿⣳⠮⠛⠊⡭⠝⠊⠁
         │ ⣿⣿⣿⣿⣿⣿⠿⠿⠛⡫⠤⠔⠒⠋⠉
         │⣤⣿⣿⣿⣿⠧⠔⠒⠊⠉
0        │⠉⠉⠉
         └─────────────────────────────────────────────────────────────────────
         0                            Usage event                           283
```

---

```bash

1.57e7   │                                                                  ⡰⢠⠃
         │                                                                ⢀⡴⠥⠃
         │                                                               ⢀⡞⠁
         │                                                              ⡴⠝
         │                                                            ⢀⡼⠁
         │                                                           ⡠⡞
         │                                                         ⢀⢼⠊
         │                                                        ⡠⡞⠁   ⢀⠎⠁
         │                                                      ⢀⣰⠏    ⡠⠃
         │                                             ⡀       ⡰⡱⠁  ⢀⣀⠔⠁
         │                                           ⢀⠔⠁     ⢠⢪⠒⠁  ⡠⠊
         │                                          ⣠⠃     ⢠⢒⠥⠃  ⡠⠊⢀⠔⠁
         │                                        ⢀⡜ ⡤⠔  ⢀⠔⢁⠎   ⡰⠁⡔⠁
         │                                       ⡤⢃⡰⠚  ⢀⢔⣁⠔⠁  ⡰⠉⡰⠉
         │                                     ⢠⠾⠋⠁  ⢀⢔⠏⠁   ⡰⡩⠔⠉     ⢀⡠
         │                                   ⢠⠞⠉   ⢀⢤⡺⠁   ⡠⣊⠜     ⣀⠔⠊⠁
         │                                 ⢀⠔⠁   ⢀⠤⡺⠁   ⡠⣪⠮    ⢀⡠⠊
         │                             ⡠⠂⢀⠔⠁   ⢀⠤⡳⠉  ⢀⣠⠮⠊⠁   ⡠⠔⠁
         │                          ⢀⡰⠉⢀⡴⠋   ⢀⢔⠕⠉  ⢀⣠⠷⠃  ⣀⣠⠞⠉
         │                   ⢠⠂   ⢀⠖⠊⣀⠴⠃   ⡠⡲⠕⠁ ⢀⣠⠴⠋⠁ ⣀⡴⠞⠊⣀⠤
         │                  ⡴⠁  ⣀⠔⠁⣠⠞⠊  ⢀⡰⡩⠊  ⣠⠔⠋  ⡠⡴⠭⠃⡠⠔⠊
         │                ⣠⠜⠁ ⡠⠊⢀⣠⠞  ⢀⣀⢎⠕⠊⣀⣠⠶⠛⢁⣀⡴⠮⠝⡩⠔⠒⠉
         │               ⡤⠃⣀⠴⠉⣀⡴⠓⠁ ⣠⡲⠕⠉⣡⣤⡾⢛⣁⠤⠒⢉⣀⠔⠊⠉
         │            ⢀⡰⢋⠤⠖⢁⢤⠮⠊⣀⣤⣲⣝⣁⣤⡶⣟⡿⠷⠖⢉⡠⠔⠒⠁
         │           ⢀⡼⠔⣉⣠⠶⣞⣷⡶⢟⣛⣥⣾⣟⠿⠛⣋⡡⠤⠒⠊⠁    ⡠⠔⠂
         │        ⡠⢤⣔⣫⣴⣿⣿⣶⣿⣿⠿⢿⢿⣛⣋⠭⣒⣊⣉⣀⣀⠤⠤⠤⠤⠤⠔⠉⠉
         │  ⣀⣀⣠⣴⣾⣿⣶⣿⣿⣿⣿⣿⣿⡿⠿⠶⠯⠛⠋⠉⠉⠉
0        │⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉
         └─────────────────────────────────────────────────────────────────────
         0                            Usage event                           283
```

---

```bash
         
         
⠀⠀⠀⠀[Worker]⠒⠒⠒⠤⠤⠤⠤⣀⣀⣀⣀⡀
⠀⠀⠀⠀⠀⠀⠀⢸⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⠉⠉⠉⠉⠒⠒⠒⠒⠤⠤⠤⠤⢄⣀⣀⣀⣀
⠀⠀⠀⠀⠀⠀⠀⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠉⠉⠉[Queue]
⠀⠀⠀⠀⠀⠀⢰⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇
⠀⠀⠀⠀⠀⠀⡎⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇
⠀⠀⠀⠀⠀⢠⠃⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇
⠀⠀⠀⠀⠀⡸⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇
[Database]⢄⣀⣀⣀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠉⠉⠉⠑⠒⠒⠒⠤⠤⠤⢄⣀⣀⣀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠉⠉⠉⠑⠒⠒⠒⠤⠤⠤⢄⣀⣀⣀⠀⠀⠀⠀⠀⡇
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠉⠉⠉[API]
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⠑⠢⢄⡀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⠑⠢⢄⡀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⠑⠢⣀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠉⠒⠤⣀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠉⠒⠤⣀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠉⠒⠤⣀
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀[Auth]

```

---

```bash

7        │                                                                  ⢀⠔⠁
         │                                                                 ⡠⠃
         │⢄⡀                                                             ⡠⠊
         │ ⠈⠒⢄⡀                                                        ⢠⠊
         │    ⠈⠒⢄⡀                                                   ⢀⠔⠁
         │       ⠈⠒⢄⡀                              ⢀⣀⠤⠒⠉⠉⠒⠒⠤⠤⢄⣀⡀   ⢀⠔⠁
         │          ⠈⠒⢄⡀                       ⣀⡠⠔⠊⠁           ⠈⠉⠉⡲⠓⠤⠤⢄⣀⡀
         │             ⠈⠒⢄⡀    ⢀⡠⢄⡀       ⢀⡠⠤⠒⠉                 ⡠⠊      ⠈⠉⠉⠒⠒⠤⠄
         │                ⠈⠒⢄⡠⠒⠁  ⠈⠑⠒⠤⣀⠤⠒⠊⠁                   ⢠⠊
         │               ⢀⡠⠒⠁⠈⠒⢄⡀⢀⡠⠔⠒⠉ ⠉⠒⠢⢄⡀                ⢀⠔⠁
         │            ⢀⡠⠒⠁      ⠈⠁         ⠈⠉⠒⠤⣀⡀         ⢀⠔⠁
         │         ⢀⡠⠒⠁                         ⠈⠑⠢⢄⣀    ⡠⠃
         │      ⢀⡠⠒⠁                                 ⠉⠒⠤⠊
         │   ⢀⡠⠒⠁
         │⢀⡠⠒⠁
1        │⠁
         └─────────────────────────────────────────────────────────────────────
         0                                Time                                3

```

---

```bash
         
         
         
1        │   ⡰⠉⢆              ⢠⠓⢢              ⡔⠑⢆              ⢠⠊⢢
         │  ⢰⠁ ⠘⡄            ⢠⠃  ⢇            ⢸  ⠈⡆            ⢠⠃ ⠈⡆
         │  ⡎   ⢱            ⡎   ⠸⡀           ⡇   ⢸            ⡎   ⢣
         │⡀⢀⠇    ⡇        ⢀⡀⢰⠁    ⡇        ⢀⡀⢸     ⡇        ⢀⣀⢰⠁   ⠘⡄        ⢀⡀
         │⠈⡺⡀    ⢸       ⢠⠃⠈⡽⡀    ⢸       ⡔⠁⠈⡿⡀    ⢣       ⡠⠃ ⡿⡀    ⢱       ⡔⠁
         │⢀⠇⠑⡄   ⠘⡄     ⡰⠁  ⡇⠱⡀   ⠸⡀     ⡜  ⢠⠃⠘⡄   ⢸      ⡰⠁ ⢰⠁⠱⡀   ⠈⡆     ⡜
         │⢸  ⠸⡀   ⡇    ⡰⠁  ⢸  ⠱⡀   ⡇    ⢰⠁  ⢸  ⢱   ⠈⡆    ⢰⠁  ⡸  ⠱⡀   ⢱    ⡸
         │⠇   ⢣   ⢸   ⢠⠃   ⡜   ⢣   ⢱   ⢀⠎   ⡇   ⢇   ⢱   ⢀⠇   ⡇   ⢱   ⠸⡀  ⢠⠃   ⡄
         │    ⠈⢆  ⠘⡄  ⡇    ⡇   ⠈⡆  ⠸⡀ ⢀⠎   ⢠⠃   ⠘⡄  ⠈⡆  ⡜   ⢰⠁    ⢇   ⡇  ⡎   ⢰⠁
         │     ⠈⢆  ⢇ ⡸    ⢸     ⠘⡄  ⡇ ⡸    ⢸     ⠱⡀  ⢱ ⡰⠁   ⡸     ⠘⡄  ⢱ ⡜    ⡜
         │      ⠈⢆ ⠸⡔⠁    ⡇      ⠘⡄ ⢱⡰⠁    ⡇      ⠱⡀ ⢘⡜     ⡇      ⠘⡄ ⠸⡰⠁    ⡇
         │        ⠑⠉⢇    ⢸        ⠈⠒⠊⡆    ⢠⠃       ⠈⠒⠊⢇    ⢸        ⠈⠒⠉⡇    ⢸
         │          ⠸⡀   ⡇           ⢱    ⡜           ⠸⡀   ⡇           ⢸    ⡜
         │           ⢇  ⡸            ⠈⡆  ⢰⠁            ⢇  ⡸             ⡇  ⢠⠃
         │           ⠈⢆⢰⠁             ⠘⡄⢠⠃             ⠘⡄⢠⠃             ⠘⡄⣀⠎
-1       │             ⠁               ⠈⠁               ⠈⠁               ⠈
         └─────────────────────────────────────────────────────────────────────
         0                              radians                          25.133

```
