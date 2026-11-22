//! NATIVE (this fork, decisions/0013): a HUD font drawn from a TrueType or
//! OpenType font file. A font's glyphs sit at fixed places on its texture
//! (FONTS/*.FNT: code, width, x, y in the texture's definition texels, one
//! height per font; the glyph blits crop those cells), so a page drawn with
//! every letter in its cell replaces the font through the texture pack.
//! `render` draws a page `SCALE` times the original's size: the baseline
//! and the capital height measured on the original's capitals and digits,
//! each letter centred where the original's ink was (narrowed when wider
//! than its cell), in the original ink's colour and strongest alpha; when
//! the font's descenders do not fit below the baseline, the letters move up
//! into the room above the capitals, then get smaller. Codes outside
//! printable ASCII keep the original's texels (magnified): their character
//! set is not established; nor are '^', '~' and '_' (8HIFONTS draws ñ and
//! Ñ at 0x5e and 0x7e; an underscore lies below the cell). `process` makes the pages for the
//! font files in texture_work\fonts, each named after a font's texture
//! (8HIFONTS.ttf, "8HIFONTS Georgia.ttf"), and removes a page when its font
//! file is gone; fonts.tsv in the pack records the pages it made.
use gdl_formats::fnt::Glyph;
use std::path::{Path, PathBuf};
use swash::FontRef;
use swash::scale::{Render, ScaleContext, Source};
use swash::zeno::Format;

/// How many times the original's size a page is (128×128 → 512×512).
pub const SCALE: u32 = 4;

/// One page of a HUD font: its texture and the glyphs on it.
#[derive(Clone, Debug)]
pub struct FontTexture {
    /// The font's texture label (font_desc's name, e.g. "8HIFONTS").
    pub label: String,
    /// texture_pack::name of the original texels.
    pub name: String,
    pub glyphs: Vec<Glyph>,
    /// The glyph height (FNT +4), in definition texels.
    pub height: i32,
    /// The definition size the glyph cells are given in (TEXDEF).
    pub def: (u32, u32),
    /// The texture's size and its GS RGBA texels (alpha 0x80 opaque).
    pub size: (u32, u32),
    pub rgba: Vec<u8>,
}

/// What a page took from the font file.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Drawn {
    /// Glyphs drawn from the font.
    pub drawn: usize,
    /// Glyphs that kept the original (not in the font, or not ASCII).
    pub kept: usize,
}

/// The character a glyph code stands for: printable ASCII but '^', '_'
/// and '~' (see the module's notes).
fn char_of(code: i32) -> Option<char> {
    (0x21..=0x7e).contains(&code).then_some(code as u8 as char).filter(|c| !matches!(c, '^' | '_' | '~'))
}

/// Coverage `data` (`w` × `h`) narrowed to `to` columns (area average).
fn squeeze(data: &[u8], w: usize, h: usize, to: usize) -> Vec<u8> {
    let r = w as f32 / to.max(1) as f32;
    let mut out = Vec::with_capacity(to * h);
    for y in 0..h {
        for x in 0..to {
            let (a, b) = (x as f32 * r, (x + 1) as f32 * r);
            let mut sum = 0.0;
            let mut i = a.floor() as usize;
            while (i as f32) < b && i < w {
                sum += f32::from(data[y * w + i]) * (b.min(i as f32 + 1.0) - a.max(i as f32));
                i += 1;
            }
            out.push((sum / r).round().min(255.0) as u8);
        }
    }
    out
}

impl FontTexture {
    /// Definition texels to texture texels.
    fn k(&self) -> (f32, f32) {
        (self.size.0 as f32 / self.def.0.max(1) as f32, self.size.1 as f32 / self.def.1.max(1) as f32)
    }

    /// A texel's GS alpha (0 outside).
    fn alpha(&self, x: i64, y: i64) -> u8 {
        if x < 0 || y < 0 || self.size.0 as i64 <= x || self.size.1 as i64 <= y {
            return 0;
        }
        self.rgba.get((y as usize * self.size.0 as usize + x as usize) * 4 + 3).copied().unwrap_or(0)
    }

