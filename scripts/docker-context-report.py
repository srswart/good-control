#!/usr/bin/env python3
import argparse
import fnmatch
import os
from pathlib import Path


def load_patterns(root: Path):
    ignore_file = root / ".dockerignore"
    patterns = []
    if ignore_file.exists():
        for line in ignore_file.read_text().splitlines():
            line = line.strip()
            if not line or line.startswith("#"):
                continue
            patterns.append(line)
    return patterns


def matches_pattern(rel_posix: str, pattern: str) -> bool:
    if pattern.endswith("/"):
        pattern = pattern[:-1]
    if pattern and not any(ch in pattern for ch in "*?[]"):
        return rel_posix == pattern or rel_posix.startswith(f"{pattern}/")
    return fnmatch.fnmatch(rel_posix, pattern)


def is_ignored(rel_posix: str, patterns) -> bool:
    return any(matches_pattern(rel_posix, pattern) for pattern in patterns)


def main():
    parser = argparse.ArgumentParser(description="Report Docker build context size.")
    parser.add_argument("--root", default=".", help="Repo root (default: .)")
    parser.add_argument("--top", type=int, default=20, help="Number of largest files to show")
    parser.add_argument("--min-mb", type=float, default=1.0, help="Minimum MB to list")
    args = parser.parse_args()

    root = Path(args.root).resolve()
    patterns = load_patterns(root)

    files = []
    total_size = 0

    for dirpath, dirnames, filenames in os.walk(root):
        rel_dir = Path(dirpath).relative_to(root).as_posix()
        if rel_dir == ".":
            rel_dir = ""

        if rel_dir and is_ignored(rel_dir, patterns):
            dirnames[:] = []
            continue

        pruned = []
        for d in dirnames:
            rel = f"{rel_dir}/{d}" if rel_dir else d
            if is_ignored(rel, patterns):
                continue
            pruned.append(d)
        dirnames[:] = pruned

        for filename in filenames:
            rel = f"{rel_dir}/{filename}" if rel_dir else filename
            if is_ignored(rel, patterns):
                continue
            path = root / rel
            try:
                size = path.stat().st_size
            except OSError:
                continue
            total_size += size
            files.append((size, rel))

    files.sort(reverse=True)
    min_bytes = int(args.min_mb * 1024 * 1024)

    print(f"Context size (approx): {total_size/1024/1024:.1f} MB")
    print(f"Top {args.top} files >= {args.min_mb:.1f} MB:")
    shown = 0
    for size, rel in files:
        if size < min_bytes:
            break
        print(f"{size/1024/1024:8.1f} MB  {rel}")
        shown += 1
        if shown >= args.top:
            break


if __name__ == "__main__":
    main()
