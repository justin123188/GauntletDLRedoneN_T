//! NATIVE ENHANCEMENT (M19): texture replacements, not part of the original
//! game. A game texture is named by the first 16 hex digits of SHA-256
//! over its width and height (u16 little-endian) and its decoded level-0
//! RGBA as the client holds it (GS alpha: 0x80 opaque). GDL_TEXTURE_DUMP=DIR
//! writes each texture the client decodes as DIR/<name>.png (alpha doubled
//! to 0..255) and a line `<name>\t<label>\t<width>x<height>\t<flags>` in
//! DIR/index.tsv (the TEXDEF label and the record's flags, 0x4/0x8 clamp
//! U/V: research tooling); a texture pack (Settings::texture_pack) replaces a texture
//! with PACK/<name>.png of any size (alpha halved back, one mip level).
//! The generated HD pack (hd::generate, the same names and format) comes
//! after the user's pack: their replacements win. Lightmaps are not
//! replaced. Without a pack or a dump, nothing is hashed.
//!
//! NATIVE (this fork, decisions/0009-texture-workflow.md): the texture_pack
//! folder beside the game is the pack unless a path is set
//! (Settings::texture_pack_folder), and saving starts and stops while the
//! game runs (the overlay's Save textures row: texture_dump beside the
//! game). A dump also records where each texture was loaded from:
//! DIR/sources.tsv, `<name>\t<kind>\t<source>` (kind level, character,
//! package or hud, from the caller's `set_source`), one line per new pair.
//! index.tsv keeps its four columns.
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Mutex, RwLock};

struct Config {
    pack: Option<PathBuf>,
    generated: Option<PathBuf>,
    dump: Option<PathBuf>,
}

static CONFIG: RwLock<Config> = RwLock::new(Config { pack: None, generated: None, dump: None });
/// The kind and name of the textures being prepared (sources.tsv).
static SOURCE: RwLock<(String, String)> = RwLock::new((String::new(), String::new()));
/// Holds the dump's writes one at a time (texture preparation runs on
/// worker threads, and saving may start during it), with the lines
/// sources.tsv already has.
static DUMP: Mutex<Option<HashSet<String>>> = Mutex::new(None);
/// Textures saved (new files) since startup.
static SAVED: AtomicUsize = AtomicUsize::new(0);
/// Lines the dump has written since startup (index.tsv and sources.tsv:
/// texture_browser's index.html follows them).
static WRITTEN: AtomicUsize = AtomicUsize::new(0);

/// At startup: the user's pack directory, and the dump directory from
/// GDL_TEXTURE_DUMP.
pub fn configure(pack: Option<PathBuf>) {
    if let Ok(mut c) = CONFIG.write() {
        c.pack = pack.filter(|p| p.is_dir());
    }
    set_dump(std::env::var_os("GDL_TEXTURE_DUMP").map(PathBuf::from));
}

/// Start saving the textures decoded from now on to `dir` (made when
/// missing), or stop (None). Whether saving is on afterwards.
pub fn set_dump(dir: Option<PathBuf>) -> bool {
    let dir = dir.filter(|d| std::fs::create_dir_all(d).is_ok() && d.is_dir());
    let seen = dir.as_deref().map(seen_sources);
    if let Ok(mut s) = DUMP.lock() {
        *s = seen;
    }
    let on = dir.is_some();
    if let Ok(mut c) = CONFIG.write() {
        c.dump = dir;
    }
    on
}

/// The lines a dump directory's sources.tsv holds.
fn seen_sources(dir: &Path) -> HashSet<String> {
    std::fs::read_to_string(dir.join("sources.tsv")).map(|t| t.lines().map(str::to_string).collect()).unwrap_or_default()
}

/// Where the textures prepared next come from (sources.tsv): `kind` is
/// level, character, package or hud.
pub fn set_source(kind: &str, name: &str) {
    let clean = |s: &str| s.replace(['\t', '\r', '\n'], " ");
    if let Ok(mut s) = SOURCE.write() {
        *s = (clean(kind), clean(name));
    }
}

