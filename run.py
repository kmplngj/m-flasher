"""Entry point for the packaged app (PyInstaller): the window, or the
command line when given arguments."""
import sys

if len(sys.argv) > 1:
    from flasher.cli import main
else:
    from flasher.gui import main

if __name__ == '__main__':
    main()
