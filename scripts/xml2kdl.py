#!/usr/bin/env python3
"""Batch convert pspec.xml → pspec.kdl while preserving actions.py & translations.xml."""

import os, sys, re, textwrap
from xml.etree import ElementTree as ET
from pathlib import Path

# ── helpers ──────────────────────────────────────────────────────────────

def kdl_escape(s: str) -> str:
    """Escape string for KDL double-quoted value."""
    s = s.replace("\\", "\\\\")
    s = s.replace('"', '\\"')
    s = s.replace("\n", "\\n")
    s = s.replace("\t", "\\t")
    return s

def text(e: ET.Element | None) -> str:
    """Element text or empty."""
    if e is None or e.text is None:
        return ""
    return e.text.strip()

def attr(e: ET.Element | None, key: str, default: str = "") -> str:
    if e is None:
        return default
    return e.get(key, default)

def bool_attr(e: ET.Element | None, key: str) -> bool:
    v = attr(e, key)
    return v.lower() in ("true", "yes", "1") if v else False

def sub_text(parent: ET.Element, tag: str) -> str:
    el = parent.find(tag)
    return text(el)

def indent(level: int) -> str:
    return "    " * level

# ── generators ───────────────────────────────────────────────────────────

def gen_value(val: str, level: int) -> list[str]:
    """Return KDL lines for a simple node name→value or name={children}."""
    return [f'{indent(level)}{val}']

def gen_section(name: str, lines: list[str], level: int) -> list[str]:
    if not lines:
        return []
    out = [f"{indent(level)}{name} {{"]
    for l in lines:
        out.append(l)
    out.append(f"{indent(level)}}}")
    return out

def gen_packager(el: ET.Element | None, level: int) -> list[str]:
    if el is None:
        return []
    pk_name = sub_text(el, "Name")
    pk_email = sub_text(el, "Email")
    if not pk_name and not pk_email:
        return []
    lines = [
        f'{indent(level+1)}Name "{kdl_escape(pk_name)}"',
        f'{indent(level+1)}Email "{kdl_escape(pk_email)}"',
    ]
    return [f"{indent(level)}Packager {{"] + lines + [f"{indent(level)}}}"]

def gen_archive(el: ET.Element | None, level: int) -> list[str]:
    if el is None:
        return []
    sha1 = attr(el, "sha1sum")
    atype = attr(el, "type")
    name_attr = attr(el, "name")
    url = text(el)
    props_parts = []
    if sha1:
        props_parts.append('sha1sum="' + sha1 + '"')
    if atype:
        props_parts.append('type="' + atype + '"')
    props_str = " " + " ".join(props_parts) if props_parts else ""
    if name_attr:
        lines = [f"{indent(level)}Archive{props_str} {{"]
        lines.append(f'{indent(level+1)}"{kdl_escape(name_attr)}"')
        if url:
            lines.append(f'{indent(level+1)}"{kdl_escape(url)}"')
        lines.append(f"{indent(level)}}}")
        return lines
    else:
        return [f"{indent(level)}Archive{props_str} {{", f'{indent(level+1)}"{kdl_escape(url)}"', f"{indent(level)}}}"]

def gen_dependencies(el: ET.Element | None, level: int, section: str = "BuildDependencies") -> list[str]:
    if el is None:
        return []
    deps = el.findall("Dependency")
    if not deps:
        return []
    lines = []
    for d in deps:
        name = text(d)
        release = attr(d, "release")
        version_from = attr(d, "versionFrom")
        version_to = attr(d, "versionTo")
        if version_from:
            lines.append(f'{indent(level+1)}Dependency "{kdl_escape(name)}" versionFrom="{version_from}"')
        elif version_to:
            lines.append(f'{indent(level+1)}Dependency "{kdl_escape(name)}" versionTo="{version_to}"')
        elif release:
            lines.append(f'{indent(level+1)}Dependency "{kdl_escape(name)}" release="{release}"')
        else:
            lines.append(f'{indent(level+1)}Dependency "{kdl_escape(name)}"')
    return gen_section(section, lines, level)

