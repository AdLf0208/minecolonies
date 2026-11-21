#!/usr/bin/env bash
# In-game check for MineColonies slices 1 and 2: a town hall founds a colony, a citizen joins it
# and farms, a lumberjack fells whole trees and replants them, a miner sinks a 1x1 shaft, and the
# colony survives a quit and relaunch.
#
# Every mod command is typed with `ctl chat "/cmd"` because `ctl command` runs the console path
# and bypasses the native `perform_command` hook.
#
# The scratch home goes under `E2E_TMP` when the launcher's env block exports one (the campaign's
# `/tmp` is a 7.5 GB tmpfs that has filled before).
#
# The End, in the void at x=2000 (away from the central island, its dragon egg and the ambient
# item entities of a generated level), on a stone floor laid before anything is placed.
#
# Default run (the launcher-built game carrying the flat SDK package):
#  A  placing the town hall founds a colony: the default "<owner>'s Colony", a 64-block radius
#     (4 chunks), owner set, saved in the block entity;
#  B  `/minecolonies colony rename Riverside` renames it and the next read shows the new name;
#  C  a second town hall 120 blocks away is REFUSED (under the 128-block distance,
#     `ColonyManager.java:313-320`), the block never appears and the chat says why;
#  D  a town hall 120 blocks across and 200 blocks up is refused too: the distance is
#     horizontal (upstream's 3D check would allow it — the defect this port fixes);
#  E  a town hall 200 blocks away is refused because the player already owns a colony;
#  F  the farmer's hut is placed and two mature wheat crops are laid inside the colony;
#  G  `/minecolonies citizen hire Ada` summons the citizen and the roster shows it;
#  H  `/minecolonies job assign Ada farmer` is refused before the hut exists and accepted after
#     it (a job needs its building);
#  I  the job TICKS: the near crop is harvested (age 7 -> 0), the colony counts it, and the
#     citizen walks toward the far one (its position moves east);
#  J  a citizen with NO colony tag, beside its own mature crop far outside the colony, does
#     nothing: the crop stays mature and the citizen does not move (the differential);
#  K  the colony survives `quit` and a relaunch: name, roster, radius and harvested count are
#     the same, and the job ticks again on a fresh crop;
#  L  a LUMBERJACK: a four-log oak tree with a leaves cap; while the hired citizen is still
#     unemployed the tree stays whole (the counterfactual), then `job assign … lumberjack`
#     fells all four logs, leaves the leaves, puts 4 oak_log in the colony's chest and replants
#     a sapling on the stump's soil because the chest held one; a second tree with no sapling in
#     the chest is felled the same way and its soil is left bare;
# L1  the lumberjack's HUT is its building (slice 3): with none in the colony the assignment is
#     REFUSED by name, and the citizen beside the whole tree leaves it whole over 200 ticks;
# L2  the hut is placed: `/job info` reads "lumberjack's hut present" and the assignment is
#     accepted; the tree then falls as in L;
# L3  the hut is BROKEN: the block is air and the same command reads "lumberjack's hut missing",
#     so the marker went with the block;
#  M  a MINER: while the hired citizen is still unemployed an eight-block stone column over
#     bedrock stays whole (the counterfactual), then `job assign … miner` digs it 1x1 down to
#     the bedrock, leaves the bedrock and the four neighbouring floor blocks untouched, and puts
#     8 cobblestone in the colony's chest;
# M1  the MINE is the miner's building: placed inside the colony, `/job info` reads "mine present"
#     before the assignment is accepted;
#  N  the five carried recipes (town hall, farmer's hut, lumberjack's hut, mine, scythe) are
#     crafted in a real crafting table, each asserting its id, its count, an emptied grid and a
#     consumed ingredient, with wrong-ingredient controls that must craft nothing; the scythe is
#     read back from the inventory and an unknown minecolonies id is refused.
#
# CONTROL=1 first launches the **unmodified** Nightly (`CONTROL_GAME`, the launcher's control
# export): every mod id must then be unknown, which is what proves the package supplied them.
#
# SELFTEST=1 inverts M's expected depth (9 blocks above the bedrock instead of 8): a miner that
# stopped one block short must FAIL, proving the assertion can fail. FOUNDING_SELFTEST=1 flips
# C's expectation instead (the second town hall must have been refused), the slice-1 leg.
# HUT_SELFTEST=1 flips L3 (a broken hut must still read "present"), so a port whose marker
# outlived its block passes that leg and fails the normal one: the break assertion can fail.
#
# The game must be a build of `horizon-mod.toml`'s `[native]` pin: a native `.hmod`'s Rust is
# compiled by the launcher against exactly that build, so a game built at another pin is not
# this mod's. `GAME` names it; nothing here builds one.
set -Eeuo pipefail
export LC_ALL=C
[ -z "${MC_DEBUG:-}" ] || set -x

CONTROL=${CONTROL:-0}
SELFTEST=${SELFTEST:-0}
FOUNDING_SELFTEST=${FOUNDING_SELFTEST:-0}
HUT_SELFTEST=${HUT_SELFTEST:-0}

CLONE=${CLONE:-$(cd "$(dirname "$0")/../../.." && pwd)}
PORT=$CLONE/ports/minecolonies
# A control client from this port's own pin when its pair has been published; the
# shared folders hold whichever pin they were built at (`horizon-1d63e0a5` today).
# The ctl protocol is additive, so the old pair still drives a newer game, but the
# game is always built here at `compatibility.json`'s pin.
PIN=${PIN:-$(python3 -c 'import json,sys;print(json.load(open(sys.argv[1]))["horizon_revision"][:8])' \
    "$PORT/compatibility.json" 2>/dev/null || echo 1d63e0a5)}
