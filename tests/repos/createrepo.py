#!/usr/bin/python
# -*- coding: utf-8 -*-
#
# Copyright (C) 2007-2010, TUBITAK/UEKAE
#
# This program is free software; you can redistribute it and/or modify it under
# the terms of the GNU General Public License as published by the Free
# Software Foundation; either version 2 of the License, or (at your option)
# any later version.
#
# Please read the COPYING file.

import os
import shutil
import time

pspecTemplate = """<?xml version="1.0" ?>
<!DOCTYPE PISI SYSTEM "http://www.pardus.org.tr/projeler/pisi/pisi-spec.dtd">
<PISI>
    <Source>
        <Name>%(package)s</Name>
        <Homepage>%(homepage)s</Homepage>
        <Packager>
            <Name>%(packager_name)s</Name>
            <Email>%(packager_email)s</Email>
        </Packager>
        <License>GPL-2</License>
        <IsA>app:gui</IsA>
        <Summary>%(summary)s</Summary>
        <Description>%(description)s</Description>
        <Archive sha1sum="%(sha1sum)s" type="targz">%(archive)s</Archive>
    </Source>

    <Package>
        <Name>%(package)s</Name>
        <RuntimeDependencies>
            %(runtimedeps)s
        </RuntimeDependencies>
        <Files>
            <Path fileType="data">/usr/bin</Path>
        </Files>
    </Package>

    <History>
        <Update release="1">
            <Date>%(date)s</Date>
            <Version>0.3</Version>
            <Comment>First release</Comment>
            <Name>%(packager_name)s</Name>
            <Email>%(packager_email)s</Email>
        </Update>
    </History>
</PISI>
"""

componentsTemplate = """
        <Component>
            <Name>%(name)s</Name>
            <LocalName>%(local_name)s</LocalName>
            <Summary>%(summary)s</Summary>
            <Description>%(description)s</Description>
            <Group>system</Group>
            <Packager>
                <Name>Joe Packager</Name>
                <Email>joe@pardus.org.tr</Email>
            </Packager>
        </Component>
"""

componentTemplate = """
<PISI>
    <Name>%(name)s</Name>
</PISI>
"""

actionsTemplate = """
from pisi.actionsapi import pisitools

WorkDir = "skeleton"

def install():
    pisitools.dobin("skeleton.py")
    pisitools.rename("/usr/bin/skeleton.py", "%s")
"""

distributionTemplate = """
<PISI>
    <SourceName>%(sourcename)s</SourceName>
    <Version>%(version)s</Version>
    <Description>%(description)s</Description>
    <Type>Core</Type>
    <Obsoletes>
        %(obsoletes)s
    </Obsoletes>
</PISI>
"""

class Component:
    def __init__(self, name):
        self.name = name

    def get_comp_template(self, subcomp):
        return componentTemplate % {"name": subcomp}

    def get_comp_path(self):
        return "/".join(self.name.split("."))

    def create(self):
        component_path = self.get_comp_path()
        if not os.path.exists(component_path):
            os.makedirs(component_path)

            cur_dir = os.getcwd()
            cur_comp = ''
            for subcomp in self.name.split("."):
                os.chdir(subcomp)

                if not cur_comp:
                    cur_comp = subcomp
                else:
                    cur_comp = ".".join([cur_comp, subcomp])

                open("component.xml", "w").write(self.get_comp_template(cur_comp))
            os.chdir(cur_dir)

class Package:

    def __init__(self, name, partof, deps):
        self.name = name
        self.partof = partof
        self.deps = deps
        self.component = Component(self.partof)
        
        # Create skeleton archive if not exists (calculate path before chdir)
        self.skeleton_path = os.path.abspath("skeleton.tar.gz")
        if not os.path.exists(self.skeleton_path):
            os.makedirs("skeleton", exist_ok=True)
            with open("skeleton/skeleton.py", "w") as f:
                f.write("print('hello')\n")
            import tarfile
            with tarfile.open(self.skeleton_path, "w:gz") as tar:
                tar.add("skeleton")

    def get_spec_template(self):
        package =  self.name
        homepage = "www.pardus.org.tr"
        packager_name = "Joe Packager"
        packager_email = "joe@pardus.org.tr"
        summary = "%s is a very useful package" % self.name
        description = "%s is a very useful package that is known for its usefulness." % self.name
        
        import hashlib
        with open(self.skeleton_path, "rb") as f:
            sha1sum = hashlib.sha1(f.read()).hexdigest()
            
        archive = "file://" + self.skeleton_path
        date = time.strftime("%Y-%m-%d")
        partof = self.partof

        runtimedeps = ""
        for dep in self.deps:
            runtimedeps += "        <Dependency>%s</Dependency>\n" % dep

        return pspecTemplate % locals()

    def create(self):
        self.component.create()
        cur_dir = os.getcwd()
        os.chdir(self.component.get_comp_path())
        os.makedirs(self.name)
        os.chdir(self.name)
        open("pspec.xml", "w").write(self.get_spec_template())
        open("actions.py", "w").write(actionsTemplate % self.name)
        os.chdir(cur_dir)

