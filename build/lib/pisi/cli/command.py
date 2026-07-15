# -*- coding:utf-8 -*-
#
# Copyright (C) 2005 - 2007, TUBITAK/UEKAE
#
# This program is free software; you can redistribute it and/or modify it under
# the terms of the GNU General Public License as published by the Free
# Software Foundation; either version 2 of the License, or (at your option)
# any later version.
#
# Please read the COPYING file.
#

import os
import sys
import argparse

import gettext
__trans = gettext.translation('pisi', fallback=True)
_ = __trans.gettext

import pisi.api
import pisi.context as ctx

class autocommand(type):
    def __init__(cls, name, bases, dict):
        super(autocommand, cls).__init__(name, bases, dict)
        if name == 'Command':
            return

        # Find the root Command class
        Command = None
        for b in cls.__mro__:
            if b.__name__ == 'Command':
                Command = b
                break

        if Command is None:
            return

        if not hasattr(Command, 'cmd'):
            Command.cmd = []
        if not hasattr(Command, 'cmd_dict'):
            Command.cmd_dict = {}
        
        Command.cmd.append(cls)
        name_attr = getattr(cls, 'name', None)
        if name_attr is None:
            # Skip base classes or commands without names
            return
            
        longname, shortname = name_attr
        def add_cmd(cmd):
            if cmd in Command.cmd_dict:
                raise pisi.cli.Error(_('Duplicate command %s') % cmd)
            else:
                Command.cmd_dict[cmd] = cls
        add_cmd(longname)
        if shortname:
            add_cmd(shortname)

