# -*- coding: utf-8 -*-

# Copyright (C) 2005 - 2011, TUBITAK/UEKAE
#
# This program is free software; you can redistribute it and/or modify it under
# the terms of the GNU General Public License as published by the Free
# Software Foundation; either version 2 of the License, or (at your option)
# any later version.
#
# Please read the COPYING file.
#

"""Yet another Pisi module for fetching files from various sources. Of
course, this is not limited to just fetching source files. We fetch
all kinds of things: source tarballs, index files, packages, and God
knows what."""

# python standard library modules
import os
import time
import base64
import shutil
import requests
import urllib.request
import urllib.parse
import urllib.error

import gettext
__trans = gettext.translation('pisi', fallback=True)
_ = __trans.gettext

# pisi modules
import pisi
import pisi.util as util
import pisi.context as ctx
import pisi.uri


class FetchError(pisi.Error):
    pass


class UIHandler:
    def __init__(self, progress):
        self.filename        = None
        self.url             = None
        self.basename        = None
        self.downloaded_size = 0
        self.percent         = None
        self.rate            = 0.0
        self.size            = 0
        self.eta             = '--:--:--'
        self.symbol          = '--/-'
        self.last_updated    = 0
        self.exist_size      = 0

    def start(self, archive, url, basename, size, text):
        if os.path.exists(archive):
            self.exist_size = os.path.getsize(archive)
        self.filename   = util.remove_suffix(ctx.const.partial_suffix, basename)
        self.url        = url
        self.basename   = basename
        self.total_size = size or 0
        self.text       = text

        self.now    = lambda: time.time()
        self.t_diff = lambda: self.now() - self.s_time

        self.s_time = self.now()

    def update(self, size):

        if self.size == size:
            return

        self.size = size
        if self.total_size:
            self.percent = (size * 100.0) / self.total_size
        else:
            self.percent = 0

        if int(self.now()) != int(self.last_updated) and size > 0:
            try:
                self.rate, self.symbol = util.human_readable_rate((size - self.exist_size) / (self.now() - self.s_time))
            except ZeroDivisionError:
                return
            if self.total_size:
                self.eta  = '%02d:%02d:%02d' %\
                    tuple([i for i in time.gmtime((self.t_diff() * (100 - self.percent)) / self.percent)[3:6]])

        self._update_ui()

    def end(self, read):
        pass

    def _update_ui(self):
        ctx.ui.display_progress(operation       = "fetching",
                                percent         = self.percent,
                                filename        = self.filename,
                                total_size      = self.total_size or self.size,
                                downloaded_size = self.size,
                                rate            = self.rate,
                                eta             = self.eta,
                                symbol          = self.symbol)

        self.last_updated = self.now()


