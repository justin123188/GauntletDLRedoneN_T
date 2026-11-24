# Concept brief: the standing barrel

For ChatGPT (or any image generator). The picture it makes goes into an
image-to-3D tool (Meshy or Tripo), and the 3D model goes into the game as a
replacement for the wooden barrels that stand around every level. Paste the
prompt below as it is, or paste this whole page.

---

## Ready-to-paste prompt

> Create a single concept image of a **wooden barrel** for a dark-fantasy
> action game, to be turned into a 3D model.
>
> **The object:** one upright wooden barrel, about as tall as it is wide
> (height slightly more than width, about 11 : 10), with a gently bulging
> middle. Vertical wooden staves with visible seams between them. Dark iron
> hoops: one near the top rim, one near the bottom rim, and one or two
> around the belly. A flat wooden lid made of planks, set just inside the top
> rim, with the planks running in one direction. Sturdy, chunky and slightly
> worn: rounded, softened edges, small dents and scuffs, a little rust on the
> hoops. No tap, no handles, no labels, no stand, nothing on top of it.
>
> **The style:** a modern remaster of an early-2000s dark-fantasy arcade game.
> Keep the classic barrel silhouette and its warm palette (orange-brown to
> dark-brown wood, near-black seams, dark grey-green iron), but with
> richer detail, believable wood grain and metal. Semi-realistic and
> hand-crafted rather than photographic. It must still read clearly as a
> barrel when seen small, from above, in a game.
>
> **The image, for 3D conversion:**
> - the whole barrel in view, centred, not cropped, filling about 70 % of the
>   frame;
> - seen at a three-quarter angle from slightly above (about 25 degrees), so
>   the lid and one side are both visible;
> - a plain, flat, light-grey background with no floor, no scenery and no
>   other objects;
> - soft, even studio lighting from the front; no strong highlights, no
>   dramatic rim light, no deep shadows, no cast shadow on the ground;
> - no text, logo, watermark or border;
> - square, as large as you can make it (at least 1024 × 1024).

---

## Why these rules

- **One object, whole, on a plain background.** Image-to-3D tools build the
  model from the outline; a floor, a shadow or a second object becomes part
  of the model.
- **Three-quarter view from a little above.** The tool sees the side and
  the lid at once, so both get shape and texture. The game also looks at
  barrels from above.
- **Soft, even light.** Whatever lighting is in the picture is painted into
  the model's texture. The game lights the barrel itself, so the picture
  should show the barrel's own colours, not a scene's lighting.
- **The original's silhouette and colours.** The remaster's rule: an
  evolution of the game's core look, the way its designers might have made
  it with modern hardware. Same shape and palette, rounder curves, more
  detail. Readability from the game camera comes first.
- **Size and placement don't matter.** The fitting script scales the model
  to the original barrel's size and stands it in the same spot. Polygon
  count doesn't matter either; the script reduces it.

## If the first picture is off

Ask ChatGPT to change only what is wrong, for example:
- "Remove the floor and the shadow; plain light-grey background only."
- "Show the whole barrel; it is cropped at the bottom."
- "Less shine on the metal, softer and more even lighting."
- "Make it a little squatter and wider in the middle."
- "Keep everything else the same."

You can also attach a screenshot of a barrel from your game and say
"in this shape and colours, remastered". That's your call: it shares a picture
of the game with ChatGPT.

## Then, in Meshy or Tripo

1. Choose **Image to 3D** and upload the picture.
2. Ask for a **textured** model. If there's a choice, pick the
   standard/medium detail; the fitting script reduces it anyway.
3. Check the preview from all sides, especially the back, which the tool
   had to guess. Generate again if it is lopsided or has holes.
4. Download it as **GLB** (.glb) into your Downloads folder.
5. Tell me the file name. I fit it to the original barrel, put it in
   `model_pack\ITEMS_G\BAROBJIDLE15F01.glb`, and show you a comparison
   before it goes into your game.

Check the tool's terms on its free tier: some make free-tier models public or
ask for credit. That's fine for personal use; the model itself is never
uploaded to the public fork.
