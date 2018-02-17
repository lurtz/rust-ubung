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

gs_script_0 = 'test = function(number) {return number + 2;}; \
               test(3);'
gs_script_1 = 'global.get_current_time()' # lg code
gs_script_2 = 'global.get_pointer()' # lg code
gs_script_3 = 'const PopupMenu = imports.ui.popupMenu; PopupMenu'
gs_script_4 = 'Main.overview.toggle()'
gs_script_5 = 'Main.panel._rightBox.get_width()'
gs_script_6 = "const Gvs = imports.gi.Gvc; Gvs.toString();"
gs_script_6 = "const Gvs = imports.gi.Gvc; \
               let dummy = new Gvs.MixerUIDevice(); \
               let id = dummy.get_id(); \
               delete dummy; \
               id;"
gs_script_7 = "const VolumeMenu = imports.ui.status.volume; \
               VolumeMenu.getMixerControl().get_state();"
gs_script_8 = "const Gvc = imports.gi.Gvc; \
               Gvc.MixerControlState.READY"

scripts = [
        gs_script_0,
        gs_script_1,
        gs_script_2,
        gs_script_3,
        gs_script_5,
        gs_script_7,
        gs_script_8
        ]

def main():
    s = ExtensionTool()

    shellversion = s.get_shell_version()
    mode = s.get_mode()
    print('GNOME shell version: {0}'.format(shellversion))
    print('mode: {0}'.format(mode))

    for scr in scripts:
        result = s.exec_script(scr)
        print('rc: {0}, result: {1}'.format(result[0], result[1]))

if __name__ == "__main__":
    main()