class Fetcher:
    """Fetcher can fetch a file from various sources using various
    protocols."""
    def __init__(self, url, destdir="/tmp", destfile=None):
        if not isinstance(url, pisi.uri.URI):
            url = pisi.uri.URI(url)

        if ctx.config.get_option("authinfo"):
            url.set_auth_info(ctx.config.get_option("authinfo"))

        self.url = url
        self.destdir = destdir
        self.destfile = destfile
        self.progress = None

        self.archive_file = os.path.join(destdir, destfile or url.filename())
        self.partial_file = os.path.join(self.destdir, self.url.filename()) + ctx.const.partial_suffix

        util.ensure_dirs(self.destdir)

    def test(self, timeout=3):
        try:
            if self.url.scheme() in ["ftp", "file"]:
                with urllib.request.urlopen(self.url.get_uri(), timeout=timeout) as response:
                    return True

            response = requests.head(self.url.get_uri(),
                                   proxies=self._get_proxies(),
                                   timeout=timeout,
                                   headers={'User-Agent': 'PiSi Fetcher/' + pisi.__version__})
            response.raise_for_status()
        except requests.RequestException:
            # Fallback to GET for servers that don't support HEAD
            try:
                response = requests.get(self.url.get_uri(),
                                       proxies=self._get_proxies(),
                                       timeout=timeout,
                                       headers={'User-Agent': 'PiSi Fetcher/' + pisi.__version__},
                                       stream=True)
                response.raise_for_status()
                response.close()
            except requests.RequestException:
                return False

        return True

    def fetch(self):
        """Return value: Fetched file's full path.."""

        if not self.url.filename():
            raise FetchError(_('Filename error'))

        if not os.access(self.destdir, os.W_OK):
            raise FetchError(_('Access denied to write to destination directory: "%s"') % (self.destdir))

        if os.path.exists(self.archive_file) and not os.access(self.archive_file, os.W_OK):
            raise FetchError(_('Access denied to destination file: "%s"') % (self.archive_file))

        try:
            if self.url.scheme() in ["ftp", "file"]:
                # requests doesn't support FTP or file, use urllib
                with urllib.request.urlopen(self.url.get_uri()) as response:
                    try:
                        total_size = int(response.headers.get('content-length', 0))
                    except (TypeError, ValueError):
                        total_size = 0
                    ui = UIHandler(self.progress)
                    ui.start(self.partial_file, self.url.get_uri(), os.path.basename(self.partial_file), total_size, "")

                    with open(self.partial_file, 'wb') as f:
                        downloaded = 0
                        while True:
                            chunk = response.read(8192)
                            if not chunk:
                                break
                            f.write(chunk)
                            downloaded += len(chunk)
                            ui.update(downloaded)
            else:
                headers = {'User-Agent': 'PiSi Fetcher/' + pisi.__version__}
                resume_header = self._get_resume_header()
                if resume_header:
                    headers.update(resume_header)

                auth = self.url.auth_info()

                with requests.get(self.url.get_uri(),
                                 headers=headers,
                                 proxies=self._get_proxies(),
                                 auth=auth,
                                 stream=True) as r:
                    r.raise_for_status()

                    total_size = int(r.headers.get('content-length', 0))
                    if resume_header and r.status_code == 206:
                        total_size += os.path.getsize(self.partial_file)

                    ui = UIHandler(self.progress)
                    ui.start(self.partial_file, self.url.get_uri(), os.path.basename(self.partial_file), total_size, "")

                    mode = 'ab' if (resume_header and r.status_code == 206) else 'wb'
                    if mode == 'wb' and os.path.exists(self.partial_file):
                        os.remove(self.partial_file)

                    with open(self.partial_file, mode) as f:
                        downloaded = os.path.getsize(self.partial_file) if mode == 'ab' else 0
                        bandwidth_limit = self._get_bandwith_limit()

                        for chunk in r.iter_content(chunk_size=8192):
                            if chunk:
                                f.write(chunk)
                                downloaded += len(chunk)
                                ui.update(downloaded)

                                if bandwidth_limit:
                                    # Simple rate limiting
                                    time.sleep(len(chunk) / bandwidth_limit)

        except (requests.RequestException, urllib.error.URLError) as e:
            raise FetchError(_('Could not fetch destination file "%s": %s') % (self.url.get_uri(), e))

        if os.stat(self.partial_file).st_size == 0:
            os.remove(self.partial_file)
            raise FetchError(_('A problem occurred. Please check the archive address and/or permissions again.'))

        shutil.move(self.partial_file, self.archive_file)

        return self.archive_file

    def _get_proxies(self):
        proxies = {}
        
        if ctx.config.values.general.http_proxy:
            proxies['http'] = ctx.config.values.general.http_proxy

        if ctx.config.values.general.https_proxy:
            proxies['https'] = ctx.config.values.general.https_proxy

        if ctx.config.values.general.ftp_proxy:
            proxies['ftp'] = ctx.config.values.general.ftp_proxy

        return proxies

    def _get_bandwith_limit(self):
        bandwidth_limit = ctx.config.options.bandwidth_limit or ctx.config.values.general.bandwidth_limit
        if bandwidth_limit and bandwidth_limit != "0":
            return 1024 * int(bandwidth_limit)
        else:
            return 0

    def _get_resume_header(self):
        if os.path.exists(self.partial_file):
            size = os.path.getsize(self.partial_file)
            return {'Range': 'bytes=%d-' % size}
        return None

    def _get_http_headers(self):
        auth = self.url.auth_info()
        if auth:
            import base64
            auth_str = '%s:%s' % auth
            enc = base64.encodebytes(auth_str.encode('utf-8')).decode('utf-8')
            return (('Authorization', 'Basic %s' % enc),)
        return ()

    def _get_ftp_headers(self):
        return ()


# helper function
def fetch_url(url, destdir, progress=None, destfile=None):
    fetch = Fetcher(url, destdir, destfile)
    fetch.progress = progress
    fetch.fetch()
