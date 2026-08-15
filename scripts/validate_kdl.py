#!/usr/bin/env python3
"""Validate all .kdl files by running convert_kdl on each."""
import subprocess, glob, os, sys, tempfile

# Compile/run convert_kdl on each .kdl file
import json

def main():
    base = "/media/luppocuk/REPO/LUPUS/LupuS_docker/core/system/base"
    kdl_files = []
    for dirpath, _, fnames in os.walk(base):
        for fn in fnames:
            if fn == "lopec.kdl":
                kdl_files.append(os.path.join(dirpath, fn))

    # Also add project-level ones
    proj = "/media/luppocuk/REPO/RUST/project/luppo"
    for fn in ["luppo.kdl", "luppo_template.kdl", "lopec.kdl"]:
        kdl_files.append(os.path.join(proj, fn))

    ok = fail = 0
    for kf in sorted(kdl_files):
        content = open(kf).read()
        with tempfile.NamedTemporaryFile(mode='w', suffix='.kdl', delete=False) as f:
            f.write(content)
            f.flush()
            result = subprocess.run(
                ["cargo", "run", "--bin", "convert_kdl_test", "--", f.name],
                cwd=proj, capture_output=True, text=True, timeout=30
            )
            os.unlink(f.name)

if __name__ == "__main__":
    main()