    /// A glyph's ink box in its cell (x0, x1, y0, y1 in definition texels
    /// from the cell's corner) and its ink's mean colour; None: no ink.
    fn ink(&self, g: &Glyph) -> Option<([f32; 4], [u8; 3])> {
        let (kx, ky) = self.k();
        let (tx, ty) = (g.x as f32 * kx, g.y as f32 * ky);
        let (tw, th) = (g.width as f32 * kx, self.height as f32 * ky);
        let (mut b, mut sum, mut n) = ([f32::MAX, f32::MIN, f32::MAX, f32::MIN], [0u64; 3], 0u64);
        for y in ty.floor() as i64..(ty + th).ceil() as i64 {
            for x in tx.floor() as i64..(tx + tw).ceil() as i64 {
                let a = self.alpha(x, y);
                if a < 0x20 {
                    continue;
                }
                let (cx, cy) = ((x as f32 - tx) / kx, (y as f32 - ty) / ky);
                b = [b[0].min(cx), b[1].max(cx + 1.0 / kx), b[2].min(cy), b[3].max(cy + 1.0 / ky)];
                let i = (y as usize * self.size.0 as usize + x as usize) * 4;
                for (c, s) in sum.iter_mut().enumerate() {
                    *s += u64::from(self.rgba[i + c]) * u64::from(a);
                }
                n += u64::from(a);
            }
        }
        (0 < n).then(|| (b, sum.map(|s| (s / n) as u8)))
    }

    /// The baseline and the capitals' top (definition texels from a cell's
    /// top): the medians over the capitals and digits (not J or Q).
    pub fn metrics(&self) -> (f32, f32) {
        let mut bottoms = Vec::new();
        let mut tops = Vec::new();
        for g in &self.glyphs {
            let c = char_of(g.code).unwrap_or(' ');
            if (c.is_ascii_uppercase() || c.is_ascii_digit()) && c != 'J' && c != 'Q'
                && let Some((b, _)) = self.ink(g)
            {
                bottoms.push(b[3]);
                tops.push(b[2]);
            }
        }
        let median = |mut v: Vec<f32>, or: f32| {
            v.sort_by(f32::total_cmp);
            v.get(v.len() / 2).copied().unwrap_or(or)
        };
        let h = self.height as f32;
        (median(bottoms, h * 0.8), median(tops, h * 0.1))
    }
}

/// A glyph of `font` at `size` pixels per em: alpha coverage and its
/// placement (left, top above the baseline, width, height).
fn glyph(cx: &mut ScaleContext, font: FontRef, id: u16, size: f32) -> Option<(Vec<u8>, i32, i32, u32, u32)> {
    let mut scaler = cx.builder(font).size(size).hint(false).build();
    let im = Render::new(&[Source::Outline]).format(Format::Alpha).render(&mut scaler, id)?;
    let p = im.placement;
    (0 < p.width && 0 < p.height).then_some((im.data, p.left, p.top, p.width, p.height))
}

