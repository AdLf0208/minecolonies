#!/usr/bin/env python3
"""Prove that no asset byte came from the upstream jar.

The upstream art is All Rights Reserved, so this port carries **none** of it: every texture,
model, blockstate, item definition and lang file under `assets/` is produced by
`tools/gen_assets.py` / `tools/assemble_content.py` from this port's own palette. This script
measures that claim instead of asserting it: it hashes every entry of the jar the port was read
from and every file of the port's `assets/` tree, and reports how many bytes of that tree match a
jar entry.

    python3 tools/check_provenance.py [JAR]

Exit 0 when the jar-derived byte count is zero, 1 otherwise.
"""

from __future__ import annotations

import hashlib
import sys
import zipfile
from pathlib import Path

PORT = Path(__file__).resolve().parent.parent
DEFAULT_JAR = PORT.parent / "_shared/licence-cache/minecolonies-1.1.1411-1.21.1-snapshot.jar"


def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def main() -> int:
    jar = Path(sys.argv[1]) if len(sys.argv) > 1 else DEFAULT_JAR
    if not jar.is_file():
        print(f"missing jar: {jar}", file=sys.stderr)
        return 2

    jar_hashes: set[str] = set()
    jar_entries = 0
    asset_entries = 0
    with zipfile.ZipFile(jar) as archive:
        for info in archive.infolist():
            if info.is_dir():
                continue
            jar_entries += 1
            if info.filename.startswith("assets/minecolonies/"):
                asset_entries += 1
            jar_hashes.add(digest(archive.read(info)))

    tree = sorted(path for path in (PORT / "assets").rglob("*") if path.is_file())
    total = 0
    matched = 0
    matched_bytes = 0
    for path in tree:
        data = path.read_bytes()
        total += len(data)
        if digest(data) in jar_hashes:
            matched += 1
            matched_bytes += len(data)
            print(f"jar-derived: {path.relative_to(PORT)}", file=sys.stderr)

    print(
        f"jar: {jar.name} ({jar_entries} files, {len(jar_hashes)} distinct hashes; "
        f"assets/minecolonies/** entries: {asset_entries})"
    )
    print(f"tree: {len(tree)} files under {PORT / 'assets'}, {total} bytes")
    print(f"jar-derived asset bytes: {matched_bytes} (files matched: {matched})")
    return 0 if matched == 0 else 1


if __name__ == "__main__":
    sys.exit(main())
