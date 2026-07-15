# -*- coding: utf-8 -*-
#
# Copyright (C) 2005-2011, TUBITAK/UEKAE
#
# This program is free software; you can redistribute it and/or modify it under
# the terms of the GNU General Public License as published by the Free
# Software Foundation; either version 2 of the License, or (at your option)
# any later version.
#
# Please read the COPYING file.
#

"""misc. utility functions, including process and file utils"""

# standard python modules

import os
import sys
import fcntl
import shutil
import string
import struct
import fnmatch
import hashlib
import termios
import operator
import subprocess
import unicodedata
from functools import reduce

import gettext
__trans = gettext.translation('pisi', fallback=True)
_ = __trans.gettext

class Singleton(type):
    def __init__(cls, name, bases, dict):
        super(Singleton, cls).__init__(name, bases, dict)
        cls.instance = None

    def __call__(cls, *args, **kw):
        if cls.instance is None:
            cls.instance = super(Singleton, cls).__call__(*args, **kw)

        return cls.instance

# pisi modules
import pisi
import pisi.context as ctx

class Error(pisi.Error):
    pass

class FileError(Error):
    pass

class FilePermissionDeniedError(Error):
    pass


#########################
# string/list/functional#
#########################

def any_pred(pred, seq):
    return any(map(pred, seq))

def flatten_list(l):
    """Flatten a list of lists."""
    return [item for sublist in l for item in sublist]

def strlist(l):
    """Concatenate string reps of l's elements."""
    return "".join(map(lambda x: str(x) + ' ', l))

def prefix(a, b):
    """Check if sequence a is a prefix of sequence b."""
    if len(a) > len(b):
        return False
    for i in range(0, len(a)):
        if a[i] != b[i]:
            return False
    return True

def remove_prefix(a, b):
    """Remove prefix a from sequence b."""
    assert prefix(a, b)
    return b[len(a):]

def suffix(a, b):
    """Check if sequence a is a suffix of sequence b."""
    if len(a) > len(b):
        return False
    for i in range(1, len(a) + 1):
        if a[-i] != b[-i]:
            return False
    return True

def remove_suffix(a, b):
    """Remove suffix a from sequence b."""
    assert suffix(a, b)
    return b[:-len(a)]

def human_readable_size(size = 0):
    symbols, depth = [' B', 'KB', 'MB', 'GB', 'TB', 'PB', 'EB', 'ZB', 'YB'], 0

    while size > 1000 and depth < 8:
        size = float(size / 1024)
        depth += 1

    return size, symbols[depth]

def human_readable_rate(size = 0):
    x = human_readable_size(size)
    return x[0], x[1] + '/s'

def format_by_columns(strings, sep_width=2):
    longest_str_len = len(max(strings, key=len)) if strings else 0
    term_rows, term_columns = get_terminal_size()

    def get_columns(max_count):
        if longest_str_len > term_columns:
            return [longest_str_len]

        columns = []
        for name in strings:
            table_width = sum(columns) + len(name) + len(columns) * sep_width
            if table_width > term_columns:
                break

            columns.append(len(name))
            if len(columns) == max_count:
                break

        return columns

    def check_size(columns):
        if not columns: return True
        total_sep_width = (len(columns) - 1) * sep_width

        for n, name in enumerate(strings):
            col = n % len(columns)
            if len(name) > columns[col]:
                columns[col] = len(name)

            if len(columns) > 1:
                width = sum(columns) + total_sep_width
                if width > term_columns:
                    return False

        return True

    columns = get_columns(term_columns)

    while columns and not check_size(columns):
        columns = get_columns(len(columns) - 1)

    if not columns:
        return "\n".join(strings)

    sep = " " * sep_width
    lines = []
    current_line = []
    for n, name in enumerate(strings):
        col = n % len(columns)
        current_line.append(name.ljust(columns[col]))

        if col == len(columns) - 1:
            lines.append(sep.join(current_line))
            current_line = []

    if current_line:
        lines.append(sep.join(current_line))

    return "\n".join(lines)

##############################
# Process Releated Functions #
##############################

def search_executable(executable):
    """Search for the executable in user's paths and return it."""
    for _path in os.environ.get("PATH", "").split(":"):
        full_path = os.path.join(_path, executable)
        if os.path.exists(full_path) and os.access(full_path, os.X_OK):
            return full_path
    return None

