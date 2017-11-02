#!/bin/bash

dbus-send --session --type=method_call --print-reply \
      --dest=org.gnome.Shell \
      /org/gnome/Shell \
      org.freedesktop.DBus.Introspectable.Introspect
