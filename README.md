# fig

`fig` is a deliberately small interactive terminal graph viewer. It reads one
JSON graph from standard input and renders it with Unicode braille characters.

```sh
cargo run -- examples/graph.json
```

Standard input is also supported:

```sh
cat examples/graph.json | cargo run -- -
```

```json
{
  "type": "graph",
  "nodes": [{ "id": "a", "label": "API", "metadata": { "port": 8080 } }],
  "edges": [{ "from": "a", "to": "b" }]
}
```

Interactive sessions need an input file so standard input remains available for
keyboard events.

Controls: `h/j/k/l` pan, `J/K` zoom, `H/L` previous/next node, `Enter` open
information, `Shift-Tab` toggle labels, `b`, `q`, or `x` back (or quit from an
unfocused visualisation), `/` search, `?` help, and `r` reset.
