# Concept brief: the treasure chest

For ChatGPT (or any image generator). The picture it makes goes into an
image-to-3D tool (TRELLIS, which made the barrel, or Meshy or Tripo), and
the 3D model goes into the game as a replacement for the treasure chests.
Paste the prompt below as it is, or paste this whole page.

One chest serves nearly every world. G1's three kinds of chest (CHEST,
CHESTEXP and CHESTG2, the one with gold inside) are the same box with the
same paint, and so are the chests of every other world but the desert
world C, whose chests are another design. The
iron-banded chests (CHESTS, grey metal instead of gold) can be this chest
with its metal repainted, as the red and poison barrels are the brown one
repainted.

---

## Ready-to-paste prompt

> Create a single concept image of a **wooden treasure chest** for a
> dark-fantasy action game, to be turned into a 3D model.
>
> **The object:** one closed treasure chest, a box wider than it is deep
> (width : depth about 4 : 3) and about two-thirds as tall as it is wide.
> The lower part, the base, is a plain upright box; the lid on top is
> about a quarter of the chest's height. The lid's shape: a flat top with
> its front and back edges bevelled at about 45 degrees, so each end of
> the lid shows a flattened half-hexagon (a trunk lid, not a round dome).
> The lid is shut, and the seam between lid and base runs as a clear,
> straight, level line all the way round.
>
> Dark brown wooden planks running horizontally, with visible seams.
> Brass-gold metal bands along every edge and corner of the base and the
> lid, studded with round rivets. On the front, centred, a gold lock plate
> with a keyhole just below the seam, and above it on the lid's front edge
> a small decorative gold plate. Sturdy and slightly worn: softened edges,
> small dents and scratches, the gold a little tarnished in the corners.
> Nothing sticking out of the box: no hanging padlock, no rings or handles
> on the ends, no feet, nothing on top of it or around it.
>
> **The style:** a modern remaster of an early-2000s dark-fantasy arcade
> game. Keep the classic treasure-chest silhouette and its palette (warm
> dark-brown wood, near-black seams, brass-gold metal), but with richer
> detail, believable wood grain and metal. Semi-realistic and
> hand-crafted rather than photographic. It must still read clearly as a
> treasure chest when seen small, from above, in a game.
>
> **The image, for 3D conversion:**
> - the whole chest in view, centred, not cropped, filling about 70 % of
>   the frame;
> - seen at a three-quarter angle from the front, slightly above (about
>   25 degrees), so the front with the lock, one end and the lid are all
>   visible;
> - a plain, flat, light-grey background with no floor, no scenery and no
>   other objects;
> - soft, even studio lighting from the front; no strong highlights, no
>   dramatic rim light, no deep shadows, no cast shadow on the ground;
> - no text, logo, watermark or border;
> - square, as large as you can make it (at least 1024 × 1024).

---

## Why these rules

- **Closed, with a clear seam.** The game opens the chest by swinging its
  lid up from the back edge, so the lid and the base are separate parts.
  The image-to-3D tool makes one closed shape; the fitting script cuts it
  at the original's seam into a base and a lid, hinges the lid where the
  original's is, and builds the inside (bare dark wood) that shows when
  the lid is up. A straight seam where the original's is makes that cut
  clean.
- **Nothing sticking out.** The script fits the model to the original
  chest's box. A padlock or handles would make the box itself smaller, and
  a hanging padlock would split between lid and base.
- **The trunk lid.** The original's lid has bevelled front and back edges;
  keeping that shape keeps the chest's silhouette from the game camera.
- **One object, whole, on a plain background; soft, even light;
  three-quarter view from a little above.** As for the barrel: the tool
  builds the model from the outline, paints whatever lighting the picture
  has into the texture, and needs to see the front, an end and the lid at
  once. The game lights the chest itself.
- **Size, placement and polygon count don't matter.** The script scales
  the model to the original chest and reduces it.

## If the first picture is off

Ask ChatGPT to change only what is wrong, for example:
- "Remove the floor and the shadow; plain light-grey background only."
- "Close the lid fully; the seam should be one straight level line."
- "Make the lid flatter, with bevelled front and back edges, not round."
- "Remove the handles and the padlock; keep the keyhole plate."
- "Show the whole chest; it is cropped."
- "Keep everything else the same."

You can also attach a screenshot of a chest from your game and say
"in this shape and colours, remastered". That's your call: it shares a
picture of the game with ChatGPT.

## Then, in TRELLIS (or Meshy or Tripo)

1. Upload the picture as you did for the barrel (Image to 3D).
2. Ask for a **textured** model at the standard detail; the fitting script
   reduces it anyway.
3. Check the preview from all sides, especially the back and the
   underside, which the tool had to guess. Generate again if it is
   lopsided, has holes, or the lid bulges past the base.
4. Download it as **GLB** (.glb) into your Downloads folder.
5. Tell me the file name. I fit it to G1's chests (base and lid), cut and
   hinge the lid, show you the chest shut and open before it goes into
   your game, and then make every world's chests from it, as with the
   barrels.

Check the tool's terms on its free tier: some make free-tier models public or
ask for credit. That's fine for personal use; the model itself is never
uploaded to the public fork.

## Later, with the chests in

- **The gold inside.** The chests with gold inside (CHESTG2 in G1, and the
  gold versions the game swaps in for some chests) show a heap of coins.
  The script keeps the original heap at first; a new heap could be a
  second brief.
- **The burst chest.** A chest caught in a blast bursts and leaves broken
  remains (CHESTGEXP0 or CHESTSEXP0). Those could be cut from the new
  chest the way the barrel's breaking frames were.