def gen_isa(el: ET.Element | None, level: int) -> list[str]:
    if el is None:
        return []
    isas = [f'{indent(level)}Isa "{kdl_escape(e)}"' for e in el]
    return gen_section("Provides", isas, level)

def gen_files(pkg: ET.Element, level: int) -> list[str]:
    files_el = pkg.find("Files")
    if files_el is None:
        return []
    lines = []
    for p in files_el.findall("Path"):
        ft = attr(p, "fileType")
        val = text(p)
        if ft:
            lines.append(f'{indent(level+1)}Path "{kdl_escape(val)}" file-type="{ft}"')
        else:
            lines.append(f'{indent(level+1)}Path "{kdl_escape(val)}"')
    if not lines:
        return []
    return [f"{indent(level)}Files {{"] + lines + [f"{indent(level)}}}"]

def gen_additional_files(pkg: ET.Element, level: int) -> list[str]:
    af_el = pkg.find("AdditionalFiles")
    if af_el is None:
        return []
    lines = []
    for af in af_el.findall("AdditionalFile"):
        target = attr(af, "target")
        owner = attr(af, "owner")
        perm = attr(af, "permission")
        src = text(af)
        props = ""
        if target:
            props += f' target="{kdl_escape(target)}"'
        if owner:
            props += f' owner="{owner}"'
        if perm:
            props += f' permission="{perm}"'
        lines.append(f'{indent(level+1)}AdditionalFile "{kdl_escape(src)}"{props}')
    if not lines:
        return []
    return [f"{indent(level)}AdditionalFiles {{"] + lines + [f"{indent(level)}}}"]

def gen_provides(pkg: ET.Element, level: int) -> list[str]:
    """COMAR provides within a Package."""
    prov = pkg.find("Provides")
    if prov is None:
        return []
    lines = []
    for comar in prov.findall("COMAR"):
        script = attr(comar, "script")
        val = text(comar)
        if script:
            lines.append(f'{indent(level+1)}COMAR "{kdl_escape(val)}" script="{script}"')
        else:
            lines.append(f'{indent(level+1)}COMAR "{kdl_escape(val)}"')
    if not lines:
        return []
    return [f"{indent(level)}Provides {{"] + lines + [f"{indent(level)}}}"]

def gen_actions_from_xml(pkg: ET.Element, level: int) -> list[str]:
    """Generate Actions from inline XML <Actions> or fallback to actions.py."""
    # First check for inline <Actions>
    act_el = pkg.find("Actions")
    if act_el is not None:
        lines = []
        for action in act_el.findall("Action"):
            pkg_attr = attr(action, "package")
            val = text(action)
            if pkg_attr:
                lines.append(f'{indent(level+1)}Action "{kdl_escape(val)}" package="{pkg_attr}"')
            else:
                lines.append(f'{indent(level+1)}Action "{kdl_escape(val)}"')
        if lines:
            return [f"{indent(level)}Actions {{"] + lines + [f"{indent(level)}}}"]
    # Fallback: use actions.py — just reference step names
    return [f"{indent(level)}Actions {{",
            f'{indent(level+1)}steps "setup"',
            f'{indent(level+1)}steps "build"',
            f'{indent(level+1)}steps "install"',
            f"{indent(level)}}}"]

def gen_history(hist_el: ET.Element | None, level: int) -> list[str]:
    if hist_el is None:
        return []
    updates = hist_el.findall("Update")
    if not updates:
        return []
    out = [f"{indent(level)}History {{"]
    for u in updates:
        release = attr(u, "release")
        date_el = u.find("Date")
        date_str = text(date_el) if date_el is not None else ""
        version = sub_text(u, "Version")
        comment = sub_text(u, "Comment")
        name = sub_text(u, "Name")
        email = sub_text(u, "Email")
        props = f'release={release}'
        if date_str:
            props += f' date="{date_str}"'
        out.append(f'{indent(level+1)}Update {props} {{')
        out.append(f'{indent(level+2)}Version "{kdl_escape(version)}"')
        out.append(f'{indent(level+2)}Comment "{kdl_escape(comment)}"')
        out.append(f'{indent(level+2)}Name "{kdl_escape(name)}"')
        out.append(f'{indent(level+2)}Email "{kdl_escape(email)}"')
        out.append(f'{indent(level+1)}}}')
    out.append(f"{indent(level)}}}")
    return out

