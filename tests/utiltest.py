import unittest
import shutil
from pisi.util import *
import os

class UtilTestCase(unittest.TestCase):

    def initialize(self):
        testcase.testCase.initialize(self, database=False)

    #process related functions
    def testRunBatch(self):
        assert (0, '', '') == run_batch('cd')
        retcode, out, err = run_batch('add')
        assert retcode == 127  # command not found

    def testRunLogged(self):
        assert 0 == run_logged('ls')
        assert 1 == run_logged('rm')

    def testXtermTitle(self):
        xterm_title('pardus')
        xterm_title_reset()

    #path processing functions tests
    def testSplitPath(self):
        assert ['usr', 'local', 'src'] == splitpath('usr/local/src')
        assert ['usr', 'lib', 'pardus'] == splitpath('usr/lib/pardus')

    def testSubPath(self):
        self.assertTrue(subpath('usr','usr'))
        self.assertTrue(subpath('usr','usr/local/src'))
        self.assertTrue(not subpath('usr/local','usr'))

    def testRemovePathPrefix(self):
        pathname = removepathprefix('usr/local', 'usr/local/src')
        assert 'src' == pathname

        pathname = removepathprefix('usr/local','usr/local/bin')
        assert not 'bim' == pathname

    def testJoinPath(self):
        assert 'usr/local/src' == join_path('usr/local','src')
        assert not 'usr/lib/hal' == join_path('usr','hal')
        assert 'usr/sbin/lpc' == join_path('usr','sbin/lpc')

    #file/directory related functions tests
    def testCheckFile(self):
        assert check_file(os.path.abspath('runtests.py'))
        assert check_file(os.path.abspath('mirrors.conf'))

    def testCleanDir(self):
        assert None == clean_dir('usr/lib')
        assert None == clean_dir('usr/local')
        assert not 'tmp/pisi-root' == clean_dir('usr/tmp')

    def testDirSize(self):
        self.assertNotEqual(dir_size('usr/lib/pardus'),2940)
        self.assertNotEqual(dir_size('usr/lib'),65)

    def testCopyFile(self):
        copy_file(os.path.abspath('runtests.py'), os.path.abspath('tmp/pisi-test1'))
        copy_file(os.path.abspath('metadata.xml'), os.path.abspath('tmp/pisi-test2'))
        copy_file_stat(os.path.abspath('runtests.py'), os.path.abspath('tmp/pisi-test1'))