PIN_BIN=$CLONE/ports/_shared/bin/horizon-$PIN
[ -d "$PIN_BIN" ] || PIN_BIN=$CLONE/ports/_shared/bin/horizon-1d63e0a5
CTL=${CTL:-$PIN_BIN/horizon-ctl}
GAME=${GAME:-$PORT/target/horizon-cargo/fast/horizon-client}
RES=${RES:-${HORIZON_RES:-/home/adlf/Desktop/horizon/.local/horizon-client/26.3}}
SOCK=${SOCK:-/tmp/opencode/minecolonies-e2e.sock}
KEEP=${KEEP:-0}
WORLD=MineColoniesE2E
CONTROL_WORLD=MineColoniesControl
# The arena: a stone floor at y=100 from x=1990 to x=2220, so the refused halls at 2120 and
# 2200 stand on ground and a chunk stays loaded between them.
X0=2000
FLOOR=100
SECOND=$((X0 + 120))
FAR=$((X0 + 200))
CONTROL_X=$((X0 + 100))
# Slice 2: the colony's chest stands east of the hall; the two trees are north of it, their
# trunks two and four blocks from where the second citizen is hired; the miner's column is the
# one under the third citizen's own hire position (the miner digs straight down from its feet).
CHEST_X=$((X0 + 10)); CHEST_Y=$((FLOOR + 1)); CHEST_Z=0
TREE1_X=$((X0 + 4)); TREE1_Z=-1
TREE2_X=$((X0 + 6)); TREE2_Z=-1
TREE_BASE=$((FLOOR + 1))
TREE_TOP=$((FLOOR + 5))
# The miner digs straight down from the third hire's own feet (y = FLOOR + 1), so its shaft is the
# arena's floor block at FLOOR and the seven under it, with bedrock eight blocks below.
SHAFT_X=$((X0 + 4)); SHAFT_Z=0
# Slice 3: each job's building stands inside the colony, away from the trees, the crops and the
# shaft the citizens work.
LHUT_X=$((X0 + 2)); LHUT_Y=$((FLOOR + 1)); LHUT_Z=-4
MINE_X=$((X0 + 6)); MINE_Y=$((FLOOR + 1)); MINE_Z=4
SHAFT_TOP=$FLOOR
SHAFT_BOTTOM=$((FLOOR - 8))
EXPECT_DEPTH=${EXPECT_DEPTH:-8}
if [ "$SELFTEST" = 1 ]; then
    # The counterfactual: a miner one block short must fail the depth assertion.
    EXPECT_DEPTH=$((EXPECT_DEPTH + 1))
fi

command -v jq >/dev/null || { echo "missing jq (every assertion parses a ctl reply)" >&2; exit 1; }
[ -x "$CTL" ] || { echo "missing horizon-ctl: $CTL" >&2; exit 1; }
[ -x "$GAME" ] || { echo "missing horizon-client: $GAME" >&2; exit 1; }
[ -d "$RES/assets" ] || { echo "missing prepared resources: $RES" >&2; exit 1; }

SCRATCH=$(mktemp -d "${E2E_TMP:-/tmp/opencode}"/minecolonies-home.XXXXXX)
export HORIZON_HOME=$SCRATCH/game
export HORIZON_LAUNCHER_HOME=$SCRATCH/launcher
export XDG_DATA_HOME=$SCRATCH/data
export XDG_CONFIG_HOME=$SCRATCH/config
export HORIZON_NO_AUDIO=1
mkdir -p "$(dirname "$SOCK")" "$XDG_CONFIG_HOME"
rm -f "$SOCK"

ctl() { "$CTL" --socket "$SOCK" "$@"; }
say() { ctl chat "/$1" >/dev/null; }
fail() { echo "FAIL: $*" >&2; ctl logs --lines 60 >&2 || true; exit 1; }
on_error() { echo "FAIL: line $1: '$2' exited $?" >&2; ctl logs --lines 40 >&2 || true; }
trap 'on_error "$LINENO" "$BASH_COMMAND"' ERR

cleanup() {
    local status=$?
    ctl quit >/dev/null 2>&1 || true
    sleep 1
    if [ "$KEEP" = 1 ]; then
        echo "kept scratch: $SCRATCH" >&2
    else
        rm -rf "$SCRATCH" 2>/dev/null || true
    fi
    # Last line of every run: `check-e2e-logs.py` reads it as the verdict.
    echo "EXIT:$status"
}
trap cleanup EXIT

launch() { # launch [WORLD] [GAME]
    local world=${1:-$WORLD} game=${2:-$GAME}
    rm -f "$SOCK"
    cd "$SCRATCH"
    # No `--dev-mod`: the launcher-built game carries the package's code, assets and data.
    # `--offscreen`, not `--headless`: phase N opens a crafting table's menu and clicks its slots,
    # and a menu needs a screen (`horizon/docs/horizon-ctl-commands.md`; modularrouters' craft check
    # measured the same requirement).
    local args=(--game "$game" --world "$world" --seed 20261105 --creative --offscreen
                --wait --resources "$RES")
    # The control leg quits a game on this same socket and the relaunch of K happens seconds after
    # a quit: give the old process time to let go, then retry rather than fail on a bound socket.
    local attempt
    for attempt in 1 2 3 4 5; do
        if "$CTL" --socket "$SOCK" launch "${args[@]}" >/dev/null 2>&1; then
            break
        fi
        [ "$attempt" = 5 ] && fail "the game did not launch on $SOCK"
        sleep 3
        rm -f "$SOCK"
    done
    ctl wait-ready 120
    # An offscreen relaunch loads terrain more slowly than `wait-ready` promises, and a chat
    # command typed while the server is still finishing is dropped: wait a little longer here, and
    # `info` (and `line`, which every assertion goes through) re-sends rather than giving up on the
    # first silent read.
    ctl wait 60
}

# The newest chat line containing TEXT that arrived after the last line this script consumed,
# waiting up to ~100 ticks for it: a system line can reach the client's chat a tick or two after
# the action that made it, and an older reply must never be mistaken for the new one (`events`
# carries a sequence number, `messages` does not).
# The sequence is kept in a file: `line` runs in a command substitution, so a variable it set
# would not reach this shell.
CHAT_SEQ_FILE=$SCRATCH/chat.seq
: > "$CHAT_SEQ_FILE"
chat_seq() { cat "$CHAT_SEQ_FILE" 2>/dev/null || echo 0; }
line() { # line TEXT
    local text=$1 reply found last
    for _ in $(seq 1 25); do
        reply=$(ctl events --since "$(chat_seq)")
        found=$(jq -r --arg text "$text" '.events[] | select(.event == "chat") | .text | select(contains($text))' <<<"$reply" | head -1)
        last=$(jq -r '.last // 0' <<<"$reply")
        printf '%s' "$last" > "$CHAT_SEQ_FILE"
        if [ -n "$found" ]; then
            printf '%s\n' "$found"
            return 0
        fi
        ctl wait 4 >/dev/null
    done
    return 1
}

block_name() { ctl block "$1" "$2" "$3" | jq -r '.name // .block // empty'; }
block_state() { ctl block "$1" "$2" "$3" | jq -r '.state // empty'; }
crop_age() { ctl block "$1" "$2" "$3" | jq -r '.state // empty' | sed -n 's/.*age=\([0-9]*\).*/\1/p'; }

