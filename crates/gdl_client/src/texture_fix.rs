//! NATIVE (this fork, decisions/0009-texture-workflow.md "Fix-up"): the
//! texture_work folder beside the game. The owner saves an image made for a
//! texture (ChatGPT's, any size) there, named after the texture's file in
//! texture_dump (`<name>.png`; words after the name are allowed:
//! `<name> v2.png`). At startup and on F5 the game fits each such image to
//! the texture and writes the result into the texture pack:
//!   - the size: an exact multiple of the original (the image's own scale
//!     rounded, 1..8, at most 2048 texels a side), the original's
//!     proportions (a square AI image of a 2:1 texture is squeezed back);
//!     resampled with a Catmull-Rom kernel, widened when it shrinks;
//!   - the transparency: the original's (an AI image keeps no cut-out
//!     mask), upscaled bilinearly as hd::prep's blended alpha, a cut-out's
//!     thresholded at half; opaque originals stay opaque;
//!   - clear texels near kept ones (four rings) take their kept
//!     neighbours' colour (hd::prep's fill_clear_rings), so no dark rim
//!     grows into magnified edges.
//!
//! The original comes from texture_dump (`<name>.png`, alpha doubled) with
//! its record flags (index.tsv: repeat or clamp per axis). The image then
//! moves to texture_work\done. Only PNG files are read.
use crate::hd::prep;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// What a pass over texture_work did.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct FixReport {
    /// Images written into the pack.
    pub fixed: usize,
    /// Images left in texture_work, with the reason (shown on screen).
    pub problems: Vec<String>,
}

impl FixReport {
    /// One part of the on-screen line ("" when nothing was waiting).
    pub fn message(&self) -> String {
        let mut parts = Vec::new();
        if 0 < self.fixed {
            parts.push(format!("{} new from texture_work", self.fixed));
        }
        if let Some(p) = self.problems.first() {
            parts.push(if self.problems.len() == 1 { p.clone() } else { format!("{p} (+{} more)", self.problems.len() - 1) });
        }
        parts.join(", ")
    }
}

const README: &str = "Making textures with ChatGPT (a native feature of this remaster)\r
\r
1. In the game press F10, choose Graphics..., set Save textures to On, and\r
   go where the texture is used (the character select screen, a level).\r
2. Open texture_dump\\index.html beside the game and find the texture.\r
   Note its file name, for example 0edda1314885f6a6.png.\r
3. In ChatGPT, attach that PNG from the texture_dump folder and ask, e.g.:\r
   \"This is a texture from a 2001 PlayStation 2 game. Make a\r
   high-resolution version of exactly this image: the same layout, the\r
   same shapes in the same places, the same colours and lighting, with\r
   finer detail. Do not add objects, text or a border. Keep the same\r
   proportions (width to height).\"\r
   If the page says the texture repeats, add: \"It must tile seamlessly:\r
   the left edge continues into the right edge, the top into the bottom.\"\r
   If it has see-through parts (the checkerboard shows on the page), add:\r
   \"Keep the see-through areas plain black.\"\r
4. Download the image and save it in this folder with the texture's file\r
   name, for example 0edda1314885f6a6.png (0edda1314885f6a6 v2.png works\r
   too). It must be a PNG.\r
5. In the game press F5 (or start the game). It sizes the image to fit,\r
   gives it the original's see-through shape, saves it into texture_pack\r
   and shows it. Your image moves to texture_work\\done.\r
\r
Good for AI: large surfaces (floors, walls, rock, wood), character skins\r
and clothes, item pictures. Less good: tiny textures (8 or 16 pixels), text\r
and HUD icons, effects (fire, sparks, glows) and animation frames (every\r
frame would come out different).\r
\r
To undo a texture: delete its file from texture_pack and press F5.\r
";

/// texture_work and its README.txt, made when missing.
pub fn ensure_work_dir(dir: &Path) {
    if std::fs::create_dir_all(dir).is_ok() {
        let readme = dir.join("README.txt");
        if std::fs::read_to_string(&readme).ok().as_deref() != Some(README) {
            let _ = std::fs::write(readme, README);
        }
    }
}

