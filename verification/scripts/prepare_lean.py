#!/usr/bin/env python3
"""Reuse an explicitly supplied local Lake package cache without altering it."""
import argparse
import json
from pathlib import Path
p = argparse.ArgumentParser()
p.add_argument('cache', type=Path, help='directory containing mathlib, batteries, aesop, ...')
a = p.parse_args()
project = Path(__file__).resolve().parents[1] / 'lean'
manifest = json.loads((project / 'lake-manifest.json').read_text())
out = project / '.lake/packages'
out.mkdir(parents=True, exist_ok=True)
for package in manifest['packages']:
    target = (a.cache / package['name']).resolve(strict=True)
    link = out / package['name']
    if link.is_symlink() and link.resolve() == target:
        continue
    if link.exists() or link.is_symlink():
        raise SystemExit('refusing to replace ' + str(link))
    link.symlink_to(target, target_is_directory=True)
print('Prepared local package links; runner verifies every pinned git revision.')