# The End's void at X0: a stone floor, clear air above it, no leftover mobs. The player is
# teleported first so the chunks the fills touch are loaded.
arena() {
    ctl command "execute in minecraft:the_end run tp @s $X0.5 $((FLOOR + 1)) 0.5" >/dev/null
    local loaded=false
    for _ in $(seq 1 120); do
        ctl wait 5 >/dev/null
        if [ "$(ctl block "$X0" $FLOOR 0 | jq -r '.loaded // false')" = true ]; then
            loaded=true
            break
        fi
    done
    [ "$loaded" = true ] || fail "the End arena chunk never loaded"
    # `/fill` takes at most 32,768 blocks, so the floor and the air above it go in slices.
    local from=$((X0 - 10)) to slice
    while [ "$from" -le $((FAR + 20)) ]; do
        slice=$((from + 57))
        [ "$slice" -le $((FAR + 20)) ] || slice=$((FAR + 20))
        ctl command "execute in minecraft:the_end run fill $from $FLOOR -14 $slice $FLOOR 14 minecraft:stone" >/dev/null
        ctl command "execute in minecraft:the_end run fill $from $((FLOOR + 1)) -14 $slice $((FLOOR + 4)) 14 minecraft:air" >/dev/null
        from=$((slice + 1))
    done
    ctl wait 20 >/dev/null
    ctl command "execute in minecraft:the_end run kill @e[type=!player]" >/dev/null
    ctl command "execute in minecraft:the_end run tp @s $X0.5 $((FLOOR + 1)) 0.5" >/dev/null
    ctl wait 20 >/dev/null
    [ "$(block_name "$X0" $FLOOR 0)" = "minecraft:stone" ] || fail "the arena floor is not stone"
    [ "$(block_name "$X0" $((FLOOR + 1)) 0)" = "minecraft:air" ] || fail "the arena air above the floor is not air"
}

# Hold `item` in the first hotbar slot and select it (`give` may fill a main-inventory slot that
# `select ITEM` cannot reach).
hold() { # hold ITEM [COUNT]
    local count=${2:-1}
    ctl command "item replace entity @s hotbar.0 with $1 $count" >/dev/null
    ctl wait 4 >/dev/null
    ctl select 1 >/dev/null
    ctl wait 2 >/dev/null
}

# Place `item` at X Y Z from the held stack, standing two blocks west and aiming at the top face
# of the floor under the target. `ctl place` clicks that face, so the game's own placement (and
# the native use hook) runs.
place() { # place X Y Z ITEM [FLOOR_Y]
    local x=$1 y=$2 z=$3 item=$4 under
    under=${5:-$((y - 1))}
    ctl command "execute in minecraft:the_end run tp @s $((x - 2)).5 $y $z.5" >/dev/null
    ctl wait 6 >/dev/null
    hold "$item" 64
    ctl look-at-block "$x" "$under" "$z" up >/dev/null || fail "cannot aim at the floor under $x,$y,$z"
    ctl place "$x" "$y" "$z" --item "$item" >/dev/null || fail "ctl place of $item at $x,$y,$z failed"
    ctl wait 6 >/dev/null
}

# `/minecolonies colony info`: the new reply, in COLONY_LINE (the harvest counter tells it apart
# from the founding notice, which also starts with "Colony").
info() {
    # A chat command typed while the server is still finishing a relaunch is dropped, and the
    # reply then never comes: send it again rather than mistake a slow world for a silent mod.
    local attempt
    for attempt in 1 2 3 4 5 6 7 8; do
        say "minecolonies colony info"
        if COLONY_LINE=$(line 'crops harvested'); then
            echo "  $COLONY_LINE" >&2
            return 0
        fi
        ctl wait 40 >/dev/null 2>&1 || true
    done
    fail "no /minecolonies colony info reply"
}
harvested() { sed -n 's/.*citizens, \([0-9]\{1,\}\) crops harvested.*/\1/p' <<<"$COLONY_LINE"; }
expect_line() { # expect_line PATTERN
    grep -q -- "$1" <<<"$COLONY_LINE" || fail "the colony line lacks '$1': $COLONY_LINE"
}

# A placement that must be refused: the block stays air and the chat explains why.
expect_refused() { # expect_refused X Y Z PATTERN
    local x=$1 y=$2 z=$3 why=$4 name line
    name=$(block_name "$x" "$y" "$z")
    [ "$name" = "minecraft:air" ] || fail "a refused town hall was placed at $x,$y,$z (got '$name')"
    local refusal
    refusal=$(line 'No colony here') || fail "no refusal line after the town hall at $x,$y,$z"
    line=$refusal
    echo "  refusal: $line" >&2
    grep -q -- "$why" <<<"$line" || fail "the refusal at $x,$y,$z does not say '$why': $line"
}

# `ctl entities` reports every mob with `kind: "mob"`; the type id is in `type`, and the
# position in `pos` (AGENTS.md, and `agent.rs::entities_value`).
citizens() { ctl entities 64 minecolonies:citizen | jq '[.entities[] | select(.type == "minecolonies:citizen")] | length'; }
citizen_pos() { ctl entities 64 minecolonies:citizen | jq -r '[.entities[] | select(.type == "minecolonies:citizen")][0].pos[0]' 2>/dev/null || true; }

# The colony's counters, parsed from `/minecolonies colony info` (`info` sets COLONY_LINE).
felled() { sed -n 's/.*, \([0-9]\{1,\}\) logs felled.*/\1/p' <<<"$COLONY_LINE"; }
dug() { sed -n 's/.*, \([0-9]\{1,\}\) blocks mined.*/\1/p' <<<"$COLONY_LINE"; }

# A block's id at a position, without its state properties (`minecraft:oak_leaves[distance=7,…]`
# is `minecraft:oak_leaves`).
block_id() { block_name "$1" "$2" "$3" | sed 's/\[.*//'; }

# A block at a position is exactly ID.
block_is() { [ "$(block_id "$1" "$2" "$3")" = "$4" ]; }

# How many of ITEM the chest at the colony's store holds (`ctl container` lists its slots).
chest_count() { # chest_count ITEM
    ctl container "$CHEST_X" "$CHEST_Y" "$CHEST_Z" \
        | jq -r --arg item "$1" '[.slots[]? | select(. != null and .item == $item) | .count] | add // 0'
}

# Wait up to MAX ticks in STEP-tick slices for a predicate to hold; then evaluate it once more so
# the caller's `||` reports the real state. A bounded wait, never a poll loop around a build.
await_ticks() { # await_ticks MAX STEP PREDICATE...
    local max=$1 step=$2 waited=0
    shift 2
    while [ "$waited" -lt "$max" ]; do
        if "$@" >/dev/null 2>&1; then return 0; fi
        ctl wait "$step" >/dev/null
        waited=$((waited + step))
    done
    "$@"
}

# The control: the same game with no `--dev-mod`, so every mod id must be unknown. A scenario
# that cannot fail is not evidence, and this leg is what proves the ids came from the package.
control_leg() {
    echo "== control: the same Nightly without the package =="
    [ -n "${CONTROL_GAME:-}" ] || fail "CONTROL=1 needs CONTROL_GAME (the unmodified Nightly: launcher-game.sh env <mod>)"
    launch "$CONTROL_WORLD" "$CONTROL_GAME"
    arena
    ctl command "item replace entity @s hotbar.0 with minecolonies:blockhuttownhall 1" >/dev/null || true
    ctl wait 4 >/dev/null
    ctl inventory | jq -e '[.hotbar[], .main[] | select(.item == "minecolonies:blockhuttownhall")] | length == 0' >/dev/null \
        || fail "the control game has the town hall item, so the content did not come from the package"
    ctl command "execute in minecraft:the_end run setblock $((X0 + 6)) $((FLOOR + 1)) 0 minecolonies:blockhuttownhall" >/dev/null || true
    ctl wait 4 >/dev/null
    local name
    name=$(block_name "$((X0 + 6))" $((FLOOR + 1)) 0)
    [ "$name" = "minecraft:air" ] || fail "the control game holds $name, so the block id was known"
    echo "  without the package: no town hall item, the block stayed $name"
    ctl quit >/dev/null 2>&1 || true
    sleep 3
}