/// Draw page `t` with the font file `ttf`: (width, height, RGBA with PNG
/// alpha, what was drawn).
pub fn render(ttf: &[u8], t: &FontTexture) -> Result<(u32, u32, Vec<u8>, Drawn), String> {
    let font = FontRef::from_index(ttf, 0).ok_or("not a font file (TrueType or OpenType)")?;
    let mut cx = ScaleContext::new();
    let (w, h) = (t.size.0 * SCALE, t.size.1 * SCALE);
    let (sx, sy) = (w as f32 / t.def.0.max(1) as f32, h as f32 / t.def.1.max(1) as f32);
    let (base, top) = t.metrics();
    // The font's capital height at 100 px (its H), then the size giving
    // the original's capital height.
    let h_id = font.charmap().map('H');
    let cap = glyph(&mut cx, font, h_id, 100.0).map(|g| g.2).filter(|&v| 0 < v).ok_or("the font has no H")?;
    let mut size = 100.0 * (base - top).max(1.0) * sy / cap as f32;
    // The baseline in page pixels below a cell's top: the original's, or
    // higher (then the size smaller) for the font's descenders.
    let (cell_h, mv) = (t.height as f32 * sy, 0.5 * sy);
    let mut cap_px = (base - top).max(1.0) * sy;
    let mut base_px = base * sy;
    let descent = |cx: &mut ScaleContext, size: f32| {
        t.glyphs.iter().filter_map(|g| char_of(g.code).filter(|c| "gjpqy".contains(*c))).filter_map(|c| glyph(cx, font, font.charmap().map(c), size)).map(|g| (g.4 as i32 - g.2) as f32).fold(0.0, f32::max)
    };
    let depth = descent(&mut cx, size);
    if cell_h - mv < base_px + depth {
        base_px -= (base_px + depth - (cell_h - mv)).min((base_px - cap_px - mv).max(0.0));
        if cell_h - mv < base_px + depth {
            let f = (cell_h - 2.0 * mv) / (cap_px + depth);
            size *= f;
            cap_px *= f;
            base_px = mv + cap_px;
        }
    }
    let strongest = t.rgba.chunks_exact(4).map(|p| p[3]).max().unwrap_or(0x80);
    let png_alpha = |a: u8| (u32::from(a) * 2).min(255);
    let mut out = vec![0u8; (w * h * 4) as usize];
    let mut done = Drawn::default();
    for g in t.glyphs.iter().filter(|g| 0 < g.width) {
        let Some((ink, colour)) = t.ink(g) else { continue };
        // The cell in page pixels.
        let (x0, y0) = ((g.x as f32 * sx).round() as i64, (g.y as f32 * sy).round() as i64);
        let (x1, y1) = (((g.x + g.width) as f32 * sx).round() as i64, ((g.y + t.height) as f32 * sy).round() as i64);
        let id = char_of(g.code).map_or(0, |c| font.charmap().map(c));
        let mut put = |x: i64, y: i64, rgb: [u8; 3], a: u32| {
            if x0 <= x && x < x1 && y0 <= y && y < y1 && 0 <= x && 0 <= y && x < w as i64 && y < h as i64 {
                let i = (y as usize * w as usize + x as usize) * 4;
                if out[i + 3] as u32 <= a {
                    out[i..i + 4].copy_from_slice(&[rgb[0], rgb[1], rgb[2], a as u8]);
                }
            }
        };
        let margin = (sx * 0.5).round() as i64;
        let room = (x1 - x0 - 2 * margin).max(1) as f32;
        // Wider than the cell: narrowed, the height kept.
        let drawn = if id == 0 {
            None
        } else {
            glyph(&mut cx, font, id, size).map(|(data, l, gtop, gw, gh)| {
                if room < gw as f32 {
                    let to = room as u32;
                    (squeeze(&data, gw as usize, gh as usize, to as usize), l, gtop, to, gh)
                } else {
                    (data, l, gtop, gw, gh)
                }
            })
        };
        match drawn {
            Some((data, _, gtop, gw, _)) => {
                let centre = x0 as f32 + (ink[0] + ink[1]) * 0.5 * sx;
                let ox = ((centre - gw as f32 * 0.5).round() as i64).clamp(x0 + margin, (x1 - margin - gw as i64).max(x0 + margin));
                let oy = (y0 as f32 + base_px).round() as i64 - i64::from(gtop);
                for (k, &c) in data.iter().enumerate() {
                    if 0 < c {
                        let (dx, dy) = ((k % gw as usize) as i64, (k / gw as usize) as i64);
                        put(ox + dx, oy + dy, colour, png_alpha(strongest) * u32::from(c) / 255);
                    }
                }
                done.drawn += 1;
            }
            None => {
                // The original's texels, magnified bilinearly.
                let (kx, ky) = t.k();
                for y in y0..y1 {
                    for x in x0..x1 {
                        let (u, v) = (((x as f32 + 0.5) / sx) * kx - 0.5, ((y as f32 + 0.5) / sy) * ky - 0.5);
                        let (ux, vy) = (u.floor(), v.floor());
                        let (fu, fv) = (u - ux, v - vy);
                        let at = |dx: i64, dy: i64| f32::from(t.alpha(ux as i64 + dx, vy as i64 + dy));
                        let a = at(0, 0) * (1.0 - fu) * (1.0 - fv) + at(1, 0) * fu * (1.0 - fv) + at(0, 1) * (1.0 - fu) * fv + at(1, 1) * fu * fv;
                        put(x, y, colour, png_alpha(a.round() as u8));
                    }
                }
                done.kept += 1;
            }
        }
    }
    Ok((w, h, out, done))
}