/// Fit every image in `work` to its texture (the original from `dump`) and
/// write it into `pack`. Textures are fitted on up to 4 threads; several
/// images for one texture go in name order (the last one stays).
pub fn process(work: &Path, dump: &Path, pack: &Path) -> FixReport {
    let mut report = FixReport::default();
    let Ok(dir) = std::fs::read_dir(work) else { return report };
    let mut files: Vec<PathBuf> = dir.filter_map(|e| e.ok()).map(|e| e.path()).filter(|p| p.is_file()).collect();
    files.sort();
    let flags = index_flags(dump);
    let mut problems: Vec<(String, String)> = Vec::new();
    let mut groups: std::collections::BTreeMap<String, Vec<(PathBuf, String)>> = std::collections::BTreeMap::new();
    for path in files {
        let file = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
        if file.eq_ignore_ascii_case("README.txt") {
            continue;
        }
        let Some(name) = texture_name(&file) else {
            problems.push((file.clone(), format!("{file}: name it after its texture")));
            continue;
        };
        if !file.to_ascii_lowercase().ends_with(".png") {
            problems.push((file.clone(), format!("{file}: save it as a PNG")));
            continue;
        }
        groups.entry(name.to_string()).or_default().push((path, file));
    }
    let groups: Vec<(String, Vec<(PathBuf, String)>)> = groups.into_iter().collect();
    let next = std::sync::atomic::AtomicUsize::new(0);
    let threads = std::thread::available_parallelism().map_or(1, |n| n.get()).clamp(1, 4);
    let results: Vec<(String, Result<(), String>)> = std::thread::scope(|scope| {
        let workers: Vec<_> = (0..threads)
            .map(|_| {
                scope.spawn(|| {
                    let mut mine = Vec::new();
                    loop {
                        let g = next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        let Some((name, images)) = groups.get(g) else { return mine };
                        for (path, file) in images {
                            let r = fix_file(path, &dump.join(format!("{name}.png")), flags.get(name).copied().unwrap_or(0), &pack.join(format!("{name}.png")));
                            if r.is_ok() {
                                let done = work.join("done");
                                let _ = std::fs::create_dir_all(&done);
                                let _ = std::fs::rename(path, done.join(file));
                            }
                            mine.push((file.clone(), r));
                        }
                    }
                })
            })
            .collect();
        workers.into_iter().flat_map(|w| w.join().unwrap_or_default()).collect()
    });
    for (file, r) in results {
        match r {
            Ok(()) => report.fixed += 1,
            Err(e) => problems.push((file.clone(), format!("{file}: {e}"))),
        }
    }
    problems.sort();
    report.problems = problems.into_iter().map(|(_, p)| p).collect();
    report
}

/// The texture name a work file starts with (16 hex digits).
fn texture_name(file: &str) -> Option<&str> {
    let name = file.get(..16)?;
    (name.bytes().all(|b| b.is_ascii_hexdigit()) && !file[16..].starts_with(|c: char| c.is_ascii_hexdigit())).then_some(name)
}

/// index.tsv's record flags by texture name.
fn index_flags(dump: &Path) -> HashMap<String, u16> {
    let text = std::fs::read_to_string(dump.join("index.tsv")).unwrap_or_default();
    text.lines()
        .filter_map(|l| {
            let f: Vec<&str> = l.split('\t').collect();
            let [name, _, _, flags] = f[..] else { return None };
            Some((name.to_string(), u16::from_str_radix(flags.trim_start_matches("0x"), 16).ok()?))
        })
        .collect()
}

