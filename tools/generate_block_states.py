#!/usr/bin/env python3
"""Regenerate block registry + block-state tables from the official data generator reports.

Inputs (vendored from `java -DbundlerMainClass=net.minecraft.data.Main -jar server.jar --reports`):
  vanilla-data/reports/block_registry_26_1_2.json  (the `minecraft:block` slice of registries.json)
  vanilla-data/reports/blocks_26_1_2.json          (the full blocks.json state report)

Outputs (all fully generated, do not edit by hand):
  src/block_metadata/registry_data_{a,b,c}.rs      (1:1 protocol-ordered block registry)
  src/block_states/state_data_{a,b,c,d}.rs         (full per-block state definitions)

The per-block property order in blocks.json's `properties` map does NOT always match
the actual state-id cartesian ordering (e.g. chest, piston_head). We therefore derive
the true ordering empirically from each block's `states` list: a property's stride is
the index of the first state whose value differs from state 0; sorting by descending
stride yields the cartesian order (last property varies fastest). The generator then
verifies every one of the 29k+ states reconstructs exactly before writing anything.
"""

import itertools
import json
import os
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
REPORTS = os.path.join(ROOT, "vanilla-data", "reports")

# Java `Blocks` constant names that are not just `upper(path)`.
FIELD_NAME_OVERRIDES = {
    "minecraft:potted_azalea_bush": "POTTED_AZALEA",
    "minecraft:potted_flowering_azalea_bush": "POTTED_FLOWERING_AZALEA",
}

HEADER = "// GENERATED FILE - DO NOT EDIT. Regenerate with tools/generate_block_states.py\n"


def load_registry():
    with open(os.path.join(REPORTS, "block_registry_26_1_2.json"), encoding="utf-8") as handle:
        entries = json.load(handle)["minecraft:block"]["entries"]
    ordered = sorted(entries.items(), key=lambda item: item[1]["protocol_id"])
    for index, (name, info) in enumerate(ordered):
        assert info["protocol_id"] == index, f"non-contiguous registry id at {name}"
    return [name for name, _ in ordered]


def load_blocks():
    with open(os.path.join(REPORTS, "blocks_26_1_2.json"), encoding="utf-8") as handle:
        return json.load(handle)


def derive_property_order(block):
    """Cartesian property order, outermost first (last property varies fastest)."""
    properties = block.get("properties", {})
    states = block["states"]
    strides = {}
    for name in properties:
        first_value = states[0]["properties"][name]
        for index, state in enumerate(states):
            if state["properties"][name] != first_value:
                strides[name] = index
                break
        else:
            raise AssertionError(f"property {name} never changes value")
    return sorted(properties, key=lambda name: -strides[name])


def verify_block(name, block, order):
    properties = block.get("properties", {})
    states = block["states"]
    ids = [state["id"] for state in states]
    assert ids == list(range(ids[0], ids[0] + len(ids))), f"{name}: ids not contiguous"
    combos = itertools.product(*[properties[prop] for prop in order]) if order else [()]
    for state, combo in zip(states, combos):
        expected = dict(zip(order, combo))
        assert state.get("properties", {}) == expected, f"{name}: state order mismatch"
    defaults = [state["id"] for state in states if state.get("default")]
    assert len(defaults) == 1, f"{name}: expected exactly one default state"
    return ids[0], defaults[0]


def rust_str_slice(values):
    return "&[" + ", ".join(f'"{value}"' for value in values) + "]"


def generate_registry(registry):
    chunks = split_even(registry, 3)
    offset = 0
    for letter, chunk in zip("abc", chunks):
        lines = [
            HEADER,
            "use super::{block, BlockRegistryEntry};\n",
            "\n",
            "pub(super) const ENTRIES: &[BlockRegistryEntry] = &[\n",
        ]
        for index, name in enumerate(chunk, start=offset):
            field = FIELD_NAME_OVERRIDES.get(name, name.split(":", 1)[1].upper())
            lines.append(f'    block({index}, "{field}", "{name}"),\n')
        lines.append("];\n")
        path = os.path.join(ROOT, "src", "block_metadata", f"registry_data_{letter}.rs")
        with open(path, "w", encoding="utf-8") as handle:
            handle.writelines(lines)
        offset += len(chunk)
    print(f"registry: {offset} entries across {len(chunks)} files")