# The scenario's core, used by every leg.
founding_and_first_room() {
    echo "== A: the town hall founds a colony =="
    place "$X0" $((FLOOR + 1)) 0 minecolonies:blockhuttownhall
    [ "$(block_name "$X0" $((FLOOR + 1)) 0)" = "minecolonies:blockhuttownhall" ] || fail "the town hall did not place"
    local founded
    founded=$(line 'founded:') || fail "no founding line after placing the town hall"
    echo "  $founded" >&2
    grep -q "blocks around the town hall, owner " <<<"$founded" || fail "the founding line lacks the radius or the owner: $founded"
    info
    expect_line " 0 citizens"
    expect_line " 64 blocks (4 chunks)"
    expect_line " 0 crops harvested"

    echo "== B: the colony is renamed =="
    say "minecolonies colony rename Riverside"
    ctl wait 12 >/dev/null
    info
    expect_line "Colony Riverside (id "
    expect_line " 64 blocks (4 chunks)"

    echo "== C: a second town hall 120 blocks away is refused =="
    place_attempt "$SECOND" $((FLOOR + 1)) 0
    if [ "$FOUNDING_SELFTEST" = 1 ]; then
        echo "(selftest: the second town hall must have been placed)"
        [ "$(block_name "$SECOND" $((FLOOR + 1)) 0)" = "minecolonies:blockhuttownhall" ] \
            || fail "selftest: the second town hall was refused, as it should be"
    else
        expect_refused "$SECOND" $((FLOOR + 1)) 0 "too close to Riverside"
    fi

    echo "== D: 120 blocks across and 120 up is refused too (the distance is horizontal) =="
    # 120 across and 120 up is 170 blocks in 3D (upstream's `BlockPosUtil.getDistance`), so
    # upstream allows this site; the colony is a 2D claim and the port refuses it.
    raised_attempt "$SECOND" $((FLOOR + 120)) 0
    if [ "$FOUNDING_SELFTEST" = 1 ]; then
        echo "(selftest: the raised town hall must have been placed)"
        [ "$(block_name "$SECOND" $((FLOOR + 201)) 0)" = "minecolonies:blockhuttownhall" ] \
            || fail "selftest: the raised town hall was refused, as it should be"
    else
        expect_refused "$SECOND" $((FLOOR + 201)) 0 "too close to Riverside"
    fi

    echo "== E: a town hall 200 blocks away is refused: the player already owns a colony =="
    place_attempt "$FAR" $((FLOOR + 1)) 0
    if [ "$FOUNDING_SELFTEST" = 1 ]; then
        echo "(selftest: the far town hall must have been placed)"
        [ "$(block_name "$FAR" $((FLOOR + 1)) 0)" = "minecolonies:blockhuttownhall" ] \
            || fail "selftest: the far town hall was refused, as it should be"
    else
        expect_refused "$FAR" $((FLOOR + 1)) 0 "you already own Riverside"
    fi
}

# A town hall attempt well above the colony: a block under the site to click and one under the
# player to stand on, so the placement is the game's own (a face is visible) and the native use
# hook runs.
raised_attempt() { # raised_attempt X Y Z
    local x=$1 y=$2 z=$3
    ctl command "execute in minecraft:the_end run setblock $((x - 2)) $((y - 1)) $z minecraft:stone" >/dev/null
    ctl command "execute in minecraft:the_end run setblock $x $((y - 1)) $z minecraft:stone" >/dev/null
    ctl wait 6 >/dev/null
    [ "$(block_name $((x - 2)) $((y - 1)) $z)" = "minecraft:stone" ] || fail "the raised platform did not stand at $x,$y,$z"
    [ "$(block_name "$x" $((y - 1)) "$z")" = "minecraft:stone" ] || fail "the raised support did not stand under $x,$y,$z"
    place_attempt "$x" "$y" "$z" $((y - 1))
}

# A placement that is *expected* to be refused: after it, the caller asserts.
place_attempt() { # place_attempt X Y Z [AIM_Y [FACE]]
    local x=$1 y=$2 z=$3 aim face
    aim=${4:-$((y - 1))}
    face=${5:-up}
    # Teleport first: it loads the chunks, which a far `/fill` slice may have missed.
    ctl command "execute in minecraft:the_end run tp @s $((x - 2)).5 $y $z.5" >/dev/null
    ctl wait 12 >/dev/null
    # Then give the player a stand and the target a support, and stand on it.
    ctl command "execute in minecraft:the_end run setblock $((x - 2)) $((y - 1)) $z minecraft:stone" >/dev/null
    ctl command "execute in minecraft:the_end run setblock $x $((y - 1)) $z minecraft:stone" >/dev/null
    ctl wait 6 >/dev/null
    ctl command "execute in minecraft:the_end run tp @s $((x - 2)).5 $y $z.5" >/dev/null
    ctl wait 6 >/dev/null
    hold minecolonies:blockhuttownhall 64
    if [ "$aim" = "$y" ]; then
        ctl look-at-block $((x - 1)) "$aim" "$z" "$face" >/dev/null 2>&1 || true
    else
        ctl look-at-block "$x" "$aim" "$z" "$face" >/dev/null 2>&1 || true
    fi
    ctl place "$x" "$y" "$z" --item minecolonies:blockhuttownhall >/dev/null 2>&1 || true
    ctl wait 6 >/dev/null
}

farm_setup() {
    echo "== F: a farmer's hut and two mature crops inside the colony =="
    place $((X0 + 3)) $((FLOOR + 1)) 3 minecolonies:blockhutfarmer
    [ "$(block_name $((X0 + 3)) $((FLOOR + 1)) 3)" = "minecolonies:blockhutfarmer" ] \
        || fail "the farmer's hut did not place"
    ctl command "execute in minecraft:the_end run setblock $((X0 + 8)) $((FLOOR + 1)) 0 minecraft:water" >/dev/null
    ctl command "execute in minecraft:the_end run fill $((X0 + 4)) $((FLOOR + 1)) -2 $((X0 + 7)) $((FLOOR + 1)) 2 minecraft:farmland[moisture=7]" >/dev/null
    ctl wait 6 >/dev/null
    ctl command "execute in minecraft:the_end run setblock $((X0 + 4)) $((FLOOR + 2)) 0 minecraft:wheat[age=7]" >/dev/null
    ctl command "execute in minecraft:the_end run setblock $((X0 + 7)) $((FLOOR + 2)) 1 minecraft:wheat[age=7]" >/dev/null
    ctl wait 6 >/dev/null
    [ "$(crop_age $((X0 + 4)) $((FLOOR + 2)) 0)" = 7 ] || fail "the near crop is not mature"
    [ "$(crop_age $((X0 + 7)) $((FLOOR + 2)) 1)" = 7 ] || fail "the far crop is not mature"
}

