#!/usr/bin/python3

import os
import sys
import argparse
from gi.repository import Gio

EXTENSION_IFACE = 'org.gnome.Shell'
EXTENSION_PATH  = '/org/gnome/Shell'

INTERFACES = {
        'DBUS_PROP': 'org.freedesktop.DBus.Properties',
        'GNOME_SHELL': 'org.gnome.Shell'}

class ExtensionTool:
    def __init__(self):
        try:
            self.bus = Gio.bus_get_sync(Gio.BusType.SESSION, None)
            self.proxy = Gio.DBusProxy.new_sync(
                    self.bus, Gio.DBusProxyFlags.NONE, None, EXTENSION_IFACE,
                    EXTENSION_PATH, INTERFACES['GNOME_SHELL'], None)
        except:
            print("Exception: {0}".format(sys.exec_info()[1]))

    def get_shell_version(self):
        return self.proxy.get_cached_property('ShellVersion')

    def get_mode(self):
        return self.proxy.get_cached_property('Mode')

    def exec_script(self, script):
        output = self.proxy.Eval('(s)', script)
        return output

gs_script = 'test = function(number) {return number + 2;}; \
             test(3)'

def main():
    s = ExtensionTool()

    shellversion = s.get_shell_version()
    mode = s.get_mode()
    print('GNOME shell version: {0}'.format(shellversion))
    print('mode: {0}'.format(mode))

    result = s.exec_script(gs_script)
    print('rc: {0}, result: {1}'.format(result[0], result[1]))

if __name__ == "__main__":
    main()
