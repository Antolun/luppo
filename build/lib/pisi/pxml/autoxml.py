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

"""
 autoxml is a metaclass for automatic XML translation, using
 a miniature type system. (w00t!) This is based on an excellent
 high-level XML processing prototype that Gurer prepared.

 Method names are mixedCase for compatibility with minidom,
 an old library.
"""

# System
import locale
import io
import sys
import inspect
import re

class DumbWriter:
    def __init__(self, file=None, maxcol=78):
        self.file = file or sys.stdout
        self.maxcol = maxcol
        self.col = 0
        self.atbreak = 0

    def send_flowing_data(self, data):
        if not data: return
        for word in data.split():
            if self.atbreak:
                if self.col + len(word) >= self.maxcol:
                    self.file.write('\n')
                    self.col = 0
                else:
                    self.file.write(' ')
                    self.col += 1
            self.file.write(word)
            self.col += len(word)
            self.atbreak = 1

    def send_literal_data(self, data):
        self.file.write(data)
        i = data.rfind('\n')
        if i >= 0:
            self.col = 0
            data = data[i+1:]
        data = data.expandtabs()
        self.col += len(data)
        self.atbreak = 0

    def send_line_break(self):
        self.file.write('\n')
        self.col = 0
        self.atbreak = 0

class AbstractFormatter:
    def __init__(self, writer):
        self.writer = writer
    def add_flowing_data(self, data):
        self.writer.send_flowing_data(data)
    def add_literal_data(self, data):
        self.writer.send_literal_data(data)
    def add_line_break(self):
        self.writer.send_line_break()

import gettext
__trans = gettext.translation('pisi', fallback=True)
_ = __trans.gettext

# PiSi
import pisi
import pisi.pxml.xmlext as xmlext
import pisi.pxml.xmlfile as xmlfile
import pisi.context as ctx
import pisi.util as util
import pisi.oo as oo

class Error(pisi.Error):
    pass

# requirement specs

mandatory, optional = range(2) # poor man's enum

# basic types
# Modern Python 3 types
String = str
Text = str
Integer = int
Long = int
Float = float

class LocalText(dict):
    """Handles XML tags with localized text"""

    def __init__(self, tag = "", req = optional):
        self.tag = tag
        self.req = req
        dict.__init__(self)

    def decode(self, node, errs, where = ""):
        # flags, tag name, instance attribute
        assert self.tag != ''
        nodes = xmlext.getAllNodes(node, self.tag)
        if not nodes:
            if self.req == mandatory:
                errs.append(where + ': ' + _("At least one '%s' tag should have local text") %
                                    self.tag )
        else:
            for node in nodes:
                lang = xmlext.getNodeAttribute(node, 'xml:lang')
                c = xmlext.getNodeText(node)
                if not c:
                    errs.append(where + ': ' + _("'%s' language of tag '%s' is empty") %
                                (lang, self.tag))
                # FIXME: check for dups and 'en'
                if not lang:
                    lang = 'en'
                self[lang] = c

    def encode(self, node, errs):
        assert self.tag != ''
        for key in self.keys():
            newnode = xmlext.addNode(node, self.tag)
            xmlext.setNodeAttribute(newnode, 'xml:lang', key)
            xmlext.addText(newnode, '',  self[key])

    #FIXME: maybe more appropriate for pisi.util
    @staticmethod
    def get_lang():
        try:
            (lang, encoding) = locale.getlocale()
            if not lang:
                (lang, encoding) = locale.getdefaultlocale()
            if lang is None: # stupid python means it is C locale
                return 'en'
            else:
                return lang[0:2]
        except KeyboardInterrupt:
            raise
        except Exception: #FIXME: what exception could we catch here, replace with that.
            raise Error(_('LocalText: unable to get either current or default locale'))

    def errors(self, where = str()):
        errs = []
        langs = [ LocalText.get_lang(), 'en', 'tr', ]
        if self.keys() and not any(x in self for x in langs):
                errs.append( where + ': ' + _("Tag should have at least the current locale, or failing that an English or Turkish version"))
        #FIXME: check if all entries are str
        return errs

    def format(self, f, errs):
        L = LocalText.get_lang()
        if L in self:
            f.add_flowing_data(self[L])
        elif 'en' in self:
            # fallback to English, blah
            f.add_flowing_data(self['en'])
        elif 'tr' in self:
            # fallback to Turkish
            f.add_flowing_data(self['tr'])
        else:
            errs.append(_("Tag should have at least the current locale, or failing that an English or Turkish version"))

    #FIXME: factor out these common routines
    def print_text(self, file = sys.stdout):
        w = Writer(file) # plain text
        f = AbstractFormatter(w)
        errs = []
        self.format(f, errs)
        if errs:
            for x in errs:
                ctx.ui.warning(x)

    def __str__(self):
        L = LocalText.get_lang()
        if L in self:
            return str(self[L])
        elif 'en' in self:
            # fallback to English, blah
            return str(self['en'])
        elif 'tr' in self:
            # fallback to Turkish
            return str(self['tr'])
        else:
            return str()