class Command(object, metaclass=autocommand):
    """generic help string for any command"""

    # class variables
    cmd = []
    cmd_dict = {}

    @staticmethod
    def commands_string():
        s = ''
        l = [x.name[0] for x in Command.cmd if hasattr(x, 'name') and x.name]
        l.sort()
        for name in l:
            commandcls = Command.cmd_dict[name]
            trans = gettext.translation('pisi', fallback=True)
            doc = commandcls.__doc__ or ""
            summary = trans.gettext(doc).split('\n')[0]
            name = commandcls.name[0]
            if commandcls.name[1]:
                name += ' (%s)' % commandcls.name[1]
            s += ' %23s - %s\n' % (name, summary)
        return s

    @staticmethod
    def get_command(cmd, fail=False, args=None):
        if cmd in Command.cmd_dict:
            return Command.cmd_dict[cmd](args)

        if fail:
            raise pisi.cli.Error(_("Unrecognized command: %s") % cmd)
        else:
            return None

    # instance variables

    def __init__(self, args = None):
        # now for the real parser
        import pisi
        self.comar = False
        
        # We use a custom usage to match the old behavior if needed, 
        # but argparse default is usually fine.
        self.parser = argparse.ArgumentParser(
            prog="pisi " + (self.name[0] if hasattr(self, 'name') else ""),
            description=gettext.translation('pisi', fallback=True).gettext(self.__doc__ or ""),
            formatter_class=PisiHelpFormatter
        )
        
        self.parser.add_argument('--version', action='version', version="%(prog)s " + pisi.__version__)
        
        self.options_group = self.parser.add_argument_group(_("options"))
        self.options(self.options_group)
        self.commonopts()
        
        # In argparse, we usually define positional arguments explicitly.
        # But since pisi commands have varying positionals, we'll use parse_known_args
        # or add a generic 'args' positional.
        self.parser.add_argument('remaining_args', nargs='*')
        
        # argparse expects args to NOT include the command name if we define the parser per command.
        # But pisicli passes orig_args which includes the command name.
        if args and args[0] == (self.name[0] if hasattr(self, 'name') else ""):
            args = args[1:]
        elif args and hasattr(self, 'name') and self.name[1] and args[0] == self.name[1]:
            args = args[1:]

        parsed_args = self.parser.parse_args(args)
        self.options = parsed_args
        self.args = parsed_args.remaining_args

        self.process_opts()

    def commonopts(self):
        '''common options'''
        group = self.parser.add_argument_group(_("general options"))

        group.add_argument("-D", "--destdir", action="store", default = None,
                     help = _("Change the system root for PiSi commands"))
        group.add_argument("-y", "--yes-all", action="store_true",
                     default=False, help = _("Assume yes in all yes/no queries"))
        group.add_argument("-u", "--username", action="store")
        group.add_argument("-p", "--password", action="store")
        group.add_argument("-L", "--bandwidth-limit", action="store", default = 0,
                     help = _("Keep bandwidth usage under specified KB's"))
        group.add_argument("-v", "--verbose", action="store_true",
                     dest="verbose", default=False,
                     help=_("Detailed output"))
        group.add_argument("-d", "--debug", action="store_true",
                     default=False, help=_("Show debugging information"))
        group.add_argument("-N", "--no-color", action="store_true", default=False,
                     help = _("Suppresses all coloring of PiSi's output"))

    def options(self, group):
        """This is a fall back function. If the implementer module provides an
        options function it will be called"""
        pass

    def process_opts(self):
        self.check_auth_info()

        # make destdir absolute
        if self.options.destdir:
            d = str(self.options.destdir)
            if not os.path.exists(d):
                pisi.cli.printu(_('Destination directory %s does not exist. Creating directory.\n') % d)
                os.makedirs(d)
            self.options.destdir = os.path.realpath(d)

    def check_auth_info(self):
        username = self.options.username
        password = self.options.password

        if username and password:
            self.options.authinfo = (username, password)
            return

        if username and not password:
            from getpass import getpass
            password = getpass(_("Password: "))
            self.options.authinfo = (username, password)
        else:
            self.options.authinfo = None

    def init(self, database = True, write = True):
        """initialize PiSi components"""

        if self.options:
            ui = pisi.cli.CLI(self.options.debug, self.options.verbose)
        else:
            ui = pisi.cli.CLI()

        pisi.api.set_userinterface(ui)
        pisi.api.set_options(self.options)
        pisi.api.set_comar(self.comar and not ctx.get_option('ignore_comar'))

        if write and not os.access(pisi.context.config.packages_dir(), os.W_OK):
            raise pisi.cli.Error(_("You have to be root for this operation."))

    def get_name(self):
        return self.__class__.name

    def format_name(self):
        (name, shortname) = self.get_name()
        if shortname:
            return "%s (%s)" % (name, shortname)
        else:
            return name

    def help(self):
        """print help for the command"""
        self.parser.print_help()

    def die(self):
        """exit program"""
        ctx.ui.error(_('Command terminated abnormally.'))
        sys.exit(-1)

class PackageOp(Command):
    """Abstract package operation command"""

    def __init__(self, args):
        super(PackageOp, self).__init__(args)
        self.comar = True

    def options(self, group):
        group.add_argument("--ignore-dependency", action="store_true",
                     default=False,
                     help=_("Do not take dependency information into account"))
        group.add_argument("--ignore-comar", action="store_true",
                     default=False, help=_("Bypass comar configuration agent"))
        group.add_argument("--ignore-safety", action="store_true",
                     default=False, help=_("Bypass safety switch"))
        group.add_argument("-n", "--dry-run", action="store_true", default=False,
                     help = _("Do not perform any action, just show what would be done"))

    def init(self, database=True, write=True):
        super(PackageOp, self).init(database, write)

class PisiHelpFormatter(argparse.HelpFormatter):
    def __init__(self, prog, indent_increment=2, max_help_position=24, width=None):
        super(PisiHelpFormatter, self).__init__(prog, indent_increment, max_help_position, width)

    def _format_action_invocation(self, action):
        if not action.option_strings:
            return super(PisiHelpFormatter, self)._format_action_invocation(action)
        else:
            parts = []
            if action.nargs == 0:
                parts.extend(action.option_strings)
            else:
                default = action.dest.upper()
                args_string = self._format_args(action, default)
                for option_string in action.option_strings:
                    parts.append('%s %s' % (option_string, args_string))
            return ', '.join(parts)