/// An 8-bit RGBA PNG (any colour type; palettes expanded).
fn read_rgba8(path: &Path) -> Result<(usize, usize, Vec<u8>), String> {
    let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    let mut decoder = png::Decoder::new(std::io::BufReader::new(file));
    decoder.set_transformations(png::Transformations::normalize_to_color8());
    let mut reader = decoder.read_info().map_err(|_| "not a readable PNG".to_string())?;
    let mut buf = vec![0; reader.output_buffer_size().ok_or("too large")?];
    let info = reader.next_frame(&mut buf).map_err(|_| "not a readable PNG".to_string())?;
    let (w, h) = (info.width as usize, info.height as usize);
    if w == 0 || h == 0 || 8192 < w || 8192 < h {
        return Err("too large".into());
    }
    let px = w * h;
    let rgba = match info.color_type {
        png::ColorType::Rgba => buf[..px * 4].to_vec(),
        png::ColorType::Rgb => buf[..px * 3].chunks_exact(3).flat_map(|p| [p[0], p[1], p[2], 255]).collect(),
        png::ColorType::GrayscaleAlpha => buf[..px * 2].chunks_exact(2).flat_map(|p| [p[0], p[0], p[0], p[1]]).collect(),
        png::ColorType::Grayscale => buf[..px].iter().flat_map(|&g| [g, g, g, 255]).collect(),
        png::ColorType::Indexed => return Err("not a readable PNG".into()),
    };
    Ok((w, h, rgba))
}

/// The scale for an image of `iw` × `ih` made for a `w` × `h` texture.
fn scale_for(iw: usize, ih: usize, w: usize, h: usize) -> usize {
    let s = (iw as f32 / w as f32).max(ih as f32 / h as f32).round() as usize;
    let most = (2048 / w.max(h)).clamp(1, 8);
    s.clamp(1, most)
}

/// Catmull-Rom (a = −0.5).
fn cubic(x: f32) -> f32 {
    let x = x.abs();
    if x < 1.0 {
        1.5 * x * x * x - 2.5 * x * x + 1.0
    } else if x < 2.0 {
        -0.5 * x * x * x + 2.5 * x * x - 4.0 * x + 2.0
    } else {
        0.0
    }
}

/// One axis of a separable resample: `lines` lines of `from` texels to
/// `to` texels each, along x (row-major `from` × `lines`) or along y
/// (row-major `lines` × `from`). Edges are clamped.
fn resample_axis(src: &[[f32; 3]], from: usize, to: usize, lines: usize, along_x: bool) -> Vec<[f32; 3]> {
    let ratio = from as f32 / to as f32;
    let width = ratio.max(1.0);
    // Per output texel: (source texel, weight), the weights summing to 1.
    let taps: Vec<Vec<(usize, f32)>> = (0..to)
        .map(|i| {
            let centre = (i as f32 + 0.5) * ratio - 0.5;
            let (first, last) = ((centre - 2.0 * width).floor() as isize, (centre + 2.0 * width).ceil() as isize);
            let mut t: Vec<(usize, f32)> = (first..=last).map(|j| (j.clamp(0, from as isize - 1) as usize, cubic((j as f32 - centre) / width))).filter(|&(_, w)| w != 0.0).collect();
            let sum: f32 = t.iter().map(|&(_, w)| w).sum();
            for tap in &mut t {
                tap.1 /= sum;
            }
            t
        })
        .collect();
    let mut out = vec![[0f32; 3]; to * lines];
    for line in 0..lines {
        for (i, t) in taps.iter().enumerate() {
            let mut acc = [0f32; 3];
            for &(j, wk) in t {
                let s = if along_x { src[line * from + j] } else { src[j * lines + line] };
                for (a, v) in acc.iter_mut().zip(s) {
                    *a += v * wk;
                }
            }
            out[if along_x { line * to + i } else { i * lines + line }] = acc.map(|v| v.clamp(0.0, 1.0));
        }
    }
    out
}

/// `src` (sw × sh) resampled to dw × dh.
fn resample(src: &[[f32; 3]], (sw, sh): (usize, usize), (dw, dh): (usize, usize)) -> Vec<[f32; 3]> {
    let rows = resample_axis(src, sw, dw, sh, true);
    resample_axis(&rows, sh, dh, dw, false)
}