citizen_and_job() {
    echo "== G: a citizen joins the colony =="
    say "minecolonies citizen hire Ada"
    ctl wait 20 >/dev/null
    local hiring
    hiring=$(line 'joined') || fail "no 'joined' line after hiring"
    echo "  $hiring" >&2
    [ "$(citizens)" = 1 ] || fail "expected one minecolonies:citizen, found $(citizens)"
    info
    expect_line " 1 citizens"
    local listing
    say "minecolonies citizen list"
    listing=$(line 'Ada — Unemployed') || fail "no citizen list line showing Ada unemployed"
    echo "  $listing" >&2

    echo "== H: the job needs its building, then is assigned =="
    # An unemployed citizen has no building; the farmer's hut placed in F is what the assigned
    # job's rule then reads, so the same command changes from "no building" to "present".
    say "minecolonies job info Ada"
    local job_line
    job_line=$(line "Ada: job") || fail "no /job info line"
    echo "  $job_line" >&2
    grep -q "Ada: job Unemployed" <<<"$job_line" || fail "Ada is not unemployed before the job: $job_line"
    grep -q ", no building\." <<<"$job_line" || fail "an unemployed citizen claims a building: $job_line"
    say "minecolonies job assign Ada farmer"
    local assigned
    assigned=$(line 'is now the colony') || fail "no assignment line"
    echo "  $assigned" >&2
    grep -q "farmer" <<<"$assigned" || fail "the assignment line is not a farmer: $assigned"
    say "minecolonies job info Ada"
    job_line=$(line "Ada: job") || fail "no /job info line after the assignment"
    echo "  $job_line" >&2
    grep -q "farmer's hut present" <<<"$job_line" || fail "the job line does not see the hut: $job_line"
    say "minecolonies citizen list"
    listing=$(line 'Ada — Farmer') || fail "the roster does not show Ada as the farmer"
    echo "  $listing" >&2
}

job_ticks() {
    echo "== I: the farmer works: the near crop is harvested and it walks to the far one =="
    info
    local before_harvest before_pos
    before_harvest=$(harvested)
    before_pos=$(citizen_pos)
    # The farmer may already have worked between the assignment and this read, so what is
    # asserted is that the near crop is harvested and counted, not that the count grew here;
    # the growth is asserted after the relaunch on a crop placed for the purpose.
    ctl wait 160 >/dev/null
    info
    local after_harvest near_age
    ctl chat "/data get entity @e[type=minecolonies:citizen,limit=1]" >/dev/null; ctl wait 3 >/dev/null
    ctl chat "/data get block $X0 $((FLOOR + 1)) 0" >/dev/null; ctl wait 3 >/dev/null
    after_harvest=$(harvested)
    near_age=$(crop_age $((X0 + 4)) $((FLOOR + 2)) 0)
    [ "$near_age" = 0 ] || fail "the near crop is still age $near_age, not replanted at 0"
    [ "${after_harvest:-0}" -ge 1 ] || fail "the colony counted no harvest (${before_harvest:-0} -> ${after_harvest:-0})"
    echo "  harvested ${before_harvest:-0} -> $after_harvest, near crop age 7 -> $near_age" >&2
    ctl wait 400 >/dev/null
    local after_pos far_age far_harvest
    after_pos=$(citizen_pos)
    far_age=$(crop_age $((X0 + 7)) $((FLOOR + 2)) 1)
    info
    far_harvest=$(harvested)
    # The far crop is harvested (age 0) and wheat grows again on its own, so the assertion is
    # that it is no longer mature *and* that the colony counted a second harvest. A farmer that
    # never reached it leaves both at 7 and 1.
    [ "${far_age:-7}" -lt 7 ] || fail "the far crop is still age $far_age: the farmer never reached it"
    [ "${far_harvest:-0}" -ge 2 ] \
        || fail "the colony counted ${far_harvest:-0} harvest(s), so the far crop was not harvested"
    echo "  citizen $before_pos -> $after_pos, far crop age 7 -> $far_age, harvested $after_harvest -> $far_harvest" >&2
    [ "$before_pos" != "$after_pos" ] || fail "the citizen never moved: $before_pos"
    echo "  the job ticks in the world: a crop changed and the citizen walked"
}

no_colony_control() {
    echo "== J: a citizen with no colony does nothing =="
    # Far outside the colony (its radius is 64 blocks), with its own mature crop beside it. The
    # player goes there so `ctl entities` sees the citizen (its radius is around the player).
    ctl command "execute in minecraft:the_end run tp @s $((CONTROL_X - 4)).5 $((FLOOR + 1)) 0.5" >/dev/null
    ctl wait 20 >/dev/null
    ctl command "execute in minecraft:the_end run setblock $((CONTROL_X + 4)) $((FLOOR + 1)) 0 minecraft:water" >/dev/null
    ctl command "execute in minecraft:the_end run setblock $((CONTROL_X + 2)) $((FLOOR + 1)) 0 minecraft:farmland[moisture=7]" >/dev/null
    ctl wait 4 >/dev/null
    ctl command "execute in minecraft:the_end run setblock $((CONTROL_X + 2)) $((FLOOR + 2)) 0 minecraft:wheat[age=7]" >/dev/null
    # `NoAI` keeps the vanilla villager from wandering: what is measured is the port's job, not
    # the template's stroll.
    ctl command "execute in minecraft:the_end run summon minecolonies:citizen $CONTROL_X.5 $((FLOOR + 1)) 0.5 {NoAI:1b,PersistenceRequired:1b}" >/dev/null
    ctl wait 10 >/dev/null
    local kind position age
    kind=$(ctl entities 32 minecolonies:citizen | jq -r '[.entities[] | select(.pos[0] > '"$CONTROL_X"')][0].type // empty')
    [ "$kind" = "minecolonies:citizen" ] || fail "the control citizen did not spawn (type '$kind')"
    position=$(ctl entities 32 minecolonies:citizen | jq -r '[.entities[] | select(.pos[0] > '"$CONTROL_X"')][0].pos[0]')
    ctl wait 200 >/dev/null
    age=$(crop_age $((CONTROL_X + 2)) $((FLOOR + 2)) 0)
    [ "$age" = 7 ] || fail "the citizen without a colony harvested: the crop is age $age"
    local moved
    moved=$(ctl entities 32 minecolonies:citizen | jq -r '[.entities[] | select(.pos[0] > '"$CONTROL_X"')][0].pos[0]')
    [ "$moved" = "$position" ] || fail "the citizen without a colony moved ($position -> $moved)"
    echo "  the citizen with no colony left the crop at age $age and stayed at x=$moved" >&2
}