/// The game's folder (the executable's: an installation's).
pub fn game_dir() -> Option<PathBuf> {
    Some(std::env::current_exe().ok()?.parent()?.to_path_buf())
}

/// The texture_pack folder beside the game.
pub fn default_pack_dir() -> Option<PathBuf> {
    Some(game_dir()?.join("texture_pack"))
}

/// The texture_dump folder beside the game (the Save textures row).
pub fn default_dump_dir() -> Option<PathBuf> {
    Some(game_dir()?.join("texture_dump"))
}

/// The texture_work folder beside the game (texture_fix: images to fit).
pub fn default_work_dir() -> Option<PathBuf> {
    Some(game_dir()?.join("texture_work"))
}

/// The pack folder, made when missing, with its README.txt.
pub fn ensure_pack_dir(dir: &Path) {
    if std::fs::create_dir_all(dir).is_ok() {
        let readme = dir.join("README.txt");
        if std::fs::read_to_string(&readme).ok().as_deref() != Some(PACK_README) {
            let _ = std::fs::write(readme, PACK_README);
        }
    }
}

const PACK_README: &str = "Replacement textures (a native feature of this remaster)\r
\r
PNG files in this folder replace the game's textures. A file must have the\r
name of the texture it replaces, as in the texture_dump folder (for example\r
0edda1314885f6a6.png). Any size up to 4096 x 4096 works. The game reads\r
this folder when it starts, and again when you press F5 in play.\r
\r
To get the game's textures: press F10, choose Graphics..., and set Save\r
textures to On. Every texture the game loads from then on is saved to the\r
texture_dump folder beside this one. Open index.html there to browse them\r
by character, level and label.\r
\r
Images made with ChatGPT go in the texture_work folder instead: the game\r
fits them and puts the result here. See texture_work\\README.txt.\r
\r
Material detail (F10, Graphics): bump and shine on the textures. Each\r
texture has a surface type, guessed from its label. To set one, add a line\r
to materials.tsv in this folder: the file name, then the type, for example\r
    0edda1314885f6a6 stone\r
Types: stone, wood, metal, ground, grass, foliage, water, lava, ice, gem,\r
cloth, skin, auto, none (none: no detail). A line can also set\r
bump=1.5 (0 flat .. 4), rough=0.4 (0 shiny .. 1 matte) and metal=1.\r
Your own maps go beside the texture, any size:\r
    0edda1314885f6a6_n.png   normal map (OpenGL style: green up)\r
    0edda1314885f6a6_h.png   height (white high), the normal is made from it\r
    0edda1314885f6a6_r.png   roughness (white matte, black shiny)\r
    0edda1314885f6a6_m.png   metal (white metal, black not)\r
What is missing is made from the texture itself. F5 in play reloads them.\r
";

/// The user's pack in use (None: none).
pub fn pack_dir() -> Option<PathBuf> {
    CONFIG.read().ok()?.pack.clone()
}

/// Where textures are being saved (None: not saving).
pub fn dump_dir() -> Option<PathBuf> {
    CONFIG.read().ok()?.dump.clone()
}

/// Textures saved since startup.
pub fn saved() -> usize {
    SAVED.load(Ordering::Relaxed)
}

/// Index and sources lines written since startup.
pub fn written() -> usize {
    WRITTEN.load(Ordering::Relaxed)
}

/// The generated HD pack in use (None: off); textures decoded from then on
/// (the next level's) take it.
pub fn set_generated(dir: Option<PathBuf>) {
    if let Ok(mut c) = CONFIG.write() {
        c.generated = dir.filter(|p| p.is_dir());
    }
}

/// Whether a pack or a dump is configured.
pub fn active() -> bool {
    CONFIG.read().is_ok_and(|c| c.pack.is_some() || c.generated.is_some() || c.dump.is_some())
}

/// Whether a texture dump is configured (replace writes its files).
pub fn dumping() -> bool {
    CONFIG.read().is_ok_and(|c| c.dump.is_some())
}

