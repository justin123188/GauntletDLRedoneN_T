# 0013: Sharper HUD fonts from a font file

The owner's request (2026-11-22, while redrawing pickup notices): the
small text, such as the crystal count "1/100" and the panels' "LV 1", is
low resolution. Both are font 1 of font_desc, 8Hifonts: a 128×128
texture (8HIFONTS in STATIC, format 146, white with alpha) whose glyphs
sit at the cells FONTS/8HIFONTS.FNT gives (code, width, x, y in the
texture's definition texels; one glyph height); the glyph blits crop
those cells (research/hud.md "Fonts"). A replacement must keep every
glyph in its cell, which hand-made or AI-made art does not do reliably.
The owner agreed to a helper that draws the page from a font file.

Decision:

- **texture_work\fonts** beside the game (made at startup with a README
  listing the game's fonts): a .ttf, .otf or .ttc file named after a
  font's texture ("8HIFONTS.ttf", "8HIFONTS Georgia.ttf") becomes that
  font's page in the texture pack, at startup and on F5 (before the HUD
  reload, so the page shows at once). Removing the file and pressing F5
  removes the page (texture_pack\fonts.tsv records the pages made, with
  the font file's name, length and time; a changed file is drawn again).
- **The drawing** (gdl_client::font_page; glyph outlines by swash, the
  version cosmic-text already brings in, now a direct dependency pinned
  at =0.2.10): a page 4 times the original's size; the baseline and the
  capital height are the medians over the original's capitals and digits
  (not J or Q); each glyph is drawn at the size that gives that capital
  height, centred where the original's ink was, narrowed (not shrunk)
  when wider than its cell, in the colour of the original's ink and with
  its strongest alpha (8HIFONTS: GS 0x78). When the font's descenders
  (g j p q y) would pass a cell's bottom, the letters move up into the
  room above the capitals, then get smaller. Glyphs keep the original's
  texels, magnified, when the font lacks them or their character is not
  established: codes outside printable ASCII and '^', '~' and '_'
  (8HIFONTS draws ñ at 0x5e and Ñ at 0x7e; an underscore lies below the
  cell).
- The pages are the front HUD's fonts (font_desc order, their textures in
  STATIC, TITLE or SELECT; MBNewFont's pages of per_page glyphs, each the
  next record). In play the same textures serve the panels.
- Presentation only, NATIVE; nothing is made without a font file, and the
  baseline has no texture pack.

Limits: the font files are the owner's (Windows' own, or free fonts); the
pages derive from them and from the disc's glyph layout and stay on the
PC like other packs. Fonts other than 8Hifonts (INITIALS, SCORE, FONT32)
work the same way but were not checked in the game; symbol fonts (bars,
arrows) would draw letters where symbols are.

Verification (2026-11-22, the owner's PC, the actual disc's import, a
scratch copy, isolated APPDATA): unit tests (metrics; a page from Bevy's
FiraMono subset: two glyphs drawn in their cells on the baseline with
the original's strongest alpha, an empty cell skipped; descenders kept
inside a cell; narrowing; the folder pass making, keeping and removing a
page, a wrongly named file reported; the README). Frozen G1 frames at
1600×900 with Georgia, Palatino Linotype, Times New Roman, Constantia,
Cambria Bold and Book Antiqua as 8HIFONTS: "LV 1" drawn in the font on
the panel; the pages hold every letter and digit in its cell, ñ, Ñ and
the symbols the rules keep as the original's (the first try drew '~'
where Ñ is, and shrank narrow cells' letters; both fixed). An F5 run
with Palatino dropped in during play and removed: "fonts drawn:
8HIFONTS; Textures reloaded: 1 replaced", then "1 font page(s) back to
the original; … 1 back to the original", screenshots of the original,
Palatino and the original again. Simulation hashes equal the reference
with a font page in use. Georgia and Constantia have old-style (uneven)
digits. The owner's playtest (2026-11-22, the build of 3f8c979 with
Palatino Linotype Bold as 8HIFONTS, their choice from the previews):
"Confirmed working and I like it".
