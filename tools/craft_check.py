#!/usr/bin/env python3
"""Craft every recipe the minecolonies package carries, in a real crafting table.

Called by `tools/e2e.sh` with the open game's control socket. The harness's
crafting table exposes the 3x3 grid as slots 100-108 (row-major) and the result
as 200 (`ctl screen`). For each of the three carried recipes this:

1. replaces the table (a fresh block entity, so a stale grid cannot leak in)
   and clears the inventory;
2. gives exactly the recipe's ingredients;
3. places them in the pattern the recipe declares (one right-click per slot);
4. asserts the result's **id and count**;
5. shift-clicks the result into the inventory and asserts it arrived;
6. asserts the grid is empty and the recipe's distinctive ingredient was
   consumed.

Two deliberately wrong patterns must leave the result empty, so the check cannot
pass on a table that crafts whatever it is handed. Exits 1 on the first
mismatch, printing the observed result.

The recipes are the port's own (`data/minecolonies/recipe/*.json`, written by
`tools/assemble_content.py`); upstream's town hall recipe names
`structurize:sceptergold`, which this campaign has not ported, so the shapes keep
their upstream ingredient *layout* with vanilla ingredients (see `notes.md`,
"Upstream defects fixed").

    python3 tools/craft_check.py SOCK CTL X Y Z
"""
import json
import subprocess
import sys

# The crafting table's grid, row-major, and its result.
GRID = {
    "tl": 100, "tm": 101, "tr": 102,
    "ml": 103, "mm": 104, "mr": 105,
    "bl": 106, "bm": 107, "br": 108,
}
RESULT = 200

TOWN_HALL = "minecolonies:blockhuttownhall"
FARMER_HUT = "minecolonies:blockhutfarmer"
SCYTHE = "minecolonies:scythe"
PLANKS = "minecraft:oak_planks"
TABLE = "minecraft:crafting_table"
HOE = "minecraft:iron_hoe"
INGOT = "minecraft:iron_ingot"
STICK = "minecraft:stick"


def main() -> int:
    sock, ctl, x, y, z = sys.argv[1], sys.argv[2], sys.argv[3], sys.argv[4], sys.argv[5]

    def run(*args):
        out = subprocess.run([ctl, "--socket", sock, *args], text=True, capture_output=True)
        try:
            return json.loads(out.stdout)
        except json.JSONDecodeError:
            raise SystemExit(f"ctl {' '.join(args)}: {out.stdout}{out.stderr}")

    def wait(ticks=4):
        run("wait", str(ticks))

    def slots():
        screen = run("screen")
        if "slots" not in screen:
            raise SystemExit(f"no menu is open: {screen}")
        return screen["slots"]

    def find(item):
        for slot in slots():
            if slot.get("item") == item:
                return slot["slot"]
        raise SystemExit(f"missing ingredient {item} in the inventory")

    def result():
        stack = [s for s in slots() if s["slot"] == RESULT]
        if not stack or not stack[0].get("item"):
            return None
        return stack[0]

    def grid():
        return [s for s in slots() if 100 <= s["slot"] <= 108]

    def grid_is_empty():
        return all(not s.get("item") for s in grid())

    def count(item):
        return run("inventory").get("counts", {}).get(item, 0)

    def place(item, targets):
        # One pickup per target: a max-stack-1 ingredient cannot fill two slots
        # from one held stack, while a stack of four can.
        for target in targets:
            slot = find(item)
            run("click-slot", str(slot))
            wait(1)
            run("click-slot", str(target), "right")
            wait(1)
            run("click-slot", str(slot))
            wait(1)

    def setup(ingredients):
        try:
            run("close")
        except SystemExit:
            pass
        wait(4)
        run("setblock", x, y, z, "minecraft:air")
        run("setblock", x, y, z, "minecraft:crafting_table")
        wait(2)
        run("command", f"tp @s {float(x) - 0.5} {y} {float(z) + 0.5}")
        wait(3)
        run("command", "clear @s")
        for item, n in ingredients:
            run("give", item, str(n))
        wait(3)
        run("interact", x, y, z)
        wait(5)

    failures = []

    def check(label, ingredients, placements, want_item, want_count, consumed_item):
        setup(ingredients)
        for item, targets in placements:
            place(item, targets)
        got = result()
        print(f"  {label} -> {json.dumps(got)}")
        if not got or got.get("item") != want_item or got.get("count") != want_count:
            print(f"    grid: {json.dumps(grid())}")
            failures.append(f"{label}: expected {want_count} {want_item}, got {got}")
            return
        run("click-slot", str(RESULT), "shift")
        wait(4)
        made = count(want_item)
        if made < want_count:
            failures.append(f"{label}: the crafted stack reached the inventory as {made}")
        if not grid_is_empty():
            failures.append(f"{label}: the grid still holds ingredients after the craft")
        left = count(consumed_item)
        if left != 0:
            failures.append(f"{label}: {consumed_item} was not consumed ({left} left)")
        run("command", "clear @s")
        wait(2)

    def empty_control(label, ingredients, placements):
        setup(ingredients)
        for item, targets in placements:
            place(item, targets)
        got = result()
        print(f"  {label} -> {json.dumps(got)}")
        if got is not None:
            failures.append(f"{label}: the control crafted {got['item']} x{got['count']}")

    def every(*slots):
        return list(slots)

    def ring(item):
        return every(
            GRID["tl"], GRID["tm"], GRID["tr"],
            GRID["ml"], GRID["mr"],
            GRID["bl"], GRID["bm"], GRID["br"],
        )

    # --- the three recipes the package carries --------------------------------
    # `PPP/PCP/PPP`: planks around the crafting table.
    check(
        "town hall: 8 planks around a crafting table",
        [(PLANKS, 8), (TABLE, 1)],
        [(PLANKS, ring(PLANKS)), (TABLE, every(GRID["mm"]))],
        TOWN_HALL, 1, TABLE,
    )
    # `PPP/PHP/PPP`: planks around an iron hoe.
    check(
        "farmer's hut: 8 planks around an iron hoe",
        [(PLANKS, 8), (HOE, 1)],
        [(PLANKS, ring(PLANKS)), (HOE, every(GRID["mm"]))],
        FARMER_HUT, 1, HOE,
    )
    # `I I`/` S `/` S `: two iron with a gap, the shaft below it. (The first shape, `II `/` S `/
    # ` S `, was exactly vanilla's iron hoe and crafted a hoe instead: the port moved an
    # ingredient, as the `constructionstick` class requires.)
    check(
        "scythe: two iron ingots with a gap over two sticks",
        [(INGOT, 2), (STICK, 2)],
        [(INGOT, every(GRID["tl"], GRID["tr"])), (STICK, every(GRID["mm"], GRID["bm"]))],
        SCYTHE, 1, INGOT,
    )

    # --- controls: a wrong ingredient leaves the result empty ----------------
    empty_control(
        "town hall with sticks where the planks go",
        [(STICK, 8), (TABLE, 1)],
        [(STICK, ring(STICK)), (TABLE, every(GRID["mm"]))],
    )
    empty_control(
        "scythe with a stick where the iron goes",
        [(STICK, 3), (INGOT, 1)],
        [(STICK, every(GRID["tl"], GRID["tr"], GRID["mm"])), (INGOT, every(GRID["bm"]))],
    )

    if failures:
        for line in failures:
            print(f"  FAIL {line}", file=sys.stderr)
        return 1
    print("  three carried recipes craft their declared id and count, the grid empties,")
    print("  each distinctive ingredient is consumed, and two wrong-ingredient controls craft nothing")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