persistence() {
    echo "== K: the colony survives quit and relaunch =="
    info
    local name_line before
    name_line=$COLONY_LINE
    before=$(harvested)
    ctl quit >/dev/null
    sleep 3
    launch "$WORLD"
    info
    expect_line " 1 citizens"
    expect_line "Colony Riverside (id "
    [ "$(harvested)" = "$before" ] || fail "the harvested count changed across the relaunch: $before -> $(harvested)"
    say "minecolonies citizen list"
    local listing
    listing=$(line 'Ada — Farmer') || fail "the roster lost Ada's job across the relaunch"
    echo "  $listing" >&2
    [ "$(citizens)" = 1 ] || fail "the citizen entity did not survive the relaunch ($(citizens))"
    # A fresh crop, and the job works again.
    ctl command "execute in minecraft:the_end run setblock $((X0 + 5)) $((FLOOR + 2)) -1 minecraft:wheat[age=7]" >/dev/null
    ctl wait 6 >/dev/null
    ctl wait 200 >/dev/null
    info
    [ "$(harvested)" -gt "$before" ] || fail "the job did not resume after the relaunch ($before -> $(harvested))"
    echo "  after the relaunch: $before -> $(harvested) harvested, roster intact" >&2
}

# A tree from the mod's own world data: a four-high oak trunk standing on dirt with a leaves cap
# (the brief's own shape). `tree_stands X Z` is the predicate the counterfactual waits on.
plant_tree() { # plant_tree X Z
    local x=$1 z=$2 y
    ctl command "execute in minecraft:the_end run setblock $x $TREE_BASE $z minecraft:dirt" >/dev/null
    for y in $((TREE_BASE + 1)) $((TREE_BASE + 2)) $((TREE_BASE + 3)) $TREE_TOP; do
        ctl command "execute in minecraft:the_end run setblock $x $y $z minecraft:oak_log" >/dev/null
    done
    # `persistent=true`: a leaf that decays on its own would make "the leaves are still there"
    # unprovable; a persistent cap only vanishes if something breaks it.
    ctl command "execute in minecraft:the_end run setblock $x $((TREE_TOP + 1)) $z minecraft:oak_leaves[persistent=true]" >/dev/null
    ctl wait 6 >/dev/null
    local log
    for log in $((TREE_BASE + 1)) $((TREE_BASE + 2)) $((TREE_BASE + 3)) $TREE_TOP; do
        block_is "$x" "$log" "$z" minecraft:oak_log || fail "the tree at $x,$z has no log at y=$log"
    done
    block_is "$x" $((TREE_TOP + 1)) "$z" minecraft:oak_leaves || fail "the tree at $x,$z has no leaves cap"
}

# The whole tree still stands (the unemployed counterfactual, and the tree after it was felled).
tree_stands() { # tree_stands X Z
    local x=$1 z=$2 y
    for y in $((TREE_BASE + 1)) $((TREE_BASE + 2)) $((TREE_BASE + 3)) $TREE_TOP; do
        block_is "$x" "$y" "$z" minecraft:oak_log || return 1
    done
    block_is "$x" $((TREE_TOP + 1)) "$z" minecraft:oak_leaves
}

tree_gone() { # tree_gone X Z — every log cell air, the leaves cap standing
    local x=$1 z=$2 y
    for y in $((TREE_BASE + 1)) $((TREE_BASE + 2)) $((TREE_BASE + 3)) $TREE_TOP; do
        block_is "$x" "$y" "$z" minecraft:air || return 1
    done
    block_is "$x" $((TREE_TOP + 1)) "$z" minecraft:oak_leaves
}

# The tree dropped: the logs above the stump are air, the stump's own cell is air or the sapling
# replanted in it, and the leaves cap still stands (a lumberjack is not a leaf-breaker).
tree_felled() { # tree_felled X Z
    local x=$1 z=$2 y stump
    for y in $((TREE_BASE + 2)) $((TREE_BASE + 3)) $TREE_TOP; do
        block_is "$x" "$y" "$z" minecraft:air || return 1
    done
    stump=$(block_id "$x" $((TREE_BASE + 1)) "$z")
    case "$stump" in
        minecraft:air | minecraft:oak_sapling) ;;
        *) return 1 ;;
    esac
    block_is "$x" $((TREE_TOP + 1)) "$z" minecraft:oak_leaves
}

# A citizen hired into the colony; its position is the hall's spawn pattern, so the phase's tree
# and column are placed where its own hire position puts it. The roster count is the colony's own
# saved record (`colony info`); the live mobs are counted by the caller when the player stands
# beside the hall, because `ctl entities`' radius is measured from the player.
hire() { # hire NAME EXPECTED_ROSTER
    say "minecolonies citizen hire $1"
    ctl wait 20 >/dev/null
    local joined
    joined=$(line 'joined') || fail "no 'joined' line after hiring $1"
    echo "  $joined" >&2
    info
    expect_line " $2 citizens"
}

# Stand the player by the hall so the client tracks its citizens within `ctl entities`' radius.
stand_by_the_hall() {
    ctl command "execute in minecraft:the_end run tp @s $((X0 + 2)).5 $((FLOOR + 1)) -3.5" >/dev/null
    ctl wait 20 >/dev/null
}

assign_job() { # assign_job NAME JOB NEEDLE
    say "minecolonies job assign $1 $2"
    ctl wait 12 >/dev/null
    local line
    line=$(line "$3") || fail "no assignment line for $1 as $2"
    echo "  $line" >&2
    grep -q "$2" <<<"$line" || fail "the assignment line is not a $2: $line"
}