class PackageFactory:
    def getPackage(self, name, runtimeDeps = [], component = "system.base"):
        return Package(name, component, runtimeDeps)

    def getPackageBundle(self, component, *packages):
        pkgs = []
        for pkg in packages:
            pkgs.append(Package(pkg, component, []))
        return pkgs

class Repository:
    def __init__(self, name, version, packages, obsoletes):
        self.name = name
        self.version = version
        self.packages = packages
        self.obsoletes = obsoletes

    def get_dist_template(self):
        obsoletes = ""
        for obs in self.obsoletes:
            obsoletes += "     <Package>%s</Package>\n" % obs
            
        return distributionTemplate % {"sourcename": "Pardus",
                                       "version": self.version,
                                       "description":self.name,
                                       "obsoletes":obsoletes}

    def create(self):
        cur_dir = os.getcwd()
        os.makedirs(self.name)
        os.chdir(self.name)
        open("distribution.xml", "w").write(self.get_dist_template())
        
        for pkg in self.packages:
            pkg.create()

        self.create_components_xml()

        os.chdir(cur_dir)

    def create_components_xml(self):
        xml_content = "<PISI>\n    <Components>"

        for root, dirs, files in os.walk("."):
            if "component.xml" in files:
                component = root[2:].replace("/", ".")
                xml_content += componentsTemplate \
                                % {"name": component,
                                   "local_name": component,
                                   "summary": component,
                                   "description": component}

        xml_content += "    </Components>\n</PISI>\n"

        open("components.xml", "w").write(xml_content)


class Pardus2007Repo(Repository):
    def __init__(self):
        Repository.__init__(self, "pardus-2007", "2007", [], ["wengophone", "rar"])
        
    def create(self):

        pf = PackageFactory()

        self.packages = [
            # system.base
            pf.getPackage("bash"),
            pf.getPackage("curl", ["libidn", "zlib", "openssl"]),
            pf.getPackage("shadow", ["db4","pam", "cracklib"]),
            pf.getPackage("jpeg"),
            
            # applications.network
            pf.getPackage("ncftp", [], "applications.network"),
            pf.getPackage("bogofilter", ["gsl"], "applications.network"),
            pf.getPackage("gsl", [], "applications.network"),
            ]
        
        # system.base
        self.packages.extend(pf.getPackageBundle("system.base", "libidn", "zlib", "openssl", "db4", "pam", "cracklib"))

        # applications.network
        self.packages.extend(pf.getPackageBundle("applications.network", "ethtool", "nfdump"))

        Repository.create(self)

class Contrib2007Repo(Repository):
    def __init__(self):
        Repository.__init__(self, "contrib-2007", "2007", [], ["xara"])
        
    def create(self):

        pf = PackageFactory()

        self.packages = [
            # applications.network
            pf.getPackage("lynx", [], "applications.network"),
            pf.getPackage("ctorrent", ["openssl"], "applications.network"),
            pf.getPackage("lft", ["libpcap"], "applications.network"),
            pf.getPackage("libpcap", [], "applications.network"),
            ]
        
        # applications.util
        self.packages.extend(pf.getPackageBundle("applications.util", "iat", "rpl", "cpulimit"))

        Repository.create(self)

class BuildFarm:
    def create_index(self, repo):
        import sys
        binrepo = "%s-bin" % repo
        shutil.copy("%s/distribution.xml" % repo, binrepo)
        os.system("PYTHONPATH=../.. %s ../../pisi-cli index %s --skip-signing -o %s/pisi-index.xml -D ../tmp" % (sys.executable, repo, repo))
        os.system("PYTHONPATH=../.. %s ../../pisi-cli index --skip-sources --skip-signing -o %s/pisi-index.xml %s %s -D ../tmp" % (sys.executable, binrepo, binrepo, repo))

    def build(self, repos):
        import sys
        for repo in repos:
            binrepo = "%s-bin" % repo
            if os.path.exists(binrepo):
                shutil.rmtree(binrepo)
            os.mkdir(binrepo)
            for root, dirs, files in os.walk(repo):
                if "pspec.xml" in files:
                    os.system("PYTHONPATH=../.. %s ../../pisi-cli build %s/%s -O %s -D ../tmp" % (sys.executable, root, "pspec.xml", binrepo))
            self.create_index(repo)

if __name__ == "__main__":
    Pardus2007Repo().create()
    Contrib2007Repo().create()
    BuildFarm().build(["pardus-2007", "contrib-2007", "repo1", "repo2"])
