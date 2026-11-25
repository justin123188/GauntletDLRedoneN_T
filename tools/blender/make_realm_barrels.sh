#!/bin/bash
# Every realm's barrels from one fitted brown barrel (decisions/0026):
# recolour.py to each item package's brown (--hue shift), red and poison
# (--hue to), break_frames.py with the brown barrel's wood inside, and
# share_data.py per package.
#
#   BLENDER=.../blender.exe BLENDER_PYTHON=.../python.exe \
#     tools/blender/make_realm_barrels.sh EXPORT_OBJECTS BROWN.glb OUT_DIR [PACKAGE...]
#
# EXPORT_OBJECTS: a model export's objects folder (gdl_play --export-models);
# BROWN.glb: the fitted standing brown barrel for ITEMS_G's BAROBJIDLE15F01
# (fit_model.py); OUT_DIR gets ITEMS_<realm>/ folders for model_pack. The
# packages default to the ten with barrels (decisions/0026).
set -e
[ $# -ge 3 ] || { sed -n 2,14p "$0"; exit 2; }
: "${BLENDER:?set BLENDER to blender.exe}" "${BLENDER_PYTHON:?set BLENDER_PYTHON to the python.exe inside Blender}"
O="$1"; N="$2"; OUT="$3"; shift 3
REALMS="${*:-A B D E F G H J K S}"
T="$(cd "$(dirname "$0")" && pwd)"
FROM="$O/ITEMS_G/BAROBJIDLE15F01.glb"
mkdir -p "$OUT/reports"
for R in $REALMS; do
  src="$O/ITEMS_$R"; dst="$OUT/ITEMS_$R"
  mkdir -p "$dst"
  for K in BAROBJ BAREXP BARPOI; do
    idle=$(ls "$src" | grep "^${K}IDLE" | head -1 || true)
    [ -z "$idle" ] && continue
    if [ "$R$K" = "GBAROBJ" ]; then
      cp "$N" "$dst/$idle"
    else
      mode=to; [ "$K" = "BAROBJ" ] && mode=shift
      "$BLENDER" --background --factory-startup --python "$T/recolour.py" -- "$N" "$FROM" "$src/$idle" "$dst/$idle" \
        --hue $mode --report "$OUT/reports/$R-$K-recolour.json" 2>&1 | grep "recolour:\|Error\|Traceback" | sed "s/^/$R $K /" || true
    fi
    frames=$( (ls "$src" | grep "^${K}ACTI" | sort; ls "$src" | grep "^${K}DONE") | sed 's/\.glb$//' | tr '\n' ' ')
    "$BLENDER" --background --factory-startup --python "$T/break_frames.py" -- "$src" "$dst/$idle" "$dst" $frames \
      --inside-from "$N" --report "$OUT/reports/$R-$K-break.json" 2>&1 | grep "break_frames:\|Error\|Traceback" | sed "s/^/$R $K /" || true
  done
  "$BLENDER_PYTHON" -I "$T/share_data.py" "$dst" --report "$OUT/reports/$R-share.json" | sed "s/^/$R /"
done
echo done