/// What `process` did.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FontReport {
    /// Pages drawn (labels).
    pub made: Vec<String>,
    /// Pages removed (their font file went).
    pub removed: usize,
    pub problems: Vec<String>,
}

impl FontReport {
    /// The on-screen part ("" when nothing happened).
    pub fn message(&self) -> String {
        let mut parts = Vec::new();
        if !self.made.is_empty() {
            parts.push(format!("fonts drawn: {}", self.made.join(", ")));
        }
        if 0 < self.removed {
            parts.push(format!("{} font page(s) back to the original", self.removed));
        }
        if let Some(p) = self.problems.first() {
            parts.push(p.clone());
        }
        parts.join("; ")
    }
}

/// A font file's fingerprint as fonts.tsv keeps it (name, length, time).
fn fingerprint(path: &Path) -> Option<String> {
    let m = std::fs::metadata(path).ok()?;
    let t = m.modified().ok().and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok()).map_or(0, |d| d.as_secs());
    Some(format!("{}\t{}\t{t}", path.file_name()?.to_string_lossy(), m.len()))
}

/// The pages for the font files in `fonts` (texture_work\fonts) among
/// `textures` (every page of the HUD's fonts), written into `pack`; pages
/// made earlier whose font file is gone are removed. A page is drawn again
/// when its font file changed (name, length or time) or the page is gone.
pub fn process(fonts: &Path, pack: &Path, textures: &[FontTexture]) -> FontReport {
    let mut report = FontReport::default();
    let records_path = pack.join("fonts.tsv");
    // name → (label, fingerprint)
    let mut records: std::collections::BTreeMap<String, (String, String)> = std::fs::read_to_string(&records_path)
        .unwrap_or_default()
        .lines()
        .filter_map(|l| {
            let mut f = l.splitn(3, '\t');
            Some((f.next()?.to_string(), (f.next()?.to_string(), f.next()?.to_string())))
        })
        .collect();
    let before = records.clone();
    let files: Vec<PathBuf> = std::fs::read_dir(fonts)
        .map(|d| d.filter_map(|e| e.ok()).map(|e| e.path()).filter(|p| p.is_file()).collect())
        .unwrap_or_default();
    let is_font = |p: &Path| p.extension().is_some_and(|e| ["ttf", "otf", "ttc"].contains(&e.to_string_lossy().to_ascii_lowercase().as_str()));
    let labels: std::collections::BTreeSet<&str> = textures.iter().map(|t| t.label.as_str()).collect();
    // Each label's font file: the longest label its name starts with.
    let mut chosen: std::collections::BTreeMap<&str, &PathBuf> = std::collections::BTreeMap::new();
    let mut sorted = files.clone();
    sorted.sort();
    for p in sorted.iter().filter(|p| is_font(p)) {
        let stem = p.file_stem().map(|s| s.to_string_lossy().to_ascii_uppercase()).unwrap_or_default();
        match labels.iter().filter(|l| stem.starts_with(&l.to_ascii_uppercase())).max_by_key(|l| l.len()) {
            Some(l) => {
                chosen.insert(*l, p);
            }
            None => report.problems.push(format!("{}: name it after a font ({})", p.file_name().unwrap_or_default().to_string_lossy(), labels.iter().copied().collect::<Vec<_>>().join(", "))),
        }
    }
    for t in textures {
        let page = pack.join(format!("{}.png", t.name));
        match chosen.get(t.label.as_str()) {
            Some(file) => {
                let Some(print) = fingerprint(file) else { continue };
                if records.get(&t.name).is_some_and(|(_, f)| *f == print) && page.is_file() {
                    continue;
                }
                let made = std::fs::read(file).map_err(|e| e.to_string()).and_then(|b| render(&b, t)).and_then(|(w, h, rgba, _)| crate::texture_pack::write_png(&page, w, h, &rgba).map_err(|e| e.to_string()));
                match made {
                    Ok(()) => {
                        records.insert(t.name.clone(), (t.label.clone(), print));
                        if !report.made.contains(&t.label) {
                            report.made.push(t.label.clone());
                        }
                    }
                    Err(e) => report.problems.push(format!("{}: {e}", file.file_name().unwrap_or_default().to_string_lossy())),
                }
            }
            None => {
                if records.remove(&t.name).is_some() {
                    let _ = std::fs::remove_file(&page);
                    report.removed += 1;
                }
            }
        }
    }
    if records != before {
        let text: String = records.iter().map(|(n, (l, f))| format!("{n}\t{l}\t{f}\n")).collect();
        if records.is_empty() {
            let _ = std::fs::remove_file(&records_path);
        } else {
            let _ = std::fs::write(&records_path, text);
        }
    }
    report
}

