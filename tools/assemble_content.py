#!/usr/bin/env python3
"""Assemble the package's `assets/` and `data/` for MineColonies slice 1, plus the generated art.

The flat SDK package keeps `assets/` and `data/` at the mod root, beside `horizon-mod.toml`
and `src/`; the manifest itself is hand-written there and is NOT produced here (it carries
the `[native]` pin, which is not content).

The **art is ours** (`tools/gen_assets.py`, a documented palette). The **ids** are the mod's:
`blockhuttownhall`, `blockhutfarmer`, `scythe` and the `citizen` entity type all exist upstream.
Every declaration below names a vanilla template, because a content block copies a vanilla
block's behaviour rather than porting the mod's Java class.

Rerun to rebuild the folder (it deletes its own generated data/ first):

    python3 tools/assemble_content.py
    python3 tools/assemble_content.py --check     # data/ + assets/ match a fresh assembly
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path
import shutil
import sys

import gen_assets

ROOT = Path(__file__).resolve().parents[1]
PACKAGE = ROOT
MOD = "minecolonies"


def block(
    *,
    template: str,
    hardness: float,
    resistance: float,
    sounds: str,
    occludes: bool,
    facing: bool,
) -> dict:
    declaration: dict = {
        "template": template,
        "properties": {
            "hardness": hardness,
            "explosion_resistance": resistance,
            "occludes": occludes,
            "requires_tool": False,
            "sounds": sounds,
        },
        "item": True,
    }
    if facing:
        declaration["state_properties"] = {
            "facing": {"type": "enum", "values": ["north", "south", "east", "west"]}
        }
        declaration["default_state"] = {"facing": "north"}
    return declaration


def data_files() -> dict[str, bytes]:
    def j(value: object) -> bytes:
        return (json.dumps(value, indent=2, sort_keys=True) + "\n").encode()

    out: dict[str, bytes] = {}
    # Hut blocks: hardness 10 and blast resistance infinite upstream
    # (`AbstractColonyBlock.java:70,75`); 1200 is vanilla's highest (obsidian), so the port's
    # block is blast-proof in practice. `sounds` names a vanilla *block* whose sound type is
    # copied (oak planks: wood, as `SoundType.WOOD` upstream), and the block occludes as
    # little as upstream's `noOcclusion`.
    out[f"data/{MOD}/horizon/block/blockhuttownhall.json"] = j(
        block(
            template="minecraft:stone",
            hardness=10.0,
            resistance=1200.0,
            sounds="minecraft:oak_planks",
            occludes=False,
            facing=True,
        )
    )
    out[f"data/{MOD}/horizon/block/blockhutfarmer.json"] = j(
        block(
            template="minecraft:stone",
            hardness=10.0,
            resistance=1200.0,
            sounds="minecraft:oak_planks",
            occludes=False,
            facing=True,
        )
    )
    # The farmer's tool: upstream's scythe is a plain tool item.
    out[f"data/{MOD}/horizon/item/scythe.json"] = j(
        {"template": "minecraft:iron_hoe", "components": {"minecraft:max_damage": 250}}
    )
    # The citizen is a living type over the villager class, with the citizen's health
    # (`CitizenConstants.BASE_MAX_HEALTH = 20`).
    out[f"data/{MOD}/horizon/entity_type/citizen.json"] = j(
        {
            "template": "minecraft:villager",
            "attributes": {"minecraft:max_health": 20.0},
        }
    )
    out[f"data/{MOD}/horizon/creative_tab/minecolonies.json"] = j(
        {
            "title": f"itemGroup.{MOD}",
            "icon": f"{MOD}:blockhuttownhall",
            "items": [
                f"{MOD}:blockhuttownhall",
                f"{MOD}:blockhutfarmer",
                f"{MOD}:scythe",
            ],
        }
    )
    for hut in ("blockhuttownhall", "blockhutfarmer"):
        out[f"data/{MOD}/loot_table/blocks/{hut}.json"] = j(
            {
                "type": "minecraft:block",
                "pools": [
                    {
                        "bonus_rolls": 0.0,
                        "entries": [{"type": "minecraft:item", "name": f"{MOD}:{hut}"}],
                        "rolls": 1.0,
                        # 26.3's singular typed condition (a `conditions` list never matches).
                        "condition": {"type": "minecraft:survives_explosion"},
                    }
                ],
                "random_sequence": f"{MOD}:blocks/{hut}",
            }
        )
    out[f"data/{MOD}/loot_table/entities/citizen.json"] = j(
        {"type": "minecraft:entity", "pools": []}
    )
    # Recipes: upstream's town hall recipe needs `structurize:sceptergold`, a parent mod this
    # campaign has not ported, so the recipe could never match at this pin. The port's recipe
    # keeps the shape and uses vanilla ingredients (a recorded upstream-data fit, see notes.md).
    out[f"data/{MOD}/recipe/blockhuttownhall.json"] = j(
        {
            "type": "minecraft:crafting_shaped",
            "category": "misc",
            "key": {
                "P": "#minecraft:planks",
                "C": "minecraft:crafting_table",
            },
            "pattern": ["PPP", "PCP", "PPP"],
            "result": {"count": 1, "id": f"{MOD}:blockhuttownhall"},
        }
    )
    out[f"data/{MOD}/recipe/blockhutfarmer.json"] = j(
        {
            "type": "minecraft:crafting_shaped",
            "category": "misc",
            "key": {
                "P": "#minecraft:planks",
                "H": "minecraft:iron_hoe",
            },
            "pattern": ["PPP", "PHP", "PPP"],
            "result": {"count": 1, "id": f"{MOD}:blockhutfarmer"},
        }
    )
    # Upstream's scythe is not a crafting-table recipe at all (the jar carries none; the
    # builder's crafter recipes make the tools), so this shape is the port's own. Its first
    # version was `II `/` S `/` S ` — exactly vanilla's iron hoe, which shadowed it and made the
    # scythe uncraftable (measured in game 2026-11-21, the `constructionstick` class). The same
    # two iron and two sticks in a shape no vanilla recipe owns: the blade split, the shaft below.
    out[f"data/{MOD}/recipe/scythe.json"] = j(
        {
            "type": "minecraft:crafting_shaped",
            "category": "equipment",
            "key": {"I": "minecraft:iron_ingot", "S": "minecraft:stick"},
            "pattern": ["I I", " S ", " S "],
            "result": {"count": 1, "id": f"{MOD}:scythe"},
        }
    )
    out["data/minecraft/tags/block/mineable/axe.json"] = j(
        {"values": [f"{MOD}:blockhuttownhall", f"{MOD}:blockhutfarmer"]}
    )
    return out


def write() -> list[str]:
    written = gen_assets.write(PACKAGE)
    files = data_files()
    for name, payload in sorted(files.items()):
        path = PACKAGE / name
        path.parent.mkdir(parents=True, exist_ok=True)
        if not path.exists() or path.read_bytes() != payload:
            path.write_bytes(payload)
            written.append(name)
    return written


def check() -> int:
    problems = []
    for name, payload in data_files().items():
        path = PACKAGE / name
        if not path.is_file() or path.read_bytes() != payload:
            problems.append(f"differs or missing: {name}")
    generated = {p.relative_to(PACKAGE).as_posix() for p in (PACKAGE / "data").rglob("*") if p.is_file()}
    extra = generated - set(data_files())
    problems.extend(f"not generated by this script: {name}" for name in sorted(extra))
    if problems:
        for problem in problems:
            print(problem, file=sys.stderr)
        return 1
    return gen_assets.check()


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    if args.check:
        return check()
    if (PACKAGE / "data").is_dir():
        shutil.rmtree(PACKAGE / "data")
    written = write()
    print(json.dumps({"content": str(PACKAGE), "changed": written}, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
