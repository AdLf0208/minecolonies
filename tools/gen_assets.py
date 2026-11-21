#!/usr/bin/env python3
"""Generate every `assets/minecolonies/` file — the port's own art, from a named palette.

MineColonies' code is GPL-3.0, but `assets/minecolonies/**` in its jar is **All Rights
Reserved** (`assets/minecolonies/LICENSE`, 19 bytes). Not one upstream texture, model or lang
file is read or copied here: only the mod's **ids** (`blockhuttownhall`, `blockhutfarmer`,
`scythe`, `citizen`) and the **shapes** the real models suggest are followed, and every pixel
comes from the palette below.

The output is deterministic: the PNGs are written with stored (uncompressed) deflate blocks, so
the same palette always gives the same bytes, and `--check` re-renders into a temp folder and
compares byte for byte.

    python3 tools/gen_assets.py            # write assets/minecolonies/
    python3 tools/gen_assets.py --check    # exit 1 if a file differs from what it would write

Palette (one named colour per family; every texture below is composed from these only):

    TOWN_HALL_TIMBER  #9a7b52   the town hall's walls
    TOWN_HALL_ROOF    #5d402c   its roof band
    TOWN_HALL_WINDOW  #8cc7d8   its lit window
    FARMER_TIMBER     #8b6d3f   the farmer's hut's walls
    FARMER_ROOF       #6b8f3a   its thatch band
    FARMER_DOOR       #4a331e   its door
    LUMBERJACK_TIMBER #6e5433  the lumberjack's hut's walls
    LUMBERJACK_ROOF   #3f6b2a  its forest-green thatch band
    LUMBERJACK_DOOR   #2f2413  its door
    MINER_STONE       #6b6f77  the mine's stone face
    MINER_TIMBER      #4a3826  the mine's head-frame beams
    MINER_LAMP        #e3c05a  the mine's lamp
    SCYTHE_BLADE      #c8d2da   the scythe's blade
    SCYTHE_HANDLE     #7a522e   the scythe's shaft
    CITIZEN_SKIN      #d9a47a   the citizen's skin
    CITIZEN_SHIRT     #3f6f8f   the citizen's shirt
    TRANSPARENT       alpha 0    everything the models leave open

Model templates (one of these three produces every model JSON):

    cube   a full block: `minecraft:block/cube_all`
    slab   a half-height block, built from six explicit faces
    cross  two crossed planes, `minecraft:block/cross`

The models used by slice 1: the town hall is a `cube`; the farmer's hut is a `slab` (a
placeholder low hut — its real model is a composite of dozens of boxes); the scythe's item
model is a `cross` (a placeholder tool silhouette).
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path
import struct
import sys
import zlib

ROOT = Path(__file__).resolve().parents[1]
PACKAGE = ROOT
ASSETS = PACKAGE / "assets" / "minecolonies"

PALETTE = {
    "town_hall_timber": (0x9A, 0x7B, 0x52),
    "town_hall_roof": (0x5D, 0x40, 0x2C),
    "town_hall_window": (0x8C, 0xC7, 0xD8),
    "farmer_timber": (0x8B, 0x6D, 0x3F),
    "farmer_roof": (0x6B, 0x8F, 0x3A),
    "farmer_door": (0x4A, 0x33, 0x1E),
    "lumberjack_timber": (0x6E, 0x54, 0x33),
    "lumberjack_roof": (0x3F, 0x6B, 0x2A),
    "lumberjack_door": (0x2F, 0x24, 0x13),
    "miner_stone": (0x6B, 0x6F, 0x77),
    "miner_timber": (0x4A, 0x38, 0x26),
    "miner_lamp": (0xE3, 0xC0, 0x5A),
    "scythe_blade": (0xC8, 0xD2, 0xDA),
    "scythe_handle": (0x7A, 0x52, 0x2E),
    "citizen_skin": (0xD9, 0xA4, 0x7A),
    "citizen_shirt": (0x3F, 0x6F, 0x8F),
}
TRANSPARENT = (0, 0, 0, 0)


def rgba(name: str) -> tuple[int, int, int, int]:
    r, g, b = PALETTE[name]
    return (r, g, b, 255)


def png(width: int, height: int, pixels: list[tuple[int, int, int, int]]) -> bytes:
    """An RGBA PNG with stored deflate blocks: same input, same bytes, on any Python."""
    raw = bytearray()
    for y in range(height):
        raw.append(0)  # filter: none
        for x in range(width):
            raw.extend(pixels[y * width + x])

    def chunk(kind: bytes, payload: bytes) -> bytes:
        return (
            struct.pack(">I", len(payload))
            + kind
            + payload
            + struct.pack(">I", zlib.crc32(kind + payload) & 0xFFFFFFFF)
        )

    # Stored deflate: a header, then blocks of at most 65535 bytes, then adler32.
    deflate = bytearray(b"\x78\x01")
    data = bytes(raw)
    for start in range(0, len(data), 0xFFFF):
        block = data[start : start + 0xFFFF]
        final = 1 if start + 0xFFFF >= len(data) else 0
        deflate.append(final)
        deflate.extend(struct.pack("<HH", len(block), len(block) ^ 0xFFFF))
        deflate.extend(block)
    deflate.extend(struct.pack(">I", zlib.adler32(data) & 0xFFFFFFFF))
    header = struct.pack(">IIBBBBB", width, height, 8, 6, 0, 0, 0)
    return b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", header) + chunk(b"IDAT", bytes(deflate)) + chunk(b"IEND", b"")


def town_hall_texture() -> bytes:
    """Timber walls, a roof band across the top, one lit window."""
    pixels = []
    for y in range(16):
        for x in range(16):
            if y < 4:
                colour = rgba("town_hall_roof")
            elif 8 <= y < 12 and 6 <= x < 10:
                colour = rgba("town_hall_window")
            else:
                colour = rgba("town_hall_timber")
            pixels.append(colour)
    return png(16, 16, pixels)


def farmer_texture() -> bytes:
    """Timber walls, a thatch band across the top, a door in the middle."""
    pixels = []
    for y in range(16):
        for x in range(16):
            if y < 3:
                colour = rgba("farmer_roof")
            elif 6 <= y < 16 and 6 <= x < 10:
                colour = rgba("farmer_door")
            else:
                colour = rgba("farmer_timber")
            pixels.append(colour)
    return png(16, 16, pixels)


def lumberjack_texture() -> bytes:
    """Timber walls, a forest-green thatch band across the top, a door off centre."""
    pixels = []
    for y in range(16):
        for x in range(16):
            if y < 3:
                colour = rgba("lumberjack_roof")
            elif 6 <= y < 16 and 5 <= x < 9:
                colour = rgba("lumberjack_door")
            else:
                colour = rgba("lumberjack_timber")
            pixels.append(colour)
    return png(16, 16, pixels)


def miner_texture() -> bytes:
    """A stone face inside a timber head-frame, with a lamp at its centre."""
    pixels = []
    for y in range(16):
        for x in range(16):
            if y == 0 or x == 0:
                colour = rgba("miner_timber")
            elif 7 <= y < 9 and 7 <= x < 9:
                colour = rgba("miner_lamp")
            else:
                colour = rgba("miner_stone")
            pixels.append(colour)
    return png(16, 16, pixels)


def scythe_texture() -> bytes:
    """A staff across the sprite with a blade along the top-left arc."""
    pixels = []
    for y in range(16):
        for x in range(16):
            on_shaft = x in (10, 11) and 3 <= y < 15
            on_blade = 1 <= y <= 3 and 2 <= x <= 11
            if on_blade:
                colour = rgba("scythe_blade")
            elif on_shaft:
                colour = rgba("scythe_handle")
            else:
                colour = TRANSPARENT
            pixels.append(colour)
    return png(16, 16, pixels)


def citizen_texture() -> bytes:
    """A flat placeholder skin: a shirt below, skin above.

    Slice 1 has no renderer for it (the citizen is drawn with its villager template's model), so
    it is a 16x16 swatch; a later slice binds it to a native mob renderer.
    """
    pixels = []
    for y in range(16):
        for x in range(16):
            pixels.append(rgba("citizen_shirt") if y >= 8 else rgba("citizen_skin"))
    return png(16, 16, pixels)


def cube(texture: str) -> dict:
    return {"parent": "minecraft:block/cube_all", "textures": {"all": texture}}


def slab(texture: str) -> dict:
    """A half-height block: one box, six explicit faces."""
    faces = {
        side: {"uv": [0, 0, 16, 8], "texture": f"#{texture}"}
        for side in ("north", "east", "south", "west")
    }
    faces["up"] = {"uv": [0, 0, 16, 16], "texture": f"#{texture}"}
    faces["down"] = {"uv": [0, 0, 16, 16], "texture": f"#{texture}", "cullface": "down"}
    return {
        "textures": {
            "particle": texture,
            "side": texture,
            "top": texture,
            "bottom": texture,
        },
        "elements": [
            {
                "from": [0, 0, 0],
                "to": [16, 8, 16],
                "faces": faces,
            }
        ],
    }


def cross(texture: str) -> dict:
    return {"parent": "minecraft:block/cross", "textures": {"cross": texture}}


def blockstate(model: str) -> dict:
    """Upstream's four `facing` variants; a mental note only, the shape is ours."""
    return {
        "variants": {
            "facing=east": {"model": model},
            "facing=south": {"model": model, "y": 90},
            "facing=west": {"model": model, "y": 180},
            "facing=north": {"model": model, "y": 270},
        }
    }


def item_definition(model: str) -> dict:
    return {"model": {"type": "minecraft:model", "model": model}}


def files() -> dict[str, bytes]:
    def defj(value: object) -> bytes:
        return (json.dumps(value, indent=2, sort_keys=True) + "\n").encode()

    out: dict[str, bytes] = {}
    out["assets/minecolonies/textures/block/blockhuttownhall.png"] = town_hall_texture()
    out["assets/minecolonies/textures/block/blockhutfarmer.png"] = farmer_texture()
    out["assets/minecolonies/textures/block/blockhutlumberjack.png"] = lumberjack_texture()
    out["assets/minecolonies/textures/block/blockhutminer.png"] = miner_texture()
    out["assets/minecolonies/textures/item/scythe.png"] = scythe_texture()
    out["assets/minecolonies/textures/entity/citizen.png"] = citizen_texture()

    out["assets/minecolonies/models/block/blockhuttownhall.json"] = defj(
        cube("minecolonies:block/blockhuttownhall")
    )
    out["assets/minecolonies/models/block/blockhutfarmer.json"] = defj(
        slab("minecolonies:block/blockhutfarmer")
    )
    out["assets/minecolonies/models/block/blockhutlumberjack.json"] = defj(
        slab("minecolonies:block/blockhutlumberjack")
    )
    out["assets/minecolonies/models/block/blockhutminer.json"] = defj(
        slab("minecolonies:block/blockhutminer")
    )
    out["assets/minecolonies/models/item/blockhuttownhall.json"] = defj(
        {"parent": "minecolonies:block/blockhuttownhall"}
    )
    out["assets/minecolonies/models/item/blockhutfarmer.json"] = defj(
        {"parent": "minecolonies:block/blockhutfarmer"}
    )
    out["assets/minecolonies/models/item/blockhutlumberjack.json"] = defj(
        {"parent": "minecolonies:block/blockhutlumberjack"}
    )
    out["assets/minecolonies/models/item/blockhutminer.json"] = defj(
        {"parent": "minecolonies:block/blockhutminer"}
    )
    out["assets/minecolonies/models/item/scythe.json"] = defj(cross("minecolonies:item/scythe"))

    out["assets/minecolonies/blockstates/blockhuttownhall.json"] = defj(
        blockstate("minecolonies:block/blockhuttownhall")
    )
    out["assets/minecolonies/blockstates/blockhutfarmer.json"] = defj(
        blockstate("minecolonies:block/blockhutfarmer")
    )
    out["assets/minecolonies/blockstates/blockhutlumberjack.json"] = defj(
        blockstate("minecolonies:block/blockhutlumberjack")
    )
    out["assets/minecolonies/blockstates/blockhutminer.json"] = defj(
        blockstate("minecolonies:block/blockhutminer")
    )

    out["assets/minecolonies/items/blockhuttownhall.json"] = defj(
        item_definition("minecolonies:item/blockhuttownhall")
    )
    out["assets/minecolonies/items/blockhutfarmer.json"] = defj(
        item_definition("minecolonies:item/blockhutfarmer")
    )
    out["assets/minecolonies/items/blockhutlumberjack.json"] = defj(
        item_definition("minecolonies:item/blockhutlumberjack")
    )
    out["assets/minecolonies/items/blockhutminer.json"] = defj(
        item_definition("minecolonies:item/blockhutminer")
    )
    out["assets/minecolonies/items/scythe.json"] = defj(item_definition("minecolonies:item/scythe"))

    # Our own words, our own file: the ids are the mod's, the names are written here.
    out["assets/minecolonies/lang/en_us.json"] = defj(
        {
            "block.minecolonies.blockhuttownhall": "Town Hall",
            "block.minecolonies.blockhutfarmer": "Farmer's Hut",
            "block.minecolonies.blockhutlumberjack": "Forester's Hut",
            "block.minecolonies.blockhutminer": "Mine",
            "item.minecolonies.scythe": "Scythe",
            "entity.minecolonies.citizen": "Citizen",
            "itemGroup.minecolonies": "MineColonies",
        }
    )
    return out


def write(target: Path) -> list[str]:
    written = []
    for name, data in sorted(files().items()):
        path = target / name
        path.parent.mkdir(parents=True, exist_ok=True)
        if not path.exists() or path.read_bytes() != data:
            path.write_bytes(data)
            written.append(name)
    return written


def check() -> int:
    on_disk = {
        p.relative_to(PACKAGE).as_posix(): p.read_bytes() for p in ASSETS.rglob("*") if p.is_file()
    }
    expected = files()
    problems = []
    for name, data in expected.items():
        if on_disk.get(name) != data:
            problems.append(f"differs or missing: {name}")
    for name in sorted(set(on_disk) - set(expected)):
        problems.append(f"not generated by this script: {name}")
    if problems:
        for problem in problems:
            print(problem, file=sys.stderr)
        return 1
    print(f"assets/minecolonies/: {len(expected)} files, byte-identical to the palette's output")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--check", action="store_true", help="verify instead of write")
    args = parser.parse_args()
    if args.check:
        return check()
    written = write(PACKAGE)
    print(json.dumps({"assets": str(ASSETS), "changed": written}, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