/// texture_work\fonts\README.txt: how to use it and the fonts there are.
pub fn readme(textures: &[FontTexture]) -> String {
    let mut labels: Vec<&str> = textures.iter().map(|t| t.label.as_str()).collect();
    labels.dedup();
    let known = |l: &str| match l {
        "8HIFONTS" => " (small text: \"LV 1\", crystal counts, many messages)",
        "INITIALS" => " (the players' names on their panels)",
        "SCORE" => " (gold and health numbers)",
        "FONT32" => " (large glowing text)",
        _ => "",
    };
    let list: String = labels.iter().map(|l| format!("  {l}{}\r\n", known(l))).collect();
    format!(
        "SHARPER FONTS\r\n\r\n\
         Put a font file (.ttf or .otf) here, named after one of the game's\r\n\
         fonts below, for example 8HIFONTS.ttf (words after the name are\r\n\
         fine: \"8HIFONTS Georgia.ttf\"). At the next start, or when you press\r\n\
         F5 in play, the game draws every letter of that font again in your\r\n\
         font, at four times the size, in the same places, and uses it.\r\n\r\n\
         To find a font: Windows keeps its fonts in C:\\Windows\\Fonts. Copy\r\n\
         one here (for example Georgia Bold, georgiab.ttf) and rename the\r\n\
         copy. Free fonts (Google Fonts and others) work too.\r\n\r\n\
         To go back to the original letters, delete the font file here and\r\n\
         press F5 (or restart). Symbols the font lacks, and accented letters,\r\n\
         keep the original drawing.\r\n\r\n\
         The game's fonts:\r\n{list}"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A 2-glyph page: "H" and "x" as ink boxes on a 32×16 white texture
    /// (definition size the same), glyph height 12, baseline at row 10.
    fn page() -> FontTexture {
        let (w, h) = (32u32, 16u32);
        let mut rgba = vec![0u8; (w * h * 4) as usize];
        let mut ink = |x0: u32, x1: u32, y0: u32, y1: u32| {
            for y in y0..y1 {
                for x in x0..x1 {
                    let i = ((y * w + x) * 4) as usize;
                    rgba[i..i + 4].copy_from_slice(&[255, 255, 255, 0x78]);
                }
            }
        };
        ink(2, 9, 2, 10);
        ink(18, 23, 5, 10);
        FontTexture {
            label: "TESTFONT".into(),
            name: "0123456789abcdef".into(),
            glyphs: vec![Glyph { code: 'H' as i32, width: 10, x: 0, y: 0 }, Glyph { code: 'x' as i32, width: 8, x: 16, y: 0 }, Glyph { code: 0xa9, width: 8, x: 24, y: 0 }],
            height: 12,
            def: (w, h),
            size: (w, h),
            rgba,
        }
    }

    #[test]
    fn metrics_and_characters() {
        let t = page();
        assert_eq!(t.metrics(), (10.0, 2.0));
        assert_eq!((char_of('A' as i32), char_of(0x20), char_of(0xa9), char_of('~' as i32)), (Some('A'), None, None, None));
        // Two columns of 100 and one of 200 into two: area averages.
        assert_eq!(squeeze(&[100, 100, 200, 0, 0, 0], 3, 2, 2), [100, 167, 0, 0]);
    }

    #[test]
    fn a_page_from_a_font() {
        let t = page();
        let (w, h, rgba, done) = render(bevy::text::DEFAULT_FONT_DATA, &t).unwrap();
        assert_eq!((w, h), (128, 64));
        // H and x from the font; the © cell is empty in the original.
        assert_eq!(done, Drawn { drawn: 2, kept: 0 });
        let a = |x: u32, y: u32| rgba[((y * w + x) * 4 + 3) as usize];
        // Ink stays in its cell and sits on the baseline (row 10 × 4).
        assert!((0..w).all(|x| (0..h).all(|y| a(x, y) == 0 || x < 40 || (64..96).contains(&x))));
        let rows: Vec<u32> = (0..h).filter(|&y| (0..40).any(|x| 128 < a(x, y))).collect();
        assert!(rows.last().is_some_and(|&r| (38..=40).contains(&r)), "{rows:?}");
        assert!(rows.first().is_some_and(|&r| (7..=10).contains(&r)), "{rows:?}");
        // The original's strongest alpha (0x78 → 240) and colour.
        assert_eq!((0..w * h).map(|i| rgba[(i * 4 + 3) as usize]).max(), Some(240));
        assert!(render(b"not a font", &t).is_err());
    }

    /// A descender (p) that would pass the cell's bottom: the letters move
    /// up and the p's tail stays inside.
    #[test]
    fn descenders_stay_in_the_cell() {
        let mut t = page();
        // A p in the third cell (x 24..32), ink rows 5..12 (to the bottom).
        t.glyphs[2] = Glyph { code: 'p' as i32, width: 8, x: 24, y: 0 };
        for y in 5..12u32 {
            for x in 26..30u32 {
                let i = ((y * 32 + x) * 4) as usize;
                t.rgba[i..i + 4].copy_from_slice(&[255, 255, 255, 0x78]);
            }
        }
        let (w, _, rgba, done) = render(bevy::text::DEFAULT_FONT_DATA, &t).unwrap();
        assert_eq!(done.drawn, 3);
        let lowest = |x0: u32, x1: u32| (0..48u32).filter(|&y| (x0..x1).any(|x| 0 < rgba[((y * w + x) * 4 + 3) as usize])).max().unwrap();
        // The cell ends at row 48; the p's tail ends above it, the H's
        // foot above the p's tail.
        let (p, h) = (lowest(96, 128), lowest(0, 40));
        assert!(p < 47 && h < p, "p {p}, H {h}");
    }

    #[test]
    fn pages_follow_the_font_files() {
        let root = std::env::temp_dir().join(format!("gdl-font-page-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let (fonts, pack) = (root.join("fonts"), root.join("pack"));
        std::fs::create_dir_all(&fonts).unwrap();
        std::fs::create_dir_all(&pack).unwrap();
        let t = page();
        let file = pack.join("0123456789abcdef.png");
        assert_eq!(process(&fonts, &pack, std::slice::from_ref(&t)), FontReport::default());
        std::fs::write(fonts.join("TESTFONT Fira.ttf"), bevy::text::DEFAULT_FONT_DATA).unwrap();
        std::fs::write(fonts.join("Other.ttf"), bevy::text::DEFAULT_FONT_DATA).unwrap();
        let r = process(&fonts, &pack, std::slice::from_ref(&t));
        assert_eq!(r.made, ["TESTFONT"]);
        assert!(r.problems[0].starts_with("Other.ttf: name it after a font (TESTFONT)"));
        assert!(file.is_file() && pack.join("fonts.tsv").is_file());
        // Unchanged: nothing to do; the font file gone: the page goes too.
        assert!(process(&fonts, &pack, std::slice::from_ref(&t)).made.is_empty());
        std::fs::remove_file(fonts.join("TESTFONT Fira.ttf")).unwrap();
        let r = process(&fonts, &pack, std::slice::from_ref(&t));
        assert_eq!(r.removed, 1);
        assert!(!file.exists() && !pack.join("fonts.tsv").exists());
        assert!(readme(&[t]).contains("  TESTFONT\r\n"));
        std::fs::remove_dir_all(&root).unwrap();
    }
}