/// The fitted RGBA8 (alpha 255 opaque) of an image for an original (dump
/// RGBA8, `w` × `h`, record `flags`), with its size.
fn fit(image: (usize, usize, &[u8]), original: (usize, usize, &[u8]), flags: u16) -> (usize, usize, Vec<u8>) {
    let (iw, ih, px) = image;
    let (w, h, orig) = original;
    let s = scale_for(iw, ih, w, h);
    let (ow, oh) = (w * s, h * s);
    let rgb: Vec<[f32; 3]> = px.chunks_exact(4).map(|p| [p[0] as f32 / 255.0, p[1] as f32 / 255.0, p[2] as f32 / 255.0]).collect();
    let mut rgb = resample(&rgb, (iw, ih), (ow, oh));
    let a: Vec<f32> = orig.chunks_exact(4).map(|p| p[3] as f32 / 255.0).collect();
    let opaque = orig.chunks_exact(4).all(|p| p[3] == 255);
    let binary = orig.chunks_exact(4).all(|p| p[3] <= 2 || p[3] == 255);
    let wrap = (flags & 0x4 == 0, flags & 0x8 == 0);
    let alpha: Vec<f32> = (0..ow * oh)
        .map(|i| {
            if opaque {
                return 1.0;
            }
            let v = prep::bilinear(&a, (w, h), wrap, s, (i % ow, i / ow));
            if binary { if 0.5 <= v { 1.0 } else { 0.0 } } else { v }
        })
        .collect();
    let q = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    if !opaque {
        // Four rings: bilinear samples reach one texel past the kept ones,
        // and texfilter's levels weight colour by alpha; a full fill at
        // these sizes took about a second per image.
        let kept: Vec<bool> = alpha.iter().map(|&v| 0 < q(v)).collect();
        prep::fill_clear_rings(&mut rgb, &kept, (ow, oh), wrap, 4);
    }
    let out = rgb.iter().zip(&alpha).flat_map(|(c, &a)| [q(c[0]), q(c[1]), q(c[2]), q(a)]).collect();
    (ow, oh, out)
}