lumberjack_job() {
    echo "== L: a lumberjack fells whole trees, leaves the leaves, and replants from the chest =="
    stand_by_the_hall
    ctl command "execute in minecraft:the_end run setblock $CHEST_X $CHEST_Y $CHEST_Z minecraft:chest" >/dev/null
    ctl wait 6 >/dev/null
    block_is "$CHEST_X" "$CHEST_Y" "$CHEST_Z" minecraft:chest || fail "the colony's chest did not place"
    # The first tree has a sapling waiting in the colony's chest; the second has none.
    ctl container "$CHEST_X" "$CHEST_Y" "$CHEST_Z" minecraft:oak_sapling 1 >/dev/null
    ctl wait 4 >/dev/null
    [ "$(chest_count minecraft:oak_sapling)" = 1 ] || fail "the chest did not take the sapling"
    plant_tree "$TREE1_X" "$TREE1_Z"
    info
    local felled_before dug_before
    felled_before=$(felled); dug_before=$(dug)
    [ -n "$felled_before" ] || fail "the colony line has no felled counter: $COLONY_LINE"

    # Bo is hired but NOT employed: the tree must stay whole (the counterfactual).
    hire Bo 2
    [ "$(citizens)" = 2 ] || fail "expected 2 live citizens after hiring Bo, found $(citizens)"
    ctl wait 200 >/dev/null
    tree_stands "$TREE1_X" "$TREE1_Z" || fail "an unemployed citizen felled the tree at $TREE1_X,$TREE1_Z"
    echo "  the unemployed citizen left the tree whole (4 logs and the leaves stand)" >&2

    echo "== L1: the lumberjack's hut is its building: without it the assignment is refused =="
    say "minecolonies job assign Bo lumberjack"
    ctl wait 12 >/dev/null
    local without
    without=$(line "needs lumberjack's hut in the colony") \
        || fail "Bo was made a lumberjack with no lumberjack's hut in the colony"
    echo "  without its hut: $without" >&2
    # The counterfactual the case needs: a citizen beside a whole tree with the hut absent leaves
    # it whole, so the fell after the hut is placed is the hut's doing and not the assignment's.
    ctl wait 200 >/dev/null
    tree_stands "$TREE1_X" "$TREE1_Z" \
        || fail "a citizen beside a whole tree felled it with no lumberjack's hut in the colony"

    echo "== L2: the hut is placed, job info reads it, and the tree falls =="
    place "$LHUT_X" "$LHUT_Y" "$LHUT_Z" minecolonies:blockhutlumberjack
    block_is "$LHUT_X" "$LHUT_Y" "$LHUT_Z" minecolonies:blockhutlumberjack \
        || fail "the lumberjack's hut did not place"
    # The assignment is accepted now that the hut stands; `job info` then reports *Bo's own*
    # job's building — the same command that read "no building" while Bo was unemployed.
    assign_job Bo lumberjack lumberjack
    local job_line
    say "minecolonies job info Bo"
    job_line=$(line "Bo: job") || fail "no /job info line for Bo"
    echo "  $job_line" >&2
    grep -q "lumberjack's hut present" <<<"$job_line" \
        || fail "job info does not see the lumberjack's hut: $job_line"
    # Four logs, one per work step, and the walk that may precede them.
    if ! await_ticks 600 40 tree_felled "$TREE1_X" "$TREE1_Z"; then
        info
        fail "the lumberjack did not fell the tree at $TREE1_X,$TREE1_Z"
    fi
    local y
    for y in $((TREE_BASE + 2)) $((TREE_BASE + 3)) $TREE_TOP; do
        [ "$(block_id "$TREE1_X" "$y" "$TREE1_Z")" = "minecraft:air" ] \
            || fail "log $TREE1_X,$y,$TREE1_Z is not air after the fell"
    done
    block_is "$TREE1_X" $((TREE_TOP + 1)) "$TREE1_Z" minecraft:oak_leaves \
        || fail "the lumberjack broke the leaves cap at $TREE1_X,$((TREE_TOP + 1)),$TREE1_Z"
    echo "  after the fell: soil at $TREE1_X,$TREE_BASE,$TREE1_Z is '$(block_name "$TREE1_X" "$TREE_BASE" "$TREE1_Z")', stump is '$(block_name "$TREE1_X" $((TREE_BASE + 1)) "$TREE1_Z")', chest $(ctl container "$CHEST_X" "$CHEST_Y" "$CHEST_Z" | jq -c '[.slots[]? | select(. != null)]')" >&2
    block_is "$TREE1_X" "$TREE_BASE" "$TREE1_Z" minecraft:dirt \
        || fail "the stump's soil is '$(block_name "$TREE1_X" "$TREE_BASE" "$TREE1_Z")', not dirt"
    block_is "$TREE1_X" $((TREE_BASE + 1)) "$TREE1_Z" minecraft:oak_sapling \
        || fail "no sapling on the stump's soil although the chest held one: '$(block_name "$TREE1_X" $((TREE_BASE + 1)) "$TREE1_Z")'"
    [ "$(chest_count minecraft:oak_log)" = 4 ] \
        || fail "the colony's chest holds $(chest_count minecraft:oak_log) oak_log, not 4"
    [ "$(chest_count minecraft:oak_sapling)" = 0 ] || fail "the replanted sapling was not taken from the chest"
    info
    [ "$(felled)" = "$((felled_before + 4))" ] \
        || fail "the colony counted $(felled) logs felled, not $((felled_before + 4))"
    echo "  4 logs air, leaves intact, 4 oak_log in the chest, a sapling on the stump's soil" >&2

    # The second tree, with no sapling in the chest: felled the same way, its soil left bare.
    plant_tree "$TREE2_X" "$TREE2_Z"
    if ! await_ticks 600 40 tree_gone "$TREE2_X" "$TREE2_Z"; then
        info
        fail "the lumberjack did not fell the second tree at $TREE2_X,$TREE2_Z"
    fi
    block_is "$TREE2_X" $((TREE_BASE + 1)) "$TREE2_Z" minecraft:air \
        || fail "a sapling appeared without one in the chest"
    [ "$(chest_count minecraft:oak_log)" = 8 ] \
        || fail "the colony's chest holds $(chest_count minecraft:oak_log) oak_log after two trees, not 8"
    info
    [ "$(felled)" = "$((felled_before + 8))" ] || fail "the colony counted $(felled) logs felled, not $((felled_before + 8))"
    [ "$(dug)" = "$dug_before" ] || fail "the lumberjack changed the dug counter"
    echo "  the second tree fell too, its soil bare (no sapling in the chest), 8 logs in all" >&2

    echo "== L3: breaking the hut takes the building with it =="
    # A **survival** break with an axe: a creative break drops nothing in vanilla, so the hut's own
    # loot table would never run. The block is in `minecraft:mineable/axe`, which is what makes the
    # break a couple of seconds; the marker's rule is the same moment in either mode.
    ctl command "gamemode survival" >/dev/null
    ctl command "item replace entity @s hotbar.0 with minecraft:iron_axe 1" >/dev/null
    ctl wait 4 >/dev/null
    ctl select 1 >/dev/null
    ctl wait 2 >/dev/null
    local broken
    broken=$(ctl break "$LHUT_X" "$LHUT_Y" "$LHUT_Z" --timeout 40) \
        || fail "the lumberjack's hut could not be broken"
    ctl command "gamemode creative" >/dev/null
    ctl wait 10 >/dev/null
    block_is "$LHUT_X" "$LHUT_Y" "$LHUT_Z" minecraft:air \
        || fail "the lumberjack's hut still stands after the break"
    # The hut's own loot table: the broken block drops the item a player can place again.
    echo "  the break dropped $(jq -c '.drops' <<<"$broken")" >&2
    jq -e '.drops["minecolonies:blockhutlumberjack"] == 1' <<<"$broken" >/dev/null \
        || fail "the broken hut did not drop its own item (loot table): $(jq -c '.drops' <<<"$broken")"
    say "minecolonies job info Bo"
    job_line=$(line "Bo: job") || fail "no /job info line for Bo after the break"
    echo "  after the break: $job_line" >&2
    if [ "$HUT_SELFTEST" = 1 ]; then
        echo "(selftest: the broken hut must still read present)"
        grep -q "lumberjack's hut present" <<<"$job_line" \
            || fail "selftest: the broken hut reads missing, so the marker went with the block"
    else
        grep -q "lumberjack's hut missing" <<<"$job_line" \
            || fail "job info still sees a broken hut: $job_line"
        echo "  the broken hut reads missing, so the marker went with the block" >&2
    fi
}

