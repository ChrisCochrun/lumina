#!/usr/bin/env python3
"""Split Flatpak Cargo's registry and Git vendor sources to avoid duplicate crates.

Usage: python3 split_flatpak_cargo_sources.py cargo-sources.json cargo-sources-split.json
"""
import json
import re
import sys
from pathlib import Path

if len(sys.argv) != 3:
    sys.exit('usage: python3 split_flatpak_cargo_sources.py INPUT.json OUTPUT.json')

sources = json.loads(Path(sys.argv[1]).read_text())
base = 'cargo/vendor/'
registry_dirs = {
    source['dest']
    for source in sources
    if source.get('type') == 'archive'
    and source.get('url', '').startswith('https://static.crates.io/crates/')
    and source.get('dest', '').startswith(base)
}
if 'cargo/vendor/dpi-0.1.2' not in registry_dirs:
    sys.exit('Missing registry dpi 0.1.2: regenerate cargo-sources.json from Cargo.lock first')

for source in sources:
    if source.get('type') == 'shell':
        source['commands'] = [
            command.replace('"cargo/vendor/', '"cargo/vendor-git/')
            for command in source['commands']
        ]
    elif source.get('dest', '').startswith(base):
        old_dest = source['dest']
        group = 'vendor-registry' if old_dest in registry_dirs else 'vendor-git'
        source['dest'] = old_dest.replace(base, f'cargo/{group}/', 1)
    elif source.get('type') == 'inline' and source.get('dest') == 'cargo' and source.get('dest-filename') == 'config':
        config = source['contents']
        header = '[source.vendored-sources]\ndirectory = "cargo/vendor"'
        if header not in config:
            sys.exit('Unrecognized generated Cargo source configuration')
        config = config.replace(header,
            '[source.vendored-registry]\ndirectory = "cargo/vendor-registry"\n\n'
            '[source.vendored-git]\ndirectory = "cargo/vendor-git"')
        config = config.replace('replace-with = "vendored-sources"', 'replace-with = "vendored-git"')
        config = config.replace(
            '[source.crates-io]\nreplace-with = "vendored-git"',
            '[source.crates-io]\nreplace-with = "vendored-registry"'
        )
        if '[source.crates-io]\nreplace-with = "vendored-registry"' not in config:
            sys.exit('Missing crates.io source replacement in generated config')
        source['contents'] = config

sources.insert(0, {'type': 'shell', 'commands': ['mkdir -p cargo/vendor-registry cargo/vendor-git']})
Path(sys.argv[2]).write_text(json.dumps(sources, indent=2) + '\n')
print(f'Wrote {sys.argv[2]} ({len(registry_dirs)} registry crates, Git crates separated)')
