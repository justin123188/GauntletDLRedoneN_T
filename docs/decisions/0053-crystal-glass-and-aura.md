# 0053: See-through crystals and a smooth aura

The owner (2026-11-27), after the new crystals were installed (decisions/
0052): "I'm impressed with how they look tbh. Would it be possible to make
them more translucent. And upgrade that flickering aura they give off."
The owner's choice of the proposal: "Yes, build both".

What the game does (read 2026-11-27):
- **The shell is solid.** A crystal's shell (XSNS) is drawn opaque and
  unlit, its colour × 1.6 (decisions/0050). The core (XP) inside it is
  drawn added, so it is hidden.
- **The aura is a ring card that flickers.** The camera-facing card
  (CFXP_GLOW, 3.76 units across) carries GLOWRING_<colour>. Its texture
  modifier (POWERUPS ANIM) copies one of four 32 px frames,
  GLOWRING_<colour>00 to 03, into it every AFrame (divisor 1, 30 a
  second).
- **The ring's profile:** dark at the centre (behind the crystal),
  brightest at about 0.4 of the card's half-width, gone by 0.9. Frame 0's
  red texels peak at 0.47, 0.13, 0.13; white at 0.30, 0.34, 0.32; black
  at 0.15, 0.16, 0.14.

Decision (NATIVE; off in Settings::baseline() and the Original preset):
- **See-through crystals,** a new setting (Settings::gem_glass, F10 →
  Graphics → "See-through crystals", on in the defaults):
  - The shell blends: its alpha is 0.62 looking straight through and
    0.95 at the edge (a Fresnel-like term, power 2). The edge brightens
    by up to 40 %.
  - It still writes depth, so the glow card stays behind it as before.
  - The core draws just before the shell (GsDraw::sort_before, a depth
    bias of −0.002), so it glows through.
  - The sheen still slides (decisions/0052).
- **The aura,** under the existing "Gem and potion glow":
  - The ring card's texels come from gs.wgsl gem_aura in place of its
    frames:
    - the ring's profile (peak at 0.4; Gaussian falloff of 0.16 inside
      and 0.3 outside), × 1.25;
    - breathing ± 14 % at 0.6 Hz;
    - six faint rays turning one way and four the other;
    - three glints going round at 0.52 of the half-width.
  - The colour is the ring texture's own, its texels 0.3 to 0.5 from the
    centre.
  - The clock is the water clock (the updates), so it stops with the
    game. Each gem has its own phase.
- **How:**
  - A new GsParams `gem` (x 1 glass, y and z its alphas; x 2 + phase the
    aura, yzw its colour).
  - present_gem_glow finds a crystal's parts by their original textures
    (GEMSHEEN_*, GEM_*_IN, GLOWRING_*; a model pack part by its
    scroll_from) and makes MatReq requests.
  - The original crystals take both too, where the pack's are absent.

Verification (2026-11-27, the scratch copy with the new crystals; the build
of this change, md5 5b8aaffd, against 4986cfa7, the installed build):
- **Tests and clippy:** gdl_client 89 + 5. Clippy has nothing new.
- **Identical frames:**
  - with both gem effects off: G1's crystal spot and C1;
  - with GDL_BASELINE=1: G1's crystal spot.
- **The flicker:** around C1's white crystal (a 160 px region, the
  crystal and its halo), each update from 242 to 246 changes:
  - the installed build by 15.7, 14.9, 19.1 and 18.9 levels on average;
  - this build by 13.7, 13.5, 13.7 and 14.0, steady.
  - The steady remainder is mostly the crystal itself turning.
- **The original crystals** (no pack files): C1 and G1 change only at
  the crystals.
- **The look** (the art folder's Gems/Gems_Glass_Aura.png): G1's ruby,
  B1's amethyst, C1's ice white and D1's smoky black, before and after,
  and the halo over four updates.
  - **A first setting was too clear:** 0.45 through the middle, 0.9 at
    the edge. From the game's camera C1's white crystals looked ghostly
    and D1's black one nearly vanished.
  - **The setting above:** the crystals show the stairs behind them
    (B1) and keep their form.
- **The simulation hashes** over the 600-update G1 walk, with the
  crystals, equal the reference in the defaults and the baseline.

## Installed

2026-11-27 (the owner's "Yes, install"): the build of this change (md5
5b8aaffd). The previous one (4986cfa7) is kept as
gdl_play-glassaura-backup.exe. The model pack is unchanged.

Checked in the installed game (an isolated APPDATA): B1, C1 and D1 are
identical to the checked frames above, but for the HUD's "LV 1" label
(619 pixels, the owner's font; decisions/0047). All 16 crystal files
load, with no messages.

Not yet: the owner's look in the game.