# The miner's column: eight stone over bedrock, under the third citizen's own hire position.
miner_column() {
    ctl command "execute in minecraft:the_end run setblock $SHAFT_X $SHAFT_BOTTOM $SHAFT_Z minecraft:bedrock" >/dev/null
    ctl command "execute in minecraft:the_end run fill $SHAFT_X $((SHAFT_BOTTOM + 1)) $SHAFT_Z $SHAFT_X $SHAFT_TOP $SHAFT_Z minecraft:stone" >/dev/null
    ctl wait 6 >/dev/null
    block_is "$SHAFT_X" "$SHAFT_BOTTOM" "$SHAFT_Z" minecraft:bedrock || fail "the shaft's bedrock did not place"
    block_is "$SHAFT_X" "$SHAFT_TOP" "$SHAFT_Z" minecraft:stone || fail "the shaft's top block did not place"
}

column_whole() { # column_whole DEPTH — the DEPTH blocks above the bedrock are stone
    local depth=$1 i
    for i in $(seq 0 $((depth - 1))); do
        block_is "$SHAFT_X" $((SHAFT_TOP - i)) "$SHAFT_Z" minecraft:stone || return 1
    done
}

column_air() { # column_air DEPTH — the DEPTH blocks above the bedrock read air
    local depth=$1 i
    for i in $(seq 0 $((depth - 1))); do
        block_is "$SHAFT_X" $((SHAFT_TOP - i)) "$SHAFT_Z" minecraft:air || return 1
    done
}

miner_job() {
    echo "== M: a miner sinks a 1x1 shaft to the bedrock and collects every drop =="
    miner_column
    # The third hire stands over the column (`spawn_point` puts it at the hall's x+4.5).
    stand_by_the_hall
    hire Cass 3
    [ "$(citizens)" = 3 ] || fail "expected 3 live citizens after hiring Cass, found $(citizens)"
    ctl wait 200 >/dev/null
    column_whole 8 || fail "an unemployed citizen dug the column at $SHAFT_X,$SHAFT_Z"
    echo "  the unemployed citizen left the 8-block column whole" >&2

    echo "== M1: the mine is the miner's building =="
    place "$MINE_X" "$MINE_Y" "$MINE_Z" minecolonies:blockhutminer
    block_is "$MINE_X" "$MINE_Y" "$MINE_Z" minecolonies:blockhutminer || fail "the mine did not place"
    assign_job Cass miner miner
    local job_line
    say "minecolonies job info Cass"
    job_line=$(line "Cass: job") || fail "no /job info line for Cass"
    echo "  $job_line" >&2
    grep -q "mine present" <<<"$job_line" || fail "job info does not see the mine: $job_line"
    local depth=${EXPECT_DEPTH:-8}
    if ! await_ticks 900 40 column_air "$depth"; then
        info
        fail "the miner did not dig $depth blocks above the bedrock at $SHAFT_X,$SHAFT_Z"
    fi
    column_air "$depth" || fail "the shaft is not $depth blocks of air"
    block_is "$SHAFT_X" "$SHAFT_BOTTOM" "$SHAFT_Z" minecraft:bedrock || fail "the miner broke the bedrock"
    # 1x1: the four blocks beside the shaft at its top are still the arena's stone floor.
    block_is $((SHAFT_X - 1)) "$SHAFT_TOP" "$SHAFT_Z" minecraft:stone || fail "the shaft is not 1x1 (west)"
    block_is $((SHAFT_X + 1)) "$SHAFT_TOP" "$SHAFT_Z" minecraft:stone || fail "the shaft is not 1x1 (east)"
    block_is "$SHAFT_X" "$SHAFT_TOP" $((SHAFT_Z - 1)) minecraft:stone || fail "the shaft is not 1x1 (north)"
    block_is "$SHAFT_X" "$SHAFT_TOP" $((SHAFT_Z + 1)) minecraft:stone || fail "the shaft is not 1x1 (south)"
    [ "$(chest_count minecraft:cobblestone)" = 8 ] \
        || fail "the colony's chest holds $(chest_count minecraft:cobblestone) cobblestone, not 8"
    info
    [ "$(dug)" = 8 ] || fail "the colony counted $(dug) blocks mined, not 8"
    echo "  the shaft is 1x1 and $depth deep, the bedrock untouched, 8 cobblestone in the chest" >&2
}

# Every carried recipe, crafted the way it declares; every carried item read back; an unknown id
# refused. The old `data/` was never exercised: a recipe that is never crafted is dead data.
craft_and_items() {
    echo "== N: the carried recipes craft, and the carried items are real ids =="
    python3 "$PORT/tools/craft_check.py" "$SOCK" "$CTL" $((X0 + 14)) $((FLOOR + 1)) 0 \
        || fail "a carried recipe did not craft its declared id and count"
    ctl command "clear @s" >/dev/null
    ctl wait 4 >/dev/null
    ctl command "give @s minecolonies:scythe 1" >/dev/null
    ctl wait 6 >/dev/null
    local held
    held=$(ctl inventory | jq '[.hotbar[], .main[] | select(.item == "minecolonies:scythe")] | length')
    [ "$held" = 1 ] || fail "the scythe is not an item the game knows ($held in the inventory)"
    ctl command "give @s minecolonies:no_such_item 1" >/dev/null
    local refused
    refused=$(line 'Unknown item') || fail "an unknown minecolonies id was not refused"
    echo "  the scythe is real, an unknown id is refused: $refused" >&2
    echo "  every carried recipe crafted its id and count in a real table" >&2
}

echo "== launch (The End arena) =="
if [ "$CONTROL" = 1 ]; then control_leg; fi
launch
arena
founding_and_first_room
farm_setup
citizen_and_job
job_ticks
no_colony_control
persistence
lumberjack_job
miner_job
craft_and_items

echo "PASS: the town hall founded a persisted colony (name, owner, 64-block radius), a second hall was refused when too close (including 120 blocks across and 120 up) and when the owner had one already, a citizen joined, took the farmer's job, harvested a mature crop and walked to the next, a citizen with no colony did nothing, the colony and its job survived quit and relaunch, a lumberjack felled two whole four-log trees (leaves untouched, 4 logs in the colony's chest each, a sapling replanted only when the chest held one) while an unemployed citizen left its tree whole, and a miner sank a 1x1 8-block shaft to the bedrock (bedrock and its neighbours untouched, 8 cobblestone in the chest) while an unemployed citizen left its column whole, both job buildings are the jobs' own (the lumberjack's hut refused, then seen, then missing once broken; the mine seen before the miner works), and all five carried recipes crafted their id and count in a real table with wrong-ingredient controls that crafted nothing"