fn fix_file(image: &Path, original: &Path, flags: u16, out: &Path) -> Result<(), String> {
    let (ow, oh, orig) = read_rgba8(original).map_err(|_| "its texture is not in texture_dump yet (Save textures, then visit it)".to_string())?;
    let (iw, ih, px) = read_rgba8(image)?;
    let (w, h, rgba) = fit((iw, ih, &px), (ow, oh, &orig), flags);
    if let Some(dir) = out.parent() {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    let tmp = out.with_extension("png.tmp");
    (|| -> Result<(), Box<dyn std::error::Error>> {
        let mut e = png::Encoder::new(std::io::BufWriter::new(std::fs::File::create(&tmp)?), w as u32, h as u32);
        e.set_color(png::ColorType::Rgba);
        e.set_depth(png::BitDepth::Eight);
        let mut writer = e.write_header()?;
        writer.write_image_data(&rgba)?;
        writer.finish()?;
        std::fs::rename(&tmp, out)?;
        Ok(())
    })()
    .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_and_scales() {
        assert_eq!(texture_name("0edda1314885f6a6.png"), Some("0edda1314885f6a6"));
        assert_eq!(texture_name("0edda1314885f6a6 v2.png"), Some("0edda1314885f6a6"));
        assert_eq!(texture_name("0edda1314885f6a6a.png"), None);
        assert_eq!(texture_name("ChatGPT Image.png"), None);
        // A square 1024 image of a 64×32 texture: 8× (most), 512×256.
        assert_eq!(scale_for(1024, 1024, 64, 32), 8);
        assert_eq!(scale_for(256, 256, 64, 64), 4);
        assert_eq!(scale_for(1536, 1024, 512, 512), 3);
        assert_eq!(scale_for(1024, 1024, 1024, 512), 2);
        assert_eq!(scale_for(10, 10, 64, 64), 1);
    }

    /// A flat image stays flat at any size; shrinking 2-texel stripes 4×
    /// averages them (away from the clamped edges); a horizontal ramp keeps
    /// its order and range.
    #[test]
    fn resampling() {
        let grey = vec![[0.25f32, 0.5, 0.75]; 7 * 5];
        assert!(resample(&grey, (7, 5), (20, 9)).iter().all(|c| c.iter().zip([0.25, 0.5, 0.75]).all(|(a, b)| (a - b).abs() < 1e-5)));
        let stripes: Vec<[f32; 3]> = (0..64 * 4).map(|i| if (i % 64) / 2 % 2 == 0 { [1.0; 3] } else { [0.0; 3] }).collect();
        let small = resample(&stripes, (64, 4), (16, 1));
        assert!(small[3..13].iter().all(|c| (c[0] - 0.5).abs() < 0.05), "{small:?}");
        let ramp: Vec<[f32; 3]> = (0..8).map(|x| [x as f32 / 7.0; 3]).collect();
        let big = resample(&ramp, (8, 1), (32, 1));
        assert!(big.windows(2).all(|p| p[0][0] <= p[1][0] + 1e-6) && big[0][0] == 0.0 && big[31][0] == 1.0, "{big:?}");
    }

    /// A 2×2 cut-out (kept texels on one diagonal) and a solid red 16×16 AI
    /// image: 8× (16×16), the original's mask back, the clear texels red
    /// (not black); an opaque original stays opaque.
    #[test]
    fn fit_restores_the_original_mask() {
        let orig = [0u8, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255];
        let red = [255u8, 0, 0, 255].repeat(16 * 16);
        let (w, h, out) = fit((16, 16, &red), (2, 2, &orig), 0xc);
        assert_eq!((w, h), (16, 16));
        let a = |x: usize, y: usize| out[(y * 16 + x) * 4 + 3];
        assert_eq!((a(0, 0), a(15, 15), a(15, 0), a(0, 15)), (255, 255, 0, 0));
        assert!(out.chunks_exact(4).all(|p| p[..3] == [255, 0, 0]));
        let opaque = [9u8, 9, 9, 255].repeat(4);
        let (_, _, out) = fit((16, 16, &red), (2, 2, &opaque), 0);
        assert!(out.chunks_exact(4).all(|p| p[3] == 255));
    }

    /// The folder pass: a fitted file in the pack and the image in done; a
    /// missing original and a non-PNG stay with their reasons.
    #[test]
    fn process_the_work_folder() {
        let root = std::env::temp_dir().join(format!("gdl-texfix-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let (work, dump, pack) = (root.join("work"), root.join("dump"), root.join("pack"));
        ensure_work_dir(&work);
        std::fs::create_dir_all(&dump).unwrap();
        let png = |path: &Path, w: u32, h: u32, rgba: &[u8]| {
            let mut e = png::Encoder::new(std::io::BufWriter::new(std::fs::File::create(path).unwrap()), w, h);
            e.set_color(png::ColorType::Rgba);
            e.set_depth(png::BitDepth::Eight);
            e.write_header().unwrap().write_image_data(rgba).unwrap();
        };
        png(&dump.join("00000000000000aa.png"), 4, 2, &[1, 2, 3, 255].repeat(8));
        std::fs::write(dump.join("index.tsv"), "00000000000000aa\tFLOOR\t4x2\t0x0\n").unwrap();
        png(&work.join("00000000000000aa v2.png"), 64, 64, &[200, 100, 50, 255].repeat(64 * 64));
        png(&work.join("00000000000000bb.png"), 8, 8, &[0; 256]);
        std::fs::write(work.join("00000000000000cc.jpg"), b"x").unwrap();
        let r = process(&work, &dump, &pack);
        assert_eq!(r.fixed, 1);
        assert_eq!(r.problems.len(), 2, "{r:?}");
        assert!(r.problems.iter().any(|p| p.starts_with("00000000000000bb.png: its texture is not in texture_dump")));
        assert!(r.problems.iter().any(|p| p == "00000000000000cc.jpg: save it as a PNG"));
        let (w, h, out) = read_rgba8(&pack.join("00000000000000aa.png")).unwrap();
        assert_eq!((w, h), (32, 16));
        assert!(out.chunks_exact(4).all(|p| p == [200, 100, 50, 255]));
        assert!(work.join("done").join("00000000000000aa v2.png").is_file() && !work.join("00000000000000aa v2.png").exists());
        assert!(work.join("README.txt").is_file());
        assert_eq!(r.message(), "1 new from texture_work, 00000000000000bb.png: its texture is not in texture_dump yet (Save textures, then visit it) (+1 more)");
        std::fs::remove_dir_all(&root).unwrap();
    }
}
