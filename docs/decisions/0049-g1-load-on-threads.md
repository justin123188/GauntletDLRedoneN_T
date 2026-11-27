# 0049: G1's load: the light carried and large textures on the threads

The game's side of decisions/0044, step 3: G1's load. With each area of new
ground it grew: the start area alone added 1.2 s, area A 2.0 s
(decisions/0043), area B 3.0 s (decisions/0046).

The loader's PERF line (decisions/0045), with the start area, area A and
area B at b1, gave this breakdown (the build of decisions/0045, c8d7441e):

| Phase | Time |
| --- | --- |
| Read and decode | 0.56 s (PNG decoding 0.17) |
| Mipmaps and detail maps | 0.70 to 0.74 s (20 jobs; the longest, 0.52 to 0.57 s, the 2048 px path texture's mipmaps) |
| Parts | 1.07 to 1.08 s (the light carried 0.96) |
| In all | 2.34 to 2.37 s |

Decision (NATIVE in what it touches; the results are the same bytes):
- **The light carried on the machine's threads** (model_pack.rs):
  - Each object's original surface is still found in order, since it
    reads the original's assets.
  - Then every new vertex's light runs on the threads, in runs of 4,096
    vertices, and the runs join in order.
- **A large mipmap level's rows in bands** (texfilter::generate, for a
  level of at least 256 × 256 texels):
  - The game's own textures share this code, so it makes the same
    texels as one band. Each texel reads only the level above.
  - The bands join in order, so a cut-out's averaged alphas keep their
    order for its binarising.
- **A large detail map's rows in bands** (material_maps, at least 256 ×
  256): the per-texel bump, roughness and metal loop, with the same
  texels.
- **DIAGNOSTIC:** the PERF line names the longest job.
- **Tests:** banded_mipmaps_equal_one_band (1024 × 512, blended and cut
  out) and banded_detail_maps_equal_one_band (512 × 512, opaque and cut
  out) compare the bytes made both ways.

Verification (2026-11-27, the owner's PC, the scratch copy; the build of
this change, md5 6e119bd2, against c8d7441e):
- **Tests and clippy:** gdl_client 89 + 5. Clippy has nothing new.
- **Identical frames:**
  - G1 at its start and C1 at its start, with the defaults;
  - G1 with GDL_BASELINE=1;
  - G1 with the review pack (the start area, areas A and B), at b1 with
    the defaults and at a1 with material detail off.
- **The simulation hashes** over 600 updates of the G1 walk equal the
  reference, in the defaults and the baseline.
- **The load** (the same pack, two runs each):

  | | Before | After |
  | --- | --- | --- |
  | First update | 6.90 to 7.01 s | 5.45 to 5.48 s |
  | G1's model pack | 2.34 to 2.37 s | 1.53 to 1.55 s |
  | Light carried | 0.96 s | 0.21 s |
  | Mipmaps and detail maps | 0.70 to 0.74 s | 0.67 to 0.68 s |

  - After the change, the longest texture job is the path texture's
    detail map, 0.44 to 0.46 s. Its downscale and its relief still run
    on one thread.
  - G1 without new ground has its first update at 4.29 s (decisions/0046),
    so with these areas G1 now loads about 1.2 s longer, not 3.0 s.
  - Frame times are unchanged: 8.40 to 8.48 ms against 8.44 and 8.46.

Later, if areas C and D need it: that detail map's downscale and relief
in bands, and the files read and parsed in parallel. Reading and
decoding takes 0.56 s, of which the PNGs are 0.17 s.

Installed 2026-11-27, with decisions/0045's build changes and the junk
chest's props (decisions/0047, "Installed").