def gen_patches(patches_el: ET.Element | None, level: int) -> list[str]:
    if patches_el is None:
        return []
    patches = patches_el.findall("Patch")
    if not patches:
        return []
    lines = []
    for p in patches:
        level_attr = p.get("level", "")
        compression = p.get("compressionType", "")
        val = text(p)
        props = ""
        if level_attr:
            props += f' level="{level_attr}"'
        if compression:
            props += f' compressionType="{compression}"'
        lines.append(f'{indent(level+1)}Patch "{kdl_escape(val)}"{props}')
    return [f"{indent(level)}Patches {{"] + lines + [f"{indent(level)}}}"]

# ── main converter ───────────────────────────────────────────────────────

def convert_one(xml_path: str, kdl_path: str, *, force: bool = False, no_clobber: bool = False):
    """Convert a single pspec.xml to {package_name}.kdl."""
    if os.path.exists(kdl_path):
        if no_clobber:
            return False, f"SKIP (exists)"
        if not force:
            return False, f"SKIP (use --force to overwrite)"

    # Parse XML (handle entities by using a custom parser)
    try:
        tree = ET.parse(xml_path)
    except ET.ParseError as e:
        return False, f"XML parse error: {e}"

    root = tree.getroot()
    if root.tag != "PISI":
        return False, f"Unexpected root tag: {root.tag}"

    src = root.find("Source")
    if src is None:
        return False, "No <Source> found"

    out_lines = ["PisiPackage {"]  # Start PisiPackage

    # ── Source ──
    source_lines = []
    pkg_name = sub_text(src, "Name")
    source_lines.append(f'{indent(2)}Name "{kdl_escape(pkg_name)}"')
    source_lines.append(f'{indent(2)}Homepage "{kdl_escape(sub_text(src, "Homepage"))}"')

    pkg_lines = gen_packager(src.find("Packager"), 2)
    source_lines.extend(pkg_lines)

    license_val = sub_text(src, "License")
    if license_val:
        # License "GPLv2 FTL" gibi boşlukla ayrılmış birden çok lisans olabilir
        for lic in license_val.split():
            source_lines.append(f'{indent(2)}License "{kdl_escape(lic)}"')

    partof = sub_text(src, "PartOf")
    if partof:
        source_lines.append(f'{indent(2)}PartOf "{kdl_escape(partof)}"')

    source_lines.append(f'{indent(2)}Summary "{kdl_escape(sub_text(src, "Summary"))}"')
    desc = sub_text(src, "Description")
    if desc:
        source_lines.append(f'{indent(2)}Description "{kdl_escape(desc)}"')

    # IsA → Provides.isa
    isa_els = src.findall("IsA")
    if isa_els:
        isa_lines = [f'{indent(3)}Isa "{kdl_escape(text(e))}"' for e in isa_els]
        source_lines.append(f"{indent(2)}Provides {{")
        source_lines.extend(isa_lines)
        source_lines.append(f"{indent(2)}}}")

    # Archive
    archive_el = src.find("Archive")
    if archive_el is not None:
        source_lines.extend(gen_archive(archive_el, 2))

    # BuildDependencies
    bdeps = src.find("BuildDependencies")
    if bdeps is not None and bdeps.findall("Dependency"):
        source_lines.extend(gen_dependencies(bdeps, 2, "BuildDependencies"))

    # Patches
    patches_el = src.find("Patches")
    if patches_el is not None:
        source_lines.extend(gen_patches(patches_el, 2))

    out_lines.append(f"{indent(1)}Source {{")
    out_lines.extend(source_lines)
    out_lines.append(f"{indent(1)}}}")

    # ── Packages ──
    for pkg in root.findall("Package"):
        name = sub_text(pkg, "Name")
        partof = sub_text(pkg, "PartOf")
        summary = sub_text(pkg, "Summary")
        build_type = sub_text(pkg, "BuildType")
        pkg_lines = [f'{indent(2)}Name "{kdl_escape(name)}"']

        if summary:
            pkg_lines.append(f'{indent(2)}Summary "{kdl_escape(summary)}"')

        if partof:
            pkg_lines.append(f'{indent(2)}PartOf "{kdl_escape(partof)}"')

        if build_type:
            pkg_lines.append(f'{indent(2)}BuildType "{kdl_escape(build_type)}"')

        # RuntimeDependencies
        rdeps = pkg.find("RuntimeDependencies")
        if rdeps is not None and rdeps.findall("Dependency"):
            pkg_lines.extend(gen_dependencies(rdeps, 2, "RuntimeDependencies"))

        # BuildDependencies (per-package)
        bdeps = pkg.find("BuildDependencies")
        if bdeps is not None and bdeps.findall("Dependency"):
            pkg_lines.extend(gen_dependencies(bdeps, 2, "BuildDependencies"))

        # Files
        pkg_lines.extend(gen_files(pkg, 2))

        # AdditionalFiles
        pkg_lines.extend(gen_additional_files(pkg, 2))

        # Icon & Screenshot (hardcoded defaults)
        pkg_lines.append(f'{indent(2)}Icon "pisi-software-all"')
        pkg_lines.append(f'{indent(2)}Screenshot ""')

        # Provides (COMAR)
        pkg_lines.extend(gen_provides(pkg, 2))

        # Actions (inline XML or fallback to actions.py)
        pkg_lines.extend(gen_actions_from_xml(pkg, 2))

        out_lines.append(f"{indent(1)}Package {{")
        out_lines.extend(pkg_lines)
        out_lines.append(f"{indent(1)}}}")

    # ── History ──
    hist_el = root.find("History")
    out_lines.extend(gen_history(hist_el, 1))

    # Close PisiPackage
    out_lines.append("}")

    # Write
    content = "\n".join(out_lines) + "\n"
    with open(kdl_path, "w", encoding="utf-8") as f:
        f.write(content)

    return True, f"OK ({len(root.findall('Package'))} pkgs, {len(hist_el.findall('Update')) if hist_el is not None else 0} history)"

