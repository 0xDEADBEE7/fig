#!/usr/bin/env python3
"""Colour selected graph roots green and their descendants red."""

import argparse
import json
from pathlib import Path


ROOT_IDS = frozenset(
    {
        "S131103",
        "S140568",
        "S130163",
        "S139138",
        "S123417",
        "S146190",
    }
)


def descendant_edges(graph: dict, roots: set[str]) -> tuple[set[str], set[int]]:
    children: dict[str, list[tuple[int, str]]] = {}
    for index, edge in enumerate(graph.get("edges", [])):
        children.setdefault(edge["from"], []).append((index, edge["to"]))

    found: set[str] = set()
    edges: set[int] = set()
    pending = list(roots)
    while pending:
        parent = pending.pop()
        for index, child in children.get(parent, []):
            if child in roots:
                continue
            edges.add(index)
            if child not in found:
                found.add(child)
                pending.append(child)
    return found, edges


def set_colour(item: dict, colour: str) -> dict:
    fig = item.get("fig")
    if not isinstance(fig, dict):
        fig = {}
        item["fig"] = fig
    fig["color"] = colour
    return fig


def recolour(graph: dict) -> dict:
    roots = {node["id"] for node in graph.get("nodes", []) if node["id"] in ROOT_IDS}
    red_nodes, red_edges = descendant_edges(graph, roots)

    for node in graph.get("nodes", []):
        if node["id"] in roots:
            set_colour(node, "green")["show-label"] = True
        elif node["id"] in red_nodes:
            set_colour(node, "red")

    for index, edge in enumerate(graph.get("edges", [])):
        if index in red_edges:
            set_colour(edge, "red")
    return graph


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("input", type=Path, help="graph JSON file to recolour in place")
    args = parser.parse_args()

    with args.input.open(encoding="utf-8") as source:
        graph = json.load(source)
    if not isinstance(graph, dict) or not isinstance(graph.get("nodes"), list):
        raise ValueError("the input must be a graph object with a nodes array")

    recolour(graph)
    with args.input.open("w", encoding="utf-8") as destination:
        json.dump(graph, destination, indent=2)
        destination.write("\n")


if __name__ == "__main__":
    main()