class Writer(DumbWriter):
    """adds str support"""

    def __init__(self, file=None, maxcol=78):
        super(Writer, self).__init__(file, maxcol)

    def send_literal_data(self, data):
        self.file.write(data)
        i = data.rfind('\n')
        if i >= 0:
            self.col = 0
            data = data[i+1:]
        data = data.expandtabs()
        self.col = self.col + len(data)
        self.atbreak = 0

class autoxml(oo.autosuper, oo.autoprop):
    """High-level automatic XML transformation interface for xmlfile."""

    def __init__(cls, name, bases, dict):
        """entry point for metaclass code"""
        # standard initialization
        super(autoxml, cls).__init__(name, bases, dict)

        xmlfile_support = xmlfile.XmlFile in bases

        cls.autoxml_bases = [base for base in bases if isinstance(base, autoxml)]

        # default class tag is class name
        if not 'tag' in dict:
            cls.tag = name

        # generate helper routines, for each XML component
        names = []
        inits = []
        decoders = []
        encoders = []
        errorss = []
        formatters = []

        # read declaration order from source
        try:
            fn = re.compile(r'\s*([tas]_[a-zA-Z]+).*').findall

            inspect.linecache.clearcache()
            lines = [line for line in inspect.getsourcelines(cls)[0] if fn(line)]
            decl_order = [line.split()[0] for line in lines]
        except (IOError, TypeError):
            decl_order = list(dict.keys())

        # find string member
        order = [x for x in decl_order if not x.startswith('s_')]
        str_members = [x for x in decl_order if x.startswith('s_')]
        
        if len(str_members) > 1:
            raise Error('Only one str member can be defined')
        elif len(str_members) == 1:
            order.insert(0, str_members[0])

        for var in order:
            if var.startswith('t_') or var.startswith('a_') or var.startswith('s_'):
                m_name = var[2:]
                if var.startswith('a_'):
                    x = autoxml.gen_attr_member(cls, m_name)
                elif var.startswith('t_'):
                    x = autoxml.gen_tag_member(cls, m_name)
                elif var.startswith('s_'):
                    x = autoxml.gen_str_member(cls, m_name)
                (m_name, init, decoder, encoder, errors, format_x) = x
                names.append(m_name)
                inits.append(init)
                decoders.append(decoder)
                encoders.append(encoder)
                errorss.append(errors)
                formatters.append(format_x)

        # generate top-level helper functions
        cls.initializers = inits
        def initialize(self, uri = None, keepDoc = False, tmpDir = '/tmp',
                       **args):
            if xmlfile_support:
                if 'tag' in args:
                    xmlfile.XmlFile.__init__(self, tag = args['tag'])
                else:
                    xmlfile.XmlFile.__init__(self, tag = cls.tag)
            for base in cls.autoxml_bases:
                base.__init__(self)
            for init in inits:
                init(self)
            for x in args.keys():
                setattr(self, x, args[x])
            # init hook
            if hasattr(self, 'init'):
                self.init(cls.tag)
            if xmlfile_support and uri:
                self.read(uri, keepDoc, tmpDir)

        cls.__init__ = initialize

        cls.decoders = decoders
        def decode(self, node, errs, where = str(cls.tag)):
            for base in cls.autoxml_bases:
                base.decode(self, node, errs, where)
            for decode_member in decoders:
                decode_member(self, node, errs, where)
            if hasattr(self, 'decode_hook'):
                self.decode_hook(node, errs, where)
        cls.decode = decode

        cls.encoders = encoders
        def encode(self, node, errs):
            for base in cls.autoxml_bases:
                base.encode(self, node, errs)
            for encode_member in encoders:
                encode_member(self, node, errs)
            if hasattr(self, 'encode_hook'):
                self.encode_hook(node, errs)
        cls.encode = encode

        cls.errorss = errorss
        def errors(self, where = str(name)):
            errs = []
            for base in cls.autoxml_bases:
                errs.extend(base.errors(self, where))
            for err_func in errorss:
                errs.extend(err_func(self, where))
            if hasattr(self, 'errors_hook'):
                errs.extend(self.errors_hook(where))
            return errs
        cls.errors = errors
        
        def check(self):
            errs = self.errors()
            if errs:
                errs.append(_("autoxml.check: '%s' errors") % len(errs))
                raise Error(*errs)
        cls.check = check

        cls.formatters = formatters
        def format(self, f, errs):
            for base in cls.autoxml_bases:
                base.format(self, f, errs)
            for format_func in formatters:
                format_func(self, f, errs)
        cls.format = format
        
        def print_text(self, file = sys.stdout):
            w = Writer(file) # plain text
            f = AbstractFormatter(w)
            errs = []
            self.format(f, errs)
            if errs:
                for x in errs:
                    ctx.ui.warning(x)
        cls.print_text = print_text
        
        if not '__str__' in dict:
            def __str__(self):
                strfile = io.StringIO()
                self.print_text(strfile)
                val = strfile.getvalue()
                strfile.close()
                return val
            cls.__str__ = __str__

        if not '__eq__' in dict:
            def equal(self, other):
                if other is None:
                    return False
                for n in names:
                    try:
                        if getattr(self, n) != getattr(other, n):
                            return False
                    except KeyboardInterrupt:
                        raise
                    except Exception:
                        return False
                return True
            def notequal(self, other):
                return not self.__eq__(other)
            cls.__eq__ = equal
            cls.__ne__ = notequal

        if xmlfile_support:
            def parse(self, xml, keepDoc = False):
                "parse XML string and decode it into a python object"
                self.parsexml(xml)
                errs = []
                self.decode(self.rootNode(), errs)
                if errs:
                    errs.append(_("autoxml.parse: String '%s' has errors") % xml)
                    raise Error(*errs)
                if hasattr(self, 'read_hook'):
                    self.read_hook(errs)

                if not keepDoc:
                    self.unlink() # get rid of the tree

                errs = self.errors()
                if errs:
                    errs.append(_("autoxml.parse: String '%s' has errors") % xml)

            def read(self, uri, keepDoc = False, tmpDir = '/tmp',
                     sha1sum = False, compress = None, sign = None, copylocal = False, nodecode = False):
                "read XML file and decode it into a python object"
                read_xml = self.readxml(uri, tmpDir, sha1sum=sha1sum, 
                             compress=compress, sign=sign, copylocal=copylocal)

                if nodecode:
                    return read_xml

                errs = []
                self.decode(self.rootNode(), errs)
                if errs:
                    errs.append(_("autoxml.read: File '%s' has errors") % uri)
                    raise Error(*errs)
                if hasattr(self, 'read_hook'):
                    self.read_hook(errs)

                if not keepDoc:
                    self.unlink() # get rid of the tree

                errs = self.errors()
                if errs:
                    errs.append(_("autoxml.read: File '%s' has errors") % uri)
                    raise Error(*errs)

            def write(self, uri, keepDoc = False, tmpDir = '/tmp',
                      sha1sum = False, compress = None, sign = None):
                "encode the contents of the python object into an XML file"
                errs = self.errors()
                if errs:
                    errs.append(_("autoxml.write: object validation has failed"))
                    raise Error(*errs)
                errs = []
                self.newDocument()
                self.encode(self.rootNode(), errs)
                if hasattr(self, 'write_hook'):
                    self.write_hook(errs)
                if errs:
                    errs.append(_("autoxml.write: File encoding '%s' has errors") % uri)
                    raise Error(*errs)
                self.writexml(uri, tmpDir, sha1sum=sha1sum, compress=compress, sign=sign)
                if not keepDoc:
                    self.unlink() # get rid of the tree

            cls.read = read
            cls.write = write
            cls.parse = parse

    def gen_attr_member(cls, attr):
        """generate readers and writers for an attribute member"""
        spec = getattr(cls, 'a_' + attr)
        def readtext(node, a):
            return xmlext.getNodeAttribute(node, a)
        def writetext(node, a, text):
            xmlext.setNodeAttribute(node, a, text)
        anonfuns = cls.gen_anon_basic(attr, spec, readtext, writetext)
        return cls.gen_named_comp(attr, spec, anonfuns)

    def gen_tag_member(cls, tag):
        """generate helper funs for tag member of class"""
        spec = getattr(cls, 't_' + tag)
        anonfuns = cls.gen_tag(tag, spec)
        return cls.gen_named_comp(tag, spec, anonfuns)

    def gen_tag(cls, tag, spec):
        """generate readers and writers for the tag"""
        tag_type = spec[0]
        if isinstance(tag_type, type) and tag_type in autoxml.basic_cons_map:
            def readtext(node, tagpath):
                return xmlext.getNodeText(node, tagpath)
            def writetext(node, tagpath, text):
                xmlext.addText(node, tagpath, text)
            return cls.gen_anon_basic(tag, spec, readtext, writetext)
        elif isinstance(tag_type, list):
            return cls.gen_list_tag(tag, spec)
        elif tag_type is LocalText:
            return cls.gen_insetclass_tag(tag, spec)
        elif isinstance(tag_type, (autoxml, type)):
            return cls.gen_class_tag(tag, spec)
        else:
            raise Error(_('gen_tag: unrecognized tag type %s in spec') %
                        str(tag_type))

    def gen_str_member(cls, token):
        """generate readers and writers for a string member"""
        spec = getattr(cls, 's_' + token)
        def readtext(node, _):
            return xmlext.getNodeText(node)
        def writetext(node, _, text):
            xmlext.addText(node, "", text)
        anonfuns = cls.gen_anon_basic(token, spec, readtext, writetext)
        return cls.gen_named_comp(token, spec, anonfuns)

    def gen_named_comp(cls, token, spec, anonfuns):
        """generate a named component tag/attr."""
        m_name = cls.mixed_case(token)
        req = spec[1]
        (init_a, decode_a, encode_a, errors_a, format_a) = anonfuns

        def init(self):
            setattr(self, m_name, init_a())

        def decode(self, node, errs, where):
            setattr(self, m_name, decode_a(node, errs, where + '.' + str(m_name)))

        def encode(self, node, errs):
            if hasattr(self, m_name):
                value = getattr(self, m_name)
            else:
                value = None
            encode_a(node, value, errs)

        def errors(self, where):
            errs = []
            if hasattr(self, m_name) and getattr(self, m_name) is not None:
                value = getattr(self,m_name)
                errs.extend(errors_a(value, where + '.' + m_name))
            else:
                if req == mandatory:
                    errs.append(where + ': ' + _('Mandatory variable %s not available') % m_name)
            return errs

        def format(self, f, errs):
            if hasattr(self, m_name):
                value = getattr(self,m_name)
                f.add_literal_data(token + ': ')
                format_a(value, f, errs)
                f.add_line_break()
            else:
                if req == mandatory:
                    errs.append(_('Mandatory variable %s not available') % m_name)

        return (m_name, init, decode, encode, errors, format)

    def mixed_case(cls, identifier):
        """helper function to turn token name into mixed case"""
        if identifier == "":
            return ""
        else:
            return identifier[0].lower() + identifier[1:]

    def gen_anon_basic(cls, token, spec, readtext, writetext):
        """Generate a tag or attribute with one of the basic types."""
        m_name, token_type, req, tagpath = cls.parse_spec(token, spec)

        def initialize():
            return None

        def decode(node, errs, where):
            text = readtext(node, token)
            if text:
                try:
                    value = autoxml.basic_cons_map[token_type](text)
                except KeyboardInterrupt:
                    raise
                except Exception:
                    value = None
                    errs.append(where + ': ' + _('Type mismatch: read text cannot be decoded'))
                return value
            else:
                if req == mandatory:
                    errs.append(where + ': ' + _('Mandatory token %s not available') % token)
                return None

        def encode(node, value, errs):
            if value is not None:
                writetext(node, token, str(value))
            else:
                if req == mandatory:
                    errs.append(_('Mandatory token %s not available') % token)

        def errors(value, where):
            errs = []
            if value and not isinstance(value, token_type):
                errs.append(where + ': ' + _('Type mismatch. Expected %s, got %s') %
                                    (token_type, type(value)) )
            return errs

        def format(value, f, errs):
            f.add_literal_data(str(value))

        return initialize, decode, encode, errors, format

    def gen_class_tag(cls, tag, spec):
        """generate a class datatype"""
        m_name, tag_type, req, path = cls.parse_spec(tag, spec)

        def make_object():
            obj = tag_type.__new__(tag_type)
            obj.__init__(tag=tag, req=req)
            return obj

        def init():
            return make_object()

        def decode(node, errs, where):
            node = xmlext.getNode(node, tag)
            if node:
                try:
                    obj = make_object()
                    obj.decode(node, errs, where)
                    return obj
                except Error:
                    errs.append(where + ': '+ _('Type mismatch: DOM cannot be decoded'))
            else:
                if req == mandatory:
                    errs.append(where + ': ' + _('Mandatory argument not available'))
            return None

        def encode(node, obj, errs):
            if node and obj:
                try:
                    classnode = xmlext.newNode(node, tag)
                    obj.encode(classnode, errs)
                    xmlext.addNode(node, '', classnode)
                except Error:
                    if req == mandatory:
                        errs.append(_('Object cannot be encoded'))
            else:
                if req == mandatory:
                    errs.append(_('Mandatory argument not available'))

        def errors(obj, where):
            return obj.errors(where)

        def format(obj, f, errs):
            if obj:
                try:
                    obj.format(f, errs)
                except Error:
                    if req == mandatory:
                        errs.append(_('Object cannot be formatted'))
            else:
                if req == mandatory:
                    errs.append(_('Mandatory argument not available'))

        return (init, decode, encode, errors, format)

    def gen_list_tag(cls, tag, spec):
        """generate a list datatype."""
        m_name, tag_type, req, path = cls.parse_spec(tag, spec)

        pathcomps = path.split('/')
        comp_tag = pathcomps.pop()
        list_tagpath = util.makepath(pathcomps, sep='/', relative=True)

        if len(tag_type) != 1:
            raise Error(_('List type must contain only one element'))

        x = cls.gen_tag(comp_tag, [tag_type[0], mandatory])
        (init_item, decode_item, encode_item, errors_item, format_item) = x

        def init():
            return []

        def decode(node, errs, where):
            l = []
            nodes = xmlext.getAllNodes(node, path)
            if len(nodes) == 0 and req == mandatory:
                errs.append(where + ': ' + _('Mandatory list "%s" under "%s" node is empty.') % (path, node.name()))
            ix = 1
            for n in nodes:
                dummy = xmlext.newNode(n, "Dummy")
                xmlext.addNode(dummy, '', n)
                l.append(decode_item(dummy, errs, where + str("[%s]" % ix)))
                ix += 1
            return l

        def encode(node, l, errs):
            if l:
                for item in l:
                    if list_tagpath:
                        listnode = xmlext.addNode(node, list_tagpath, branch = False)
                    else:
                        listnode = node
                    encode_item(listnode, item, errs)
            else:
                if req is mandatory:
                    errs.append(_('Mandatory list "%s" under "%s" node is empty.') % (path, node.name()))

        def errors(l, where):
            errs = []
            ix = 1
            for n in l:
                errs.extend(errors_item(n, where + '[%s]' % ix))
                ix += 1
            return errs

        def format(l, f, errs):
            l.sort()
            for n in l:
                format_item(n, f, errs)
                f.add_literal_data(' ')

        return (init, decode, encode, errors, format)

    def gen_insetclass_tag(cls, tag, spec):
        """generate a class datatype that is highly integrated"""
        m_name, tag_type, req, path = cls.parse_spec(tag, spec)

        def make_object():
            obj = tag_type.__new__(tag_type)
            obj.__init__(tag=tag, req=req)
            return obj

        def init():
            return make_object()

        def decode(node, errs, where):
            if node:
                try:
                    obj = make_object()
                    obj.decode(node, errs, where)
                    return obj
                except Error:
                    errs.append(where + ': ' + _('Type mismatch: DOM cannot be decoded'))
            else:
                if req == mandatory:
                    errs.append(where + ': ' + _('Mandatory argument not available'))
            return None

        def encode(node, obj, errs):
            if node and obj:
                try:
                    obj.encode(node, errs)
                except Error:
                    if req == mandatory:
                        errs.append(_('Object cannot be encoded'))
            else:
                if req == mandatory:
                    errs.append(_('Mandatory argument not available'))

        def errors(obj, where):
            return obj.errors(where)

        def format(obj, f, errs):
            if obj:
                try:
                    obj.format(f, errs)
                except Error:
                    if req == mandatory:
                        errs.append(_('Object cannot be formatted'))
            else:
                if req == mandatory:
                    errs.append(_('Mandatory argument not available'))

        return (init, decode, encode, errors, format)

    def parse_spec(cls, token, spec):
        """decompose member specification"""
        m_name = cls.mixed_case(token)
        token_type = spec[0]
        req = spec[1]

        if len(spec) >= 3:
            path = spec[2]
        elif isinstance(token_type, list):
            if isinstance(token_type[0], autoxml):
                path = token + '/' + token_type[0].tag
            else:
                path = token
        elif isinstance(token_type, autoxml):
            path = token_type.tag
        else:
            path = token
        return m_name, token_type, req, path

    basic_cons_map = {
        str : str,
        int : int,
        float : float,
        }