# ── CLI ──────────────────────────────────────────────────────────────────

def main():
    import argparse
    ap = argparse.ArgumentParser(description="Batch convert pspec.xml → pspec.kdl")
    ap.add_argument("paths", nargs="*", help="Paths to pspec.xml files or directories to scan")
    ap.add_argument("--force", "-f", action="store_true", help="Overwrite existing .kdl files")
    ap.add_argument("--no-clobber", action="store_true", help="Skip if .kdl exists")
    ap.add_argument("--recursive", "-r", action="store_true", help="Scan directories recursively")
    args = ap.parse_args()

    # Collect files
    files: list[str] = []
    for p in args.paths or ["."]:
        if os.path.isfile(p):
            files.append(p)
        elif os.path.isdir(p):
            if args.recursive:
                for dirpath, _, filenames in os.walk(p):
                    for fn in filenames:
                        if fn == "pspec.xml":
                            files.append(os.path.join(dirpath, fn))
            else:
                for fn in os.listdir(p):
                    if fn == "pspec.xml":
                        files.append(os.path.join(p, fn))

    if not files:
        print("No pspec.xml files found.")
        sys.exit(1)

    ok = skip = fail = 0
    for fpath in sorted(files):
        dpath = os.path.dirname(fpath)
        # Çıktı her zaman pspec.kdl
        try:
            name_tree = ET.parse(fpath)
            name_root = name_tree.getroot()
            name_src = name_root.find("Source")
            pkg_name = text(name_src.find("Name")) if name_src is not None else "pspec"
        except Exception:
            pkg_name = "pspec"
        kpath = os.path.join(dpath, "pspec.kdl")
        status, msg = convert_one(fpath, kpath, force=args.force, no_clobber=args.no_clobber)
        if status:
            ok += 1
        elif msg.startswith("SKIP"):
            skip += 1
        else:
            fail += 1
        print(f"  {'✅' if status else '⚠️' if msg.startswith('SKIP') else '❌'} {fpath} → {msg}")

    print(f"\nDone: {ok} converted, {skip} skipped, {fail} failed")

if __name__ == "__main__":
    main()
