#!/usr/bin/env python3
"""Convert components.xml, groups.xml, distribution.xml → KDL format."""

import os, re, sys
from xml.etree import ElementTree as ET
from pathlib import Path

def kdl_escape(s):
    s = s.replace("\\", "\\\\")
    s = s.replace('"', '\\"')
    s = s.replace("\n", "\\n")
    return s

def text(e):
    return (e.text or "").strip()

def attr(e, key, default=""):
    return e.get(key, default)

# ── components.xml → components.kdl ──

def convert_components(xml_path, kdl_path):
    tree = ET.parse(xml_path)
    root = tree.getroot()
    comps = root.find(".//Components")
    if comps is None:
        print("No <Components> found")
        return

    lines = ["// LupuS Components\n"]
    for c in comps.findall("Component"):
        name = text(c.find("Name"))
        lines.append(f'component "{kdl_escape(name)}" {{')
        for ln in c.findall("LocalName"):
            lang = attr(ln, "{http://www.w3.org/XML/1998/namespace}lang", "en")
            lines.append(f'    local-name lang="{lang}" "{kdl_escape(text(ln))}"')
        for s in c.findall("Summary"):
            lang = attr(s, "{http://www.w3.org/XML/1998/namespace}lang", "en")
            lines.append(f'    summary lang="{lang}" "{kdl_escape(text(s))}"')
        for d in c.findall("Description"):
            lang = attr(d, "{http://www.w3.org/XML/1998/namespace}lang", "en")
            lines.append(f'    description lang="{lang}" "{kdl_escape(text(d))}"')
        grp = c.find("Group")
        if grp is not None:
            lines.append(f'    group "{kdl_escape(text(grp))}"')
        maint = c.find("Maintainer")
        if maint is not None:
            nm = text(maint.find("Name"))
            em = text(maint.find("Email"))
            if nm:
                lines.append(f'    maintainer-name "{kdl_escape(nm)}"')
            if em:
                lines.append(f'    maintainer-email "{kdl_escape(em)}"')
        lines.append("}\n")

    with open(kdl_path, "w", encoding="utf-8") as f:
        f.write("\n".join(lines) + "\n")
    print(f"  ✅ {kdl_path} ({len(comps)} components)")


# ── groups.xml → groups.kdl ──

def convert_groups(xml_path, kdl_path):
    tree = ET.parse(xml_path)
    root = tree.getroot()
    groups_el = root.find(".//Groups")
    if groups_el is None:
        print("No <Groups> found")
        return

    lines = ["// LupuS Groups\n"]
    for g in groups_el.findall("Group"):
        name = text(g.find("Name"))
        lines.append(f'group "{kdl_escape(name)}" {{')
        for ln in g.findall("LocalName"):
            lang = attr(ln, "{http://www.w3.org/XML/1998/namespace}lang", "en")
            lines.append(f'    local-name lang="{lang}" "{kdl_escape(text(ln))}"')
        icon = g.find("Icon")
        if icon is not None:
            lines.append(f'    icon "{kdl_escape(text(icon))}"')
        lines.append("}\n")

    with open(kdl_path, "w", encoding="utf-8") as f:
        f.write("\n".join(lines) + "\n")
    print(f"  ✅ {kdl_path} ({len(groups_el)} groups)")


# ── distribution.xml → distribution.kdl ──

def convert_distribution(xml_path, kdl_path):
    tree = ET.parse(xml_path)
    root = tree.getroot()

    lines = ["// LupuS Distribution\n"]
    lines.append("distribution {")

    sn = root.find("SourceName")
    if sn is not None:
        lines.append(f'    source-name "{kdl_escape(text(sn))}"')

    ver = root.find("Version")
    if ver is not None:
        lines.append(f'    version "{kdl_escape(text(ver))}"')

    for d in root.findall("Description"):
        lang = attr(d, "{http://www.w3.org/XML/1998/namespace}lang", "en")
        lines.append(f'    description lang="{lang}" "{kdl_escape(text(d))}"')

    dt = root.find("Type")
    if dt is not None:
        lines.append(f'    type "{kdl_escape(text(dt))}"')

    bn = root.find("BinaryName")
    if bn is not None:
        lines.append(f'    binary-name "{kdl_escape(text(bn))}"')

    obs = root.find("Obsoletes")
    if obs is not None:
        pkgs = obs.findall("Package")
        if pkgs:
            lines.append("    obsoletes {")
            for p in pkgs:
                lines.append(f'        package "{kdl_escape(text(p))}"')
            lines.append("    }")

    lines.append("}\n")

    with open(kdl_path, "w", encoding="utf-8") as f:
        f.write("\n".join(lines) + "\n")
    print(f"  ✅ {kdl_path}")


# ── main ──

if __name__ == "__main__":
    base = "/media/USER/REPO/LUPUS/LupuS_docker/luppo/core"
    convert_components(os.path.join(base, "components.xml"), os.path.join(base, "components.kdl"))
    convert_groups(os.path.join(base, "groups.xml"), os.path.join(base, "groups.kdl"))
    convert_distribution(os.path.join(base, "distribution.xml"), os.path.join(base, "distribution.kdl"))