def run_batch(cmd):
    """Run command and report return value and output."""
    ctx.ui.info(_('Running ') + cmd, verbose=True)
    p = subprocess.Popen(cmd, shell=True,
                         stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    out, err = p.communicate()
    if isinstance(out, bytes):
        out = out.decode('utf-8', 'replace')
    if isinstance(err, bytes):
        err = err.decode('utf-8', 'replace')
    ctx.ui.debug(_('return value for "%s" is %s') % (cmd, p.returncode))
    return (p.returncode, out, err)

def run_logged(cmd):
    """Run command and get the return value."""
    ctx.ui.info(_('Running ') + cmd, verbose=True)
    if ctx.stdout:
        stdout = ctx.stdout
    else:
        if ctx.get_option('debug'):
            stdout = None
        else:
            stdout = subprocess.PIPE
    if ctx.stderr:
        stderr = ctx.stderr
    else:
        if ctx.get_option('debug'):
            stderr = None
        else:
            stderr = subprocess.STDOUT

    p = subprocess.Popen(cmd, shell=True, stdout=stdout, stderr=stderr)
    out, err = p.communicate()
    ctx.ui.debug(_('return value for "%s" is %s') % (cmd, p.returncode))

    return p.returncode

######################
# Terminal functions #
######################

def get_terminal_size():
    try:
        import termios, fcntl, struct, sys
        ret = fcntl.ioctl(sys.stdout.fileno(), termios.TIOCGWINSZ, struct.pack("hhhh", 0, 0, 0, 0))
        rows, cols, x, y = struct.unpack("hhhh", ret)
        return rows, cols
    except Exception:
        rows = int(os.environ.get("LINES", 25))
        cols = int(os.environ.get("COLUMNS", 80))
        return rows, cols

def xterm_title(message):
    """Set message as console window title."""
    if "TERM" in os.environ and sys.stderr.isatty():
        terminalType = os.environ["TERM"]
        for term in ["xterm", "Eterm", "aterm", "rxvt", "screen", "kterm", "rxvt-str"]:
            if terminalType.startswith(term):
                sys.stderr.write("\x1b]2;"+str(message)+"\x07")
                sys.stderr.flush()
                break

def xterm_title_reset():
    """Reset console window title."""
    if "TERM" in os.environ:
        xterm_title("")

#############################
# Path Processing Functions #
#############################

def splitpath(a):
    """split path into components and return as a list"""
    comps = a.split(os.path.sep)
    if comps and comps[-1] == '':
        comps.pop()
    return comps

def makepath(comps, relative = False, sep = os.path.sep):
    """Reconstruct a path from components."""
    if not comps:
        return ""
    path = reduce(lambda x,y: x + sep + y, comps, '')
    if relative:
        return path[len(sep):]
    else:
        return path

def parentpath(a, sep = os.path.sep):
    # remove trailing '/'
    a = a.rstrip(sep)
    return a[:a.rfind(sep)]

def parenturi(a):
    return parentpath(a, '/')

def subpath(a, b):
    """Find if path a is before b in the directory tree."""
    return prefix(splitpath(a), splitpath(b))

def removepathprefix(prefix_path, path):
    """Remove path prefix a from b, finding the pathname rooted at a."""
    comps = remove_prefix(splitpath(prefix_path), splitpath(path))
    if len(comps) > 0:
        return join_path(*tuple(comps))
    else:
        return ""

def join_path(a, *p):
    """Join two or more pathname components."""
    path = a
    for b in p:
        b = b.lstrip('/')
        if path == '' or path.endswith('/'):
            path +=  b
        else:
            path += '/' + b
    return path

####################################
# File/Directory Related Functions #
####################################

def check_file(_file, mode = os.F_OK):
    """Shorthand to check if a file exists."""
    if not os.access(_file, mode):
        raise FileError("File " + _file + " not found")
    return True

def ensure_dirs(path):
    """Make sure the given directory path exists."""
    if not os.path.exists(path):
        os.makedirs(path)

def clean_dir(path):
    """Remove all content of a directory."""
    if os.path.exists(path):
        shutil.rmtree(path)

def creation_time(_file):
    """Return the creation time of the given file."""
    if check_file(_file):
        import time
        st = os.stat(_file)
        return time.localtime(st.st_ctime)

def dir_size(_dir):
    """Calculate the size of files under a directory."""
    if os.path.exists(_dir) and (not os.path.isdir(_dir) and not os.path.islink(_dir)):
        return os.path.getsize(_dir)

    if os.path.islink(_dir):
        return int(len(read_link(_dir)))

    def sizes():
        for root, dirs, files in os.walk(_dir):
            yield sum([os.path.getsize(join_path(root, name)) for name in files if not os.path.islink(join_path(root, name))])
    return sum(sizes())

def copy_file(src,dest):
    """Copy source file to the destination file."""
    check_file(src)
    ensure_dirs(os.path.dirname(dest))
    shutil.copyfile(src, dest)

def copy_file_stat(src,dest):
    """Copy source file to the destination file with all stat info."""
    check_file(src)
    ensure_dirs(os.path.dirname(dest))
    shutil.copy2(src, dest)

def read_link(link):
    """Return the normalized path which is pointed by the symbolic link."""
    return os.path.normpath(os.readlink(link))

def is_ar_file(file_path):
    with open(file_path, 'rb') as f:
        return f.read(8) == b'!<arch>\n'

def clean_ar_timestamps(ar_file):
    """Zero all timestamps in the ar files."""
    if not is_ar_file(ar_file):
        return
    with open(ar_file, 'rb') as f:
        content = f.readlines()
    with open(ar_file, 'wb') as fp:
        for line in content:
            pos = line.rfind(b' ' + b'`')
            if pos > -1 and line[pos - 57:pos + 2].find(b'/') > -1:
                 line = line[:pos - 41] + b'0000000000' + line[pos - 31:]
            fp.write(line)

def calculate_hash(path):
    """Return a (path, hash) tuple for given path."""
    if os.path.islink(path):
        # For symlinks, path string is hashed instead of the content
        value = sha1_data(read_link(path).encode('utf-8'))
        if not os.path.exists(path):
            ctx.ui.info(_("Including external link '%s'") % path)
    elif os.path.isdir(path):
        ctx.ui.info(_("Including directory '%s'") % path)
        value = None
    else:
        if path.endswith('.a'):
            # .a file content changes with each compile due to timestamps
            # We pad them with zeroes, thus hash will be stable
            clean_ar_timestamps(path)
        value = sha1_file(path)

    return (path, value)

def get_file_hashes(top, excludePrefix=None, removePrefix=None):
    """Yield (path, hash) tuples for given directory tree."""
    def is_included(path):
        if excludePrefix:
            temp = remove_prefix(removePrefix, path)
            while temp != "/":
                if any(fnmatch.fnmatch(temp, x) for x in excludePrefix):
                    return False
                temp = os.path.dirname(temp)
        return True

    # single file/symlink case
    if not os.path.isdir(top) or os.path.islink(top):
        if is_included(top):
            yield calculate_hash(top)
        return

    for root, dirs, files in os.walk(top):
        for name in files:
            path = os.path.join(root, name)
            if is_included(path):
                yield calculate_hash(path)

        for name in dirs:
            path = os.path.join(root, name)
            if os.path.islink(path):
                if is_included(path):
                    yield calculate_hash(path)

        if len(files) == 0 and len(dirs) == 0:
            if is_included(root):
                yield calculate_hash(root)

def check_file_hash(filename, hash_val):
    """Check the file's integrity with a given hash."""
    return sha1_file(filename) == hash_val

def sha1_file(filename):
    """Calculate sha1 hash of file."""
    try:
        m = hashlib.sha1()
        with open(filename, 'rb') as f:
            while True:
                block = f.read(256 * 1024)
                if len(block) == 0:
                    break
                m.update(block)
                del block
        return m.hexdigest()
    except IOError as e:
        if e.errno == 13:
            raise FilePermissionDeniedError(_("You don't have necessary read permissions"))
        else:
            raise FileError(_("Cannot calculate SHA1 hash of %s") % filename)

def sha1_data(data):
    """Calculate sha1 hash of given data."""
    if isinstance(data, str):
        data = data.encode('utf-8')
    m = hashlib.sha1()
    m.update(data)
    return m.hexdigest()

def uncompress(patchFile, compressType="gz", targetDir=""):
    """Uncompress the file and return the new path."""
    formats = ("gz", "gzip", "bz2", "bzip2", "lzma", "xz")
    if compressType not in formats:
        raise Error(_("Compression type is not valid: '%s'") % compressType)

    archive = pisi.archive.Archive(patchFile, compressType)
    try:
        archive.unpack(targetDir)
    except Exception as msg:
        raise Error(_("Error while decompressing %s: %s") % (patchFile, msg))

    filePath = join_path(targetDir, os.path.basename(patchFile))

    extensions = {"gzip": "gz", "bzip2": "bz2"}
    extension = extensions.get(compressType, compressType)
    return filePath.split(".%s" % extension)[0]


def do_patch(sourceDir, patchFile, level=0, name=None, reverse=False):
    """Apply given patch to the sourceDir."""
    cwd = os.getcwd()
    if os.path.exists(sourceDir):
        os.chdir(sourceDir)
    else:
        raise Error(_("ERROR: WorkDir (%s) does not exist\n") % (sourceDir))

    if level is None:
        level = 0

    if name is None:
        name = os.path.basename(patchFile)

    check_file(patchFile)

    if ctx.get_option('use_quilt'):
        patchesDir = join_path(sourceDir, ctx.const.quilt_dir_suffix)
        if not os.path.exists(patchesDir):
            os.makedirs(patchesDir)
        (ret, out, err) = run_batch('quilt import %s -p %d -P %s \"%s\"' % (("-R" if reverse else ""), level, name, patchFile))
        (ret, out, err) = run_batch('quilt push')
    else:
        (ret, out, err) = run_batch("patch --remove-empty-files --no-backup-if-mismatch %s -p%d -i \"%s\"" % (("-R" if reverse else ""), level, patchFile))

    if ret:
        if out is None and err is None:
            raise Error(_("ERROR: patch (%s) failed") % (patchFile))
        else:
            raise Error(_("ERROR: patch (%s) failed: %s") % (patchFile, out))

    os.chdir(cwd)

def strip_file(filepath, fileinfo, outpath):
    """Strip an elf file from debug symbols."""
    def run_strip(f, flags=""):
        cmd = ["strip"] + flags.split() + [f]
        res = subprocess.run(cmd, capture_output=True, text=True)
        if res.returncode:
            ctx.ui.warning(_("strip command failed for file '%s'!") % f)

    def run_chrpath(f):
        res = subprocess.run(["chrpath", "-d", f], capture_output=True, text=True)
        if res.returncode:
            ctx.ui.warning(_("chrpath command failed for file '%s'!") % f)

    def save_elf_debug(f, o):
        debug_file = o + ctx.const.debug_file_suffix
        res = subprocess.run(["objcopy", "--only-keep-debug", f, debug_file], capture_output=True, text=True)
        if res.returncode:
            ctx.ui.warning(_("objcopy (keep-debug) command failed for file '%s'!") % f)

        res = subprocess.run(["objcopy", "--add-gnu-debuglink=" + debug_file, f], capture_output=True, text=True)
        if res.returncode:
            ctx.ui.warning(_("objcopy (add-debuglink) command failed for file '%s'!") % f)

    if "current ar archive" in fileinfo:
        run_strip(filepath, "--strip-debug")
        return True

    elif "SB executable" in fileinfo:
        if ctx.config.values.build.generatedebug:
            ensure_dirs(os.path.dirname(outpath))
            save_elf_debug(filepath, outpath)
        run_strip(filepath)
        return True

    elif "SB shared object" in fileinfo:
        if ctx.config.values.build.generatedebug:
            ensure_dirs(os.path.dirname(outpath))
            save_elf_debug(filepath, outpath)
        run_strip(filepath, "--strip-unneeded")
        return True

    return False

def partition_freespace(directory):
    """Return free space of given directory's partition."""
    st = os.statvfs(directory)
    return st.f_bsize * st.f_bfree

########################################
# Package/Repository Related Functions #
########################################

def package_filename(name, version, release, distro_id=None, arch=None):
    """Return a filename for a package with the given information. """

    if distro_id is None:
        distro_id = ctx.config.values.general.distribution_id

    if arch is None:
        arch = ctx.config.values.general.architecture

    fn = "-".join((name, version, release, distro_id, arch))
    fn += ctx.const.package_suffix

    return fn

def parse_package_name_legacy(package_name):
    """Separate package name and version string for package formats <= 1.1."""
    name = []
    for part in package_name.split("-"):
        if name != [] and part[0] in string.digits:
            break
        else:
            name.append(part)
    name = "-".join(name)
    version = package_name[len(name) + 1:]

    return (name, version)

def parse_package_name(package_name):
    """Separate package name and version string."""

    if package_name.endswith(ctx.const.package_suffix):
        package_name = remove_suffix(ctx.const.package_suffix, package_name)

    try:
        name, version, release, distro_id, arch = package_name.rsplit("-", 4)
        if not arch or arch[0] in string.digits:
            raise ValueError

    except ValueError:
        try:
            return parse_package_name_legacy(package_name)
        except Exception:
            raise Error(_("Invalid package name: %s") % package_name)

    return name, "%s-%s" % (version, release)

def parse_delta_package_name_legacy(package_name):
    """Separate delta package name and release infos for package formats <= 1.1."""
    name, build = parse_package_name(package_name)
    build = build[:-len(ctx.const.delta_package_suffix)]
    buildFrom, buildTo = build.split("-")

    return name, buildFrom, buildTo

def parse_delta_package_name(package_name):
    """Separate delta package name and release infos"""

    if package_name.endswith(ctx.const.delta_package_suffix):
        package_name = remove_suffix(ctx.const.delta_package_suffix,
                                     package_name)

    try:
        name, source_release, target_release, distro_id, arch = \
                package_name.rsplit("-", 4)
        if not arch or arch[0] in string.digits:
            raise ValueError

    except ValueError:
        try:
            return parse_delta_package_name_legacy(package_name)
        except Exception:
            raise Error(_("Invalid delta package name: %s") % package_name)

    return name, source_release, target_release

def split_package_filename(filename):
    """Split fields in package filename."""

    if filename.endswith(ctx.const.package_suffix):
        filename = remove_suffix(ctx.const.package_suffix, filename)

    try:
        name, version, release, distro_id, arch = filename.rsplit("-", 4)
        if not arch or arch[0] in string.digits:
            raise ValueError

    except ValueError:
        name, version = parse_package_name_legacy(filename)
        version, release, build = split_version(version)
        distro_id = arch = None

    return name, version, release, distro_id, arch

def split_delta_package_filename(filename):
    """Split fields in delta package filename."""

    if filename.endswith(ctx.const.delta_package_suffix):
        filename = remove_suffix(ctx.const.delta_package_suffix, filename)

    try:
        name, source_release, target_release, distro_id, arch = \
                filename.rsplit("-", 4)
        if not arch or arch[0] in string.digits:
            raise ValueError

    except ValueError:
        name = parse_delta_package_name_legacy(filename)[0]
        source_release = target_release = None

    return name, source_release, target_release, distro_id, arch

def split_version(package_version):
    """Split version, release and build parts of a package version"""
    version, sep, release_and_build = package_version.partition("-")
    release, sep, build = release_and_build.partition("-")
    return version, release, build

def filter_latest_packages(package_paths):
    """ For a given pisi package paths list where there may also be multiple versions
        of the same package, filters only the latest versioned ones """

    import pisi.version

    latest = {}
    for path in package_paths:

        name, version = parse_package_name(os.path.basename(path[:-len(ctx.const.package_suffix)]))

        if name in latest:
            l_version, l_release, l_build = split_version(latest[name][1])
            r_version, r_release, r_build = split_version(version)

            try:
                l_release = int(l_release)
                r_release = int(r_release)

                l_build = int(l_build) if l_build else None
                r_build = int(r_build) if r_build else None

            except ValueError:
                continue

            if l_build and r_build:
                if l_build > r_build:
                    continue

            elif l_release > r_release:
                continue

            elif l_release == r_release:
                l_version = pisi.version.make_version(l_version)
                r_version = pisi.version.make_version(r_version)

                if l_version > r_version:
                    continue

        if version:
            latest[name] = (path, version)

    return [x[0] for x in latest.values()]

def colorize(msg, color):
    """Colorize the given message for console output"""
    if color in ctx.const.colors and not ctx.get_option('no_color'):
        return ctx.const.colors[color] + msg + ctx.const.colors['default']
    else:
        return msg

def config_changed(config_file):
    fpath = pisi.util.join_path(ctx.config.dest_dir(), config_file.path)
    if os.path.exists(fpath) and not os.path.isdir(fpath):
        if os.path.islink(fpath):
            f = os.readlink(fpath)
            if os.path.exists(f) and pisi.util.sha1_data(f) != config_file.hash:
                return True
        else:
            if pisi.util.sha1_file(fpath) != config_file.hash:
                return True
    return False

# recursively remove empty dirs starting from dirpath
def rmdirs(dirpath):
    if os.path.isdir(dirpath) and not os.listdir(dirpath):
        ctx.ui.debug("Removing empty dir: %s" % dirpath)
        os.rmdir(dirpath)
        rmdirs(os.path.dirname(dirpath))

def letters():
    result = []
    for index in range(sys.maxunicode + 1):
        c = chr(index)
        if unicodedata.category(c)[0] == 'L':
            result.append(c)
    return ''.join(result)
