# -*- coding: utf-8 -*-
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

import sys
import argparse

import gettext
__trans = gettext.translation('pisi', fallback=True)
_ = __trans.gettext

import pisi
import pisi.cli
import pisi.cli.command as command
import pisi.cli.addrepo
import pisi.cli.blame
import pisi.cli.build
import pisi.cli.check
import pisi.cli.clean
import pisi.cli.configurepending
import pisi.cli.deletecache
import pisi.cli.delta
import pisi.cli.emerge
import pisi.cli.fetch
import pisi.cli.graph
import pisi.cli.index
import pisi.cli.info
import pisi.cli.install
import pisi.cli.history
import pisi.cli.listnewest
import pisi.cli.listavailable
import pisi.cli.listcomponents
import pisi.cli.listinstalled
import pisi.cli.listpending
import pisi.cli.listrepo
import pisi.cli.listsources
import pisi.cli.listupgrades
import pisi.cli.rebuilddb
import pisi.cli.remove
import pisi.cli.removerepo
import pisi.cli.enablerepo
import pisi.cli.disablerepo
import pisi.cli.searchfile
import pisi.cli.search
import pisi.cli.updaterepo
import pisi.cli.upgrade

#FIXME: why does this has to be imported last
import pisi.cli.help

class PisiCLI(object):

    def __init__(self, orig_args=None):
        if orig_args is None:
            orig_args = sys.argv[1:]

        # Create a parser for common options to find the command name
        self.parser = argparse.ArgumentParser(
            prog="pisi",
            usage=pisi.cli.help.usage_text,
            add_help=False # We handle help manually or in commands
        )
        self.parser.add_argument('--version', action='store_true')
        self.parser.add_argument('-h', '--help', action='store_true')
        
        # We use parse_known_args to find the command and ignore other options for now
        args, unknown = self.parser.parse_known_args(orig_args)
        
        if args.version:
            print("pisi " + pisi.__version__)
            sys.exit(0)
            
        if not unknown and args.help:
            self.die()
            
        if not unknown:
            raise pisi.cli.Error(_('No command given'))
            
        cmd_name = unknown[0]
        # If the first unknown starts with '-', it might be a misplaced option
        if cmd_name.startswith('-'):
            if args.help:
                self.die()
            raise pisi.cli.Error(_('No command given or options precede command'))

        self.command = command.Command.get_command(cmd_name, args=orig_args)
        if not self.command:
            raise pisi.cli.Error(_("Unrecognized command: %s") % cmd_name)

    def die(self):
        pisi.cli.printu('\n' + pisi.cli.help.usage_text)
        print("\n" + _("Commands:") + "\n")
        print(command.Command.commands_string())
        sys.exit(1)

    def run_command(self):
        self.command.run()


def main():
    import errno
    import traceback
    import signal
    import pisi.context as ctx

    def sig_handler(sig, frame):
        sys.exit(1)

    def handle_exception(exception, value, tb):
        signal.signal(signal.SIGINT, signal.SIG_IGN)
        ui = pisi.cli.CLI()
        show_traceback = False

        if exception == KeyboardInterrupt:
            ui.error(_("Keyboard Interrupt: Exiting..."))
            sys.exit(1)
        elif isinstance(value, pisi.Error):
            ui.error(_("Program terminated."))
        elif isinstance(value, pisi.Exception):
            show_traceback = True
            ui.error(_("Unhandled internal exception.\n"
                       "Please file a bug report to <http://bugs.pardus.org.tr>."))
        elif isinstance(value, IOError) and value.errno == errno.EPIPE:
            sys.exit(0)
        else:
            show_traceback = ctx.get_option('debug')
            ui.error(_("System error. Program terminated."))

        if show_traceback:
            ui.error("%s: %s" % (exception, str(value)))
        else:
            msg = str(value)
            if msg:
                ui.error(msg)

        ui.info(_("Please use 'pisi help' for general help."))

        if show_traceback:
            ui.info(_("\nTraceback:"))
            traceback.print_tb(tb)
        elif not isinstance(value, pisi.Error):
            ui.info(_("Use --debug to see a traceback."))

        sys.exit(1)

    sys.excepthook = handle_exception
    signal.signal(signal.SIGINT, sig_handler)
    signal.signal(signal.SIGTERM, sig_handler)

    try:
        cli = PisiCLI()
        cli.run_command()
    except pisi.cli.Error as e:
        ui = pisi.cli.CLI()
        ui.error(str(e))
        sys.exit(1)
    except KeyboardInterrupt:
        ui = pisi.cli.CLI()
        ui.error(_("Keyboard Interrupt: Exiting..."))
        sys.exit(1)
    except Exception as e:
        handle_exception(type(e), e, sys.exc_info()[2])