/// A texture's file name stem.
pub fn name(width: u16, height: u16, rgba: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(width.to_le_bytes());
    h.update(height.to_le_bytes());
    h.update(rgba);
    h.finalize().iter().take(8).map(|b| format!("{b:02x}")).collect()
}

/// The replacement (width, height, RGBA with GS alpha) for a decoded
/// texture, after writing it (with its TEXDEF `label` and record `flags`)
/// to the dump directory when one is set.
pub fn replace(width: u16, height: u16, rgba: &[u8], label: &str, flags: u16) -> Option<(u32, u32, Vec<u8>)> {
    lookup(width, height, rgba, label, flags)?.file.map(|(_, w, h, px)| (w, h, px))
}

/// NATIVE (this fork, texture_reload): `replace`'s result with the
/// texture's name and the file it came from (None while no pack or dump
/// is set).
pub struct Lookup {
    pub name: String,
    pub file: Option<(PathBuf, u32, u32, Vec<u8>)>,
}

/// `replace`, keeping the name and the file (texture_reload registers them).
pub fn lookup(width: u16, height: u16, rgba: &[u8], label: &str, flags: u16) -> Option<Lookup> {
    let cfg = CONFIG.read().ok()?;
    if cfg.pack.is_none() && cfg.generated.is_none() && cfg.dump.is_none() {
        return None;
    }
    let stem = name(width, height, rgba);
    if let Some(dir) = &cfg.dump {
        dump(dir, &stem, width, height, rgba, label, flags);
    }
    let file = [&cfg.pack, &cfg.generated].into_iter().flatten().find_map(|dir| {
        let path = dir.join(format!("{stem}.png"));
        read_png(&path).map(|(w, h, px)| (path, w, h, px))
    });
    Some(Lookup { name: stem, file })
}

/// NATIVE (this fork, texture_reload): the file that replaces texture
/// `name` now (the user's pack first, then the generated pack).
pub fn find(name: &str) -> Option<PathBuf> {
    let cfg = CONFIG.read().ok()?;
    [&cfg.pack, &cfg.generated].into_iter().flatten().map(|dir| dir.join(format!("{name}.png"))).find(|p| p.is_file())
}

/// One texture into the dump: its file and index.tsv line when new, and
/// its sources.tsv line when its (name, source) pair is new.
fn dump(dir: &Path, stem: &str, width: u16, height: u16, rgba: &[u8], label: &str, flags: u16) {
    use std::io::Write;
    let Ok(mut seen) = DUMP.lock() else { return };
    let path = dir.join(format!("{stem}.png"));
    if !path.exists() && write_png(&path, width.into(), height.into(), rgba).is_ok() {
        SAVED.fetch_add(1, Ordering::Relaxed);
        WRITTEN.fetch_add(1, Ordering::Relaxed);
        let index = std::fs::OpenOptions::new().create(true).append(true).open(dir.join("index.tsv"));
        let _ = index.and_then(|mut f| writeln!(f, "{stem}\t{label}\t{width}x{height}\t{flags:#x}"));
    }
    let Some((kind, source)) = SOURCE.read().ok().map(|s| s.clone()).filter(|s| !s.0.is_empty()) else { return };
    let line = format!("{stem}\t{kind}\t{source}");
    if path.exists() && seen.get_or_insert_with(HashSet::new).insert(line.clone()) {
        WRITTEN.fetch_add(1, Ordering::Relaxed);
        let sources = std::fs::OpenOptions::new().create(true).append(true).open(dir.join("sources.tsv"));
        let _ = sources.and_then(|mut f| writeln!(f, "{line}"));
    }
}

