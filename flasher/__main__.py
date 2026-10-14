"""python -m flasher: the window (the webview one, or the Tk one if no
webview is available, or with --tk); with other arguments, the command line."""
import sys


def main():
    args = sys.argv[1:]
    if args and args != ['--tk']:
        from .cli import main as cli
        return cli()
    if not args:
        try:
            from .webui import main as web
        except Exception as e:          # no pywebview, or no WebView2 / WebKitGTK
            print(f'webview window unavailable ({e}); using the simple window', file=sys.stderr)
        else:
            return web()
    from .gui import main as tk
    tk()


if __name__ == '__main__':
    main()