def split_even(items, parts):
    size = (len(items) + parts - 1) // parts
    return [items[start : start + size] for start in range(0, len(items), size)]


def generate_states(registry, blocks):
    # Intern value lists, properties, and property lists to keep the output small.
    value_lists = {}
    properties = {}
    property_lists = {}

    def intern_values(values):
        key = tuple(values)
        if key not in value_lists:
            value_lists[key] = f"V{len(value_lists)}"
        return value_lists[key]

    def intern_property(name, values):
        key = (name, tuple(values))
        if key not in properties:
            properties[key] = f"P{len(properties)}"
        return properties[key]

    def intern_property_list(props):
        key = tuple(props)
        if key not in property_lists:
            property_lists[key] = f"L{len(property_lists)}"
        return property_lists[key]

    entries = []
    total_states = 0
    for name in registry:
        block = blocks[name]
        order = derive_property_order(block)
        base, default = verify_block(name, block, order)
        block_properties = block.get("properties", {})
        prop_consts = [
            intern_property(prop, block_properties[prop]) for prop in order
        ]
        for prop in order:
            intern_values(block_properties[prop])
        list_const = intern_property_list(prop_consts) if prop_consts else "EMPTY"
        entries.append((name, base, default, list_const))
        total_states += len(block["states"])

    shared = [
        HEADER,
        "use super::StateProperty;\n",
        "\n",
        "pub(super) const EMPTY: &[StateProperty] = &[];\n",
    ]
    for values, const in value_lists.items():
        shared.append(f"const {const}: &[&str] = {rust_str_slice(values)};\n")
    for (name, values), const in properties.items():
        shared.append(
            f'const {const}: StateProperty = StateProperty {{ name: "{name}", '
            f"values: {value_lists[tuple(values)]} }};\n"
        )
    for props, const in property_lists.items():
        joined = ", ".join(props)
        shared.append(f"pub(super) const {const}: &[StateProperty] = &[{joined}];\n")

    chunks = split_even(entries, 3)
    files = []
    for letter, chunk in zip("bcd", chunks):
        lines = [
            HEADER,
            "use super::state_data_a::*;\n",
            "use super::BlockStateEntryData;\n",
            "\n",
            "pub(super) const ENTRIES: &[BlockStateEntryData] = &[\n",
        ]
        for name, base, default, list_const in chunk:
            lines.append(
                f'    BlockStateEntryData {{ registry_id: "{name}", '
                f"base_state_id: {base}, default_state_id: {default}, "
                f"properties: {list_const} }},\n"
            )
        lines.append("];\n")
        files.append((f"state_data_{letter}.rs", lines))
    files.insert(0, ("state_data_a.rs", shared))

    out_dir = os.path.join(ROOT, "src", "block_states")
    os.makedirs(out_dir, exist_ok=True)
    for file_name, lines in files:
        with open(os.path.join(out_dir, file_name), "w", encoding="utf-8") as handle:
            handle.writelines(lines)
    print(
        f"states: {len(entries)} blocks, {total_states} states, "
        f"{len(value_lists)} value lists, {len(properties)} properties, "
        f"{len(property_lists)} property lists"
    )
    return total_states


def main():
    registry = load_registry()
    blocks = load_blocks()
    assert set(registry) == set(blocks), "registry and blocks report disagree"
    generate_registry(registry)
    total_states = generate_states(registry, blocks)
    print(f"done: {len(registry)} blocks / {total_states} states")
    return 0


if __name__ == "__main__":
    sys.exit(main())