pub(crate) fn write_png(path: &Path, width: u32, height: u32, rgba: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    let data: Vec<u8> = rgba.chunks_exact(4).flat_map(|p| [p[0], p[1], p[2], (u16::from(p[3]) * 2).min(255) as u8]).collect();
    let tmp = path.with_extension("png.tmp");
    let mut encoder = png::Encoder::new(std::io::BufWriter::new(std::fs::File::create(&tmp)?), width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header()?;
    writer.write_image_data(&data)?;
    writer.finish()?;
    std::fs::rename(tmp, path)?;
    Ok(())
}

/// A PNG as RGBA with GS alpha (8-bit, any colour type; at most 4096²).
pub(crate) fn read_png(path: &Path) -> Option<(u32, u32, Vec<u8>)> {
    let file = std::fs::File::open(path).ok()?;
    let mut decoder = png::Decoder::new(std::io::BufReader::new(file));
    decoder.set_transformations(png::Transformations::normalize_to_color8());
    let mut reader = decoder.read_info().ok()?;
    let mut buf = vec![0; reader.output_buffer_size()?];
    let info = reader.next_frame(&mut buf).ok()?;
    if info.width == 0 || info.height == 0 || 4096 < info.width || 4096 < info.height {
        return None;
    }
    let px = (info.width * info.height) as usize;
    let rgba: Vec<u8> = match info.color_type {
        png::ColorType::Rgba => buf[..px * 4].to_vec(),
        png::ColorType::Rgb => buf[..px * 3].chunks_exact(3).flat_map(|p| [p[0], p[1], p[2], 255]).collect(),
        png::ColorType::GrayscaleAlpha => buf[..px * 2].chunks_exact(2).flat_map(|p| [p[0], p[0], p[0], p[1]]).collect(),
        png::ColorType::Grayscale => buf[..px].iter().flat_map(|&g| [g, g, g, 255]).collect(),
        png::ColorType::Indexed => return None,
    };
    let gs = rgba.chunks_exact(4).flat_map(|p| [p[0], p[1], p[2], ((u16::from(p[3]) * 128 + 127) / 255) as u8]).collect();
    Some((info.width, info.height, gs))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn png_round_trip_keeps_gs_alpha() {
        let dir = std::env::temp_dir().join(format!("gdl-texpack-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let rgba = vec![10, 20, 30, 0x80, 40, 50, 60, 0, 70, 80, 90, 0x40, 1, 2, 3, 0x7f];
        let path = dir.join(format!("{}.png", name(2, 2, &rgba)));
        write_png(&path, 2, 2, &rgba).unwrap();
        let (w, h, back) = read_png(&path).unwrap();
        std::fs::remove_dir_all(&dir).unwrap();
        assert_eq!((w, h, back), (2, 2, rgba));
    }

    #[test]
    fn names_depend_on_size_and_texels() {
        let a = name(2, 2, &[0; 16]);
        assert_eq!(a.len(), 16);
        assert_ne!(a, name(4, 1, &[0; 16]));
        assert_ne!(a, name(2, 2, &[1; 16]));
    }

    /// The dump writes a texture once, its index line once, and one
    /// sources line per (texture, source) pair, also after saving starts
    /// again (sources.tsv read back). The dump's own state only: the
    /// global configuration stays off for the other tests.
    #[test]
    fn dump_records_each_source_once() {
        let dir = std::env::temp_dir().join(format!("gdl-texdump-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let restart = || *DUMP.lock().unwrap() = Some(seen_sources(&dir));
        restart();
        let rgba = [1u8, 2, 3, 0x80];
        let stem = name(1, 1, &rgba);
        let lines = |f: &str| std::fs::read_to_string(dir.join(f)).unwrap_or_default().lines().map(str::to_string).collect::<Vec<_>>();
        set_source("level", "L1 The Tower");
        dump(&dir, &stem, 1, 1, &rgba, "FLOOR", 0x80);
        dump(&dir, &stem, 1, 1, &rgba, "FLOOR", 0x80);
        set_source("character", "WIZ\tY");
        dump(&dir, &stem, 1, 1, &rgba, "FLOOR", 0x80);
        restart();
        dump(&dir, &stem, 1, 1, &rgba, "FLOOR", 0x80);
        assert_eq!(lines("index.tsv"), [format!("{stem}\tFLOOR\t1x1\t0x80")]);
        assert_eq!(lines("sources.tsv"), [format!("{stem}\tlevel\tL1 The Tower"), format!("{stem}\tcharacter\tWIZ Y")]);
        *DUMP.lock().unwrap() = None;
        set_source("", "");
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
