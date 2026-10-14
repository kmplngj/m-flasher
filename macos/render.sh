#!/bin/sh
# Renders the macOS artwork with the app's fonts (../assets), with headless
# Chromium: dmg/background.svg to the DMG's 1x and 2x backgrounds, icon.svg to
# the 1024-pixel icon.png the app's icon is made from.
#   CHROME=/path/to/chrome-headless-shell macos/render.sh
set -e
cd "$(dirname "$0")"
assets=$(cd ../assets && pwd)
render() { # svg, png, width, height, scale
  html=$(mktemp --suffix .html)
  cat > "$html" <<HTML
<html><head><style>
@font-face { font-family: "Barlow Condensed"; font-weight: 700; src: url("file://$assets/BarlowCondensed-Bold.ttf"); }
@font-face { font-family: "Barlow Condensed"; font-weight: 600; src: url("file://$assets/BarlowCondensed-SemiBold.ttf"); }
html, body { margin: 0; background: transparent; }
</style></head><body>$(cat "$1")</body></html>
HTML
  "${CHROME:-chromium}" --headless --no-sandbox --disable-gpu --hide-scrollbars --allow-file-access-from-files \
    --default-background-color=00000000 --force-device-scale-factor="$5" --window-size="$3,$4" \
    --screenshot="$PWD/$2" "file://$html" 2>/dev/null || true
  rm -f "$html"
}
render dmg/background.svg dmg/background.png 720 500 1
render dmg/background.svg dmg/background@2x.png 720 500 2
render icon.svg icon.png 1024 1024 1
ls -l dmg/background.png dmg/background@2x.png icon.png
