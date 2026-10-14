"""Entry point for the packaged app (PyInstaller): the window, or the
command line when given arguments."""
import sys

from flasher.__main__ import main

if __name__ == '__main__':
    main()
