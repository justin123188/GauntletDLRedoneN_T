# 0010: HD textures keep the original's soft transparency

The owner's playtest (2026-11-22, this fork): some 2D sprites, the glows
of gem pickups and spells, "almost look like they lost some of their
transparency" with a black outline, most visible zoomed in with the free
camera. The HD pack (research/enhancements.md "HD textures") keeps the
originals of effects, but these sprites are not effects by its rules
(they are drawn as items and weapon parts): GEMSHEEN_*, GEM_HIGHLIGHT,
GEMSTONES3, GLOWBOLT, GLOWBOLTF, GLOWFEATHER, TURBO_GLOW_NEW and MAGIC
were in the pack. Their blended alpha went through the network as grey,
which sharpened a soft halo into hard bands with clear gaps (GLOWBOLT: the
arrow, a clear ring, a solid outer ring).

The owner's choice (2026-11-22, over keeping such sprites original):
keep the HD colour, upscale the transparency smoothly, and keep the glow.

Decision:

- **A blended alpha is the original's, upscaled bilinearly** (hd::prep,
  wrapped or clamped per axis like the padding): the GPU's own
  magnification of the original, so a glow fades as it did. The colour
  still goes through the network. Cut-outs are unchanged (network,
  thresholded at half); opaque textures stay opaque. Blended textures no
  longer take a network pass for their alpha.
- **PROCESSING 2** (hd::generate): the pack directory's name changes
  (…-p2), so a game with HD textures on makes the new pack once (about 4
  minutes on the owner's RTX 3060). The p1 directory stays until removed.
- **Brightness.** The owner asked that glows keep their glow. A texel
  brighter than opaque (GS alpha above 0x80) could not survive the pack's
  PNG alpha; the census of the disc found none among its 20596 textures
  as the client decodes them, so nothing is kept for that reason (a rule
  for it was written and removed again: it matched nothing).
- tools/python/upscale_compare.py (upstream's reference) still sends
  blended alpha through the network: `gdl_play --hd-check` against its
  packs now differs in blended textures' alpha by design.

Verification (2026-11-22, the owner's PC, the actual disc's import):
synthetic unit tests (the bilinear upscale clamped and wrapped, which
textures skip the network pass, finish's alpha classes). A full PROCESSING 2
pack generated from the disc: 13742 textures, 6854 kept, complete, 258.9 s.
The glow sprites compared from the files (the G1 dump against the p1 and
p2 packs): p2's alpha follows the original's soft halo, p1's had hard
bands and gaps. Not checked yet: the same sprites in a rendered frame, and
the owner's playtest.
