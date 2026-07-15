# -*- coding:utf-8 -*-
#
# Copyright (C) 2005-2010, TUBITAK/UEKAE
#
# This program is free software; you can redistribute it and/or modify it under
# the terms of the GNU General Public License as published by the Free
# Software Foundation; either version 2 of the License, or (at your option)
# any later version.
#
# Please read the COPYING file.
#


import gettext
__trans = gettext.translation('pisi', fallback=True)
_ = __trans.gettext

import pisi.cli.command as command
import pisi.context as ctx
import pisi.api
import pisi.db

class Install(command.PackageOp, metaclass=command.autocommand):
    __doc__ = _("""Install PiSi packages

Usage: install <package1> <package2> ... <packagen>

You may use filenames, URI's or package names for packages. If you have
specified a package name, it should exist in a specified repository.

You can also specify components instead of package names, which will be
expanded to package names.
""")

    def __init__(self, args):
        super(Install, self).__init__(args)
        self.componentdb = pisi.db.componentdb.ComponentDB()

    name = "install", "it"

    def options(self, group):
        group.title = _("install options")

        super(Install, self).options(group)

        group.add_argument("--reinstall", action="store_true",
                     default=False, help=_("Reinstall already installed packages"))
        group.add_argument("--ignore-check", action="store_true",
                     default=False, help=_("Skip distribution release and architecture check"))
        group.add_argument("--ignore-file-conflicts", action="store_true",
                     default=False, help=_("Ignore file conflicts"))
        group.add_argument("--ignore-package-conflicts", action="store_true",
                     default=False, help=_("Ignore package conflicts"))
        group.add_argument("-c", "--component", action="append",
                               default=None, help=_("Install component's and recursive components' packages"))
        group.add_argument("-r", "--repository", action="store",
                               type=str, default=None, help=_('Name of the component\'s repository'))
        group.add_argument("-f", "--fetch-only", action="store_true",
                     default=False, help=_("Fetch upgrades but do not install."))
        group.add_argument("-x", "--exclude", action="append",
                     default=None, help=_("When installing packages, ignore packages and components whose basenames match pattern."))
        group.add_argument("--exclude-from", action="store",
                     default=None,
                     help=_("When installing packages, ignore packages "
                            "and components whose basenames match "
                            "any pattern contained in file."))

    def run(self):

        if self.options.fetch_only:
            self.init(database=True, write=False)
        else:
            self.init()

        components = ctx.get_option('component')
        if not components and not self.args:
            self.help()
            return

        packages = []
        if components:
            for name in components:
                if self.componentdb.has_component(name):
                    repository = ctx.get_option('repository')
                    if repository:
                        packages.extend(self.componentdb.get_packages(name, walk=True, repo=repository))
                    else:
                        packages.extend(self.componentdb.get_union_packages(name, walk=True))
                else:
                    ctx.ui.info(_('There is no component named %s') % name)

        packages.extend(self.args)

        if ctx.get_option('exclude_from'):
            packages = pisi.blacklist.exclude_from(packages, ctx.get_option('exclude_from'))

        if ctx.get_option('exclude'):
            packages = pisi.blacklist.exclude(packages, ctx.get_option('exclude'))

        reinstall = bool(packages) and packages[0].endswith(ctx.const.package_suffix)
        pisi.api.install(packages, ctx.get_option('reinstall') or reinstall)
