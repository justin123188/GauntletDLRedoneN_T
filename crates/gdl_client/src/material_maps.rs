//! NATIVE (this fork, decisions/0015-material-detail.md; not part of the
//! original game): material detail. While Settings::material_detail is on,
//! every opaque or cut-out base texture gets a surface type and a detail
//! map, drawn by gs.wgsl with the live lights (the game's point lights,
//! torch lights, the level's sun direction).
//!
//! The detail map is one RGBA8 image sampled with the base texture's UVs:
//! R and G the tangent-space normal (OpenGL convention: green towards the
//! image's top), B the roughness, A the metalness. It comes from files in
//! the texture pack beside the texture's own (`<name>_n.png` normal,
//! `<name>_h.png` height, `<name>_r.png` roughness, `<name>_m.png` metal,
//! any size) or, for what is missing, is generated from the texture: a
//! height from its brightness, then the normal and roughness shaped by the
//! surface type.
//!
//! The type comes from `materials.tsv` in the pack (`<name> <type>
//! [bump=… rough=… metal=…]`, `#` starts a comment), or is guessed from
//! the texture's TEXDEF label words (BARN_WALL: wood). Type "none" gives a
//! flat map: the texture draws as without detail. Blended textures (glows,
//! smoke, most effects) get none. F5 reloads the table and the map files.
use crate::texture_reload::Applied;
use bevy::asset::{AssetId, Assets};
use bevy::image::{Image, ImageAddressMode, ImageSampler, ImageSamplerDescriptor};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, RwLock};

/// A surface type.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Kind {
    None,
    Auto,
    Stone,
    Wood,
    Metal,
    Ground,
    Grass,
    Foliage,
    Water,
    Lava,
    Ice,
    Gem,
    Cloth,
    Skin,
}

pub const KINDS: [Kind; 14] = [
    Kind::None,
    Kind::Auto,
    Kind::Stone,
    Kind::Wood,
    Kind::Metal,
    Kind::Ground,
    Kind::Grass,
    Kind::Foliage,
    Kind::Water,
    Kind::Lava,
    Kind::Ice,
    Kind::Gem,
    Kind::Cloth,
    Kind::Skin,
];

/// How a type's map is generated: bump strength (0: flat), roughness
/// (0 mirror-smooth .. 1 matte) and metalness (0..1).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Params {
    pub bump: f32,
    pub rough: f32,
    pub metal: f32,
}

impl Kind {
    pub fn name(self) -> &'static str {
        match self {
            Kind::None => "none",
            Kind::Auto => "auto",
            Kind::Stone => "stone",
            Kind::Wood => "wood",
            Kind::Metal => "metal",
            Kind::Ground => "ground",
            Kind::Grass => "grass",
            Kind::Foliage => "foliage",
            Kind::Water => "water",
            Kind::Lava => "lava",
            Kind::Ice => "ice",
            Kind::Gem => "gem",
            Kind::Cloth => "cloth",
            Kind::Skin => "skin",
        }
    }

    pub fn parse(s: &str) -> Option<Kind> {
        KINDS.iter().copied().find(|k| k.name().eq_ignore_ascii_case(s.trim()))
    }

    /// NATIVE values chosen by eye (frozen frames of G1 and the Tower).
    /// Water's ripples and lava's glow are later steps (models-plan.md M2):
    /// here they are only smooth or flat.
    pub fn params(self) -> Params {
        let p = |bump, rough, metal| Params { bump, rough, metal };
        match self {
            Kind::None => p(0.0, 1.0, 0.0),
            Kind::Auto => p(0.5, 0.75, 0.0),
            Kind::Stone => p(1.0, 0.7, 0.0),
            Kind::Wood => p(0.7, 0.7, 0.0),
            Kind::Metal => p(0.45, 0.35, 1.0),
            Kind::Ground => p(0.8, 0.85, 0.0),
            Kind::Grass => p(0.5, 0.8, 0.0),
            Kind::Foliage => p(0.35, 0.6, 0.0),
            Kind::Water => p(0.0, 0.1, 0.0),
            Kind::Lava => p(0.3, 0.6, 0.0),
            Kind::Ice => p(0.3, 0.15, 0.0),
            // Gems and crystals: nearly mirror-smooth, small sharp glints
            // (the owner asked for them, 2026-11-22; sparkle is a later
            // effects step).
            Kind::Gem => p(0.2, 0.08, 0.0),
            Kind::Cloth => p(0.4, 0.9, 0.0),
            Kind::Skin => p(0.15, 0.6, 0.0),
        }
    }
}

/// Label words of each type, tried in this order (effects and skies
/// first). The label is split into runs of letters (BARN_WALL is BARN and
/// WALL; SKYWOODWALLSHARP stays one run). A word of four letters or more
/// matches anywhere in a run, a shorter one at its start; `^` marks a word
/// that must start or end a run (FERN, not INFERNO), `=` one that must be
/// the whole run (SKY, a realm's name inside many labels). `*` one that
/// matches anywhere, effects too (HORIZON: the skies). Effect words
/// always start or end a run: HIGHLIGHTS is no light.
const WORDS: &[(Kind, &[&str])] = &[
    (
        Kind::None,
        &[
            "FIRE", "FLAME", "GLOW", "SPARK", "SMOKE", "LIGHT", "RAY", "BEAM", "MAGIC", "EXPLO", "EXP", "FX", "SFX", "SHADOW", "FLASH", "AURA", "HALO",
            "TRAIL", "STREAK", "BLUR", "LNING", "ELEC", "PARTICLE", "MIST", "^FOG", "CLOUD", "=SKY", "CLEARSKY", "*HORIZON", "STAR", "SHINE", "SHEEN",
            "GLINT", "BLAST", "SPLASH", "BUBBLE", "STEAM", "DUST", "EMBER", "ZAP", "BOLT", "FONT", "BUTTON", "ICON", "HUD", "PORTAL", "SWIRL",
            "VORTEX", "TORCH", "CANDLE", "LAMP", "^GAS", "FORCEFIELD", "SUN", "XRAY", "XLU", "TRANSITION", "SCORE", "POINTER",
        ],
    ),
    // A wall at the water's edge (C1's BASIC_WALL_WATEREDGE) is stone.
    (Kind::Stone, &["WATEREDGE"]),
    (Kind::Water, &["WATER", "WATR", "WAVE", "RAPID", "RIVER", "OCEAN", "FOAM", "RIPPLE", "POND", "LAKE", "SWAMP", "BOG"]),
    (Kind::Lava, &["LAVA", "MAGMA", "MOLTEN"]),
    (Kind::Gem, &["^GEM", "JEWEL", "RUBY", "EMERALD", "SAPPHIRE", "DIAMOND", "CRYSTAL", "AMETHYST", "TOPAZ", "PEARL", "^SHARD", "^OPAL"]),
    (Kind::Ice, &["ICE", "FROST", "FROZEN", "GLACIER", "GLASS"]),
    (
        Kind::Metal,
        &[
            "METAL", "^IRON", "STEEL", "GOLD", "SILVER", "BRONZE", "COPPER", "BRASS", "PEWTER", "CHAIN", "GRATE", "ARMOR", "ARMOUR", "SWORD", "AXE",
            "BLADE", "SHIELD", "HELM", "CHROME", "RIVET", "PIPE", "GEAR", "COG", "TURBINE", "CAGE", "SPIKE", "KNIFE", "DAGGER", "HAMMER", "ANVIL",
            "COIN", "KEY", "BRAZIER", "ARROWTIP",
        ],
    ),
    (
        Kind::Wood,
        &[
            "WOOD", "PLANK", "BARN", "CRATE", "BARREL", "FENCE", "LOG", "BARK", "TRUNK", "TABLE", "CHAIR", "BOARD", "TIMBER", "DOOR", "SHELF", "CART",
            "WAGON", "WHEEL", "STUMP", "TROUGH", "TREASR", "TREASURE", "CHEST", "CROSSBOW", "CRSBOW", "=BOW", "SHAFT", "DOCK",
        ],
    ),
    (
        Kind::Foliage,
        &[
            "LEAF", "LEAVES", "^TREE", "^BUSH", "PLANT", "^FERN", "^VINE", "IVY", "WEED", "FLOWER", "HEDGE", "SHRUB", "BRANCH", "FOLIAGE", "^PALM",
            "^REED", "CACTUS", "=PINE", "PINETREE", "PINECONE", "OAK", "FROND", "GROWTH", "CORN", "STALK", "WILLOW", "CYPRESS",
        ],
    ),
    (
        Kind::Stone,
        &[
            "STONE", "^ROCK", "BRICK", "BLOCK", "COBBLE", "MARBLE", "MARB", "PILLAR", "COLUMN", "STATUE", "GARGOYLE", "GARG", "^TILE", "CLIFF", "CAVE",
            "BOULDER", "SLATE", "SLAB", "GRANITE", "MASON", "CEMENT", "CONCRETE", "STAIR", "STEP", "^ARCH", "BUTRESS", "BUTTRESS", "CASTLE", "TOMB",
            "CRYPT", "RUIN", "WALL", "FLOOR", "FOUNT", "ROOF", "SHINGLE", "GATEPOST", "RAILING",
        ],
    ),
    (Kind::Grass, &["GRASS", "LAWN", "MOSS", "TURF", "MEADOW", "=HAY", "HAYPILE", "HAYSTACK", "STRAW"]),
    (Kind::Ground, &["DIRT", "MUD", "^SAND", "GROUND", "EARTH", "^PATH", "^ROAD", "SOIL", "GRAVEL", "ASH", "SNOW"]),
    (
        Kind::Cloth,
        &[
            "CLOTH", "^CAPE", "ROBE", "BANNER", "^FLAG", "RUG", "CARPET", "TAPESTRY", "CURTAIN", "SACK", "BAG", "^TENT", "CLOAK", "CLOACK", "DRESS",
            "^HOOD", "SHIRT", "PANTS", "LEATHER", "^BELT", "^BOOT", "GLOVE", "=FUR", "^HAIR", "BEARD", "^ROPE", "SLEEVE", "SKIRT", "FLAP", "COLLAR",
            "LOIN", "^HAT", "HATBRIM", "PAPER", "PAGE",
        ],
    ),
    (
        Kind::Skin,
        &[
            "SKIN", "^FACE", "FLESH", "^BODY", "TORSO", "ARM", "LEG", "^HAND", "^HEAD", "NECK", "SHOULDER", "THIGH", "CALF", "FINGER", "MOUTH",
            "^NOSE", "JAW", "TEETH", "FANG", "CLAW",
        ],
    ),
];

/// Whether `word` (with its `^` / `=` mark) matches the run `part`.
fn word_matches(part: &str, word: &str, edges_only: bool) -> bool {
    if let Some(w) = word.strip_prefix('=') {
        return part == w;
    }
    if let Some(w) = word.strip_prefix('*') {
        return part.contains(w);
    }
    let (w, edges, marked) = match word.strip_prefix('^') {
        Some(w) => (w, true, true),
        None => (word, edges_only, false),
    };
    if w.len() < 4 && !marked {
        return part.starts_with(w);
    }
    if edges { part.starts_with(w) || part.ends_with(w) } else { part.contains(w) }
}

/// The type a TEXDEF label suggests (Auto when no word matches).
pub fn guess(label: &str) -> Kind {
    let upper = label.to_ascii_uppercase();
    let parts: Vec<&str> = upper.split(|c: char| !c.is_ascii_alphabetic()).filter(|p| !p.is_empty()).collect();
    for (kind, words) in WORDS {
        let edges_only = *kind == Kind::None;
        if parts.iter().any(|p| words.iter().any(|w| word_matches(p, w, edges_only))) {
            return *kind;
        }
    }
    Kind::Auto
}

/// One materials.tsv line: the type and the values it sets.
#[derive(Clone, Debug, PartialEq)]
pub struct Line {
    pub kind: Kind,
    pub bump: Option<f32>,
    pub rough: Option<f32>,
    pub metal: Option<f32>,
}

impl Line {
    pub fn params(&self) -> Params {
        let d = self.kind.params();
        Params { bump: self.bump.unwrap_or(d.bump), rough: self.rough.unwrap_or(d.rough), metal: self.metal.unwrap_or(d.metal) }
    }
}

/// materials.tsv: `<name> <type> [bump=… rough=… metal=…]`, separated by
/// tabs or spaces, `#` to the line's end a comment. Lines whose name or
/// type is not understood are counted, not used.
pub fn parse_table(text: &str) -> (HashMap<String, Line>, usize) {
    parse_named(text, |name| name.len() == 16 && name.chars().all(|c| c.is_ascii_hexdigit()), true)
}

/// The same lines with the names `ok` accepts, lowercased or kept as
/// written: a model pack package's materials.tsv names glTF materials
/// (decisions/0029).
pub fn parse_named(text: &str, ok: impl Fn(&str) -> bool, lowercase: bool) -> (HashMap<String, Line>, usize) {
    let mut out = HashMap::new();
    let mut bad = 0;
    for raw in text.lines() {
        let line = raw.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        let mut words = line.split_whitespace();
        let (Some(name), Some(kind)) = (words.next(), words.next().and_then(Kind::parse)) else {
            bad += 1;
            continue;
        };
        if !ok(name) {
            bad += 1;
            continue;
        }
        let mut entry = Line { kind, bump: None, rough: None, metal: None };
        for w in words {
            let Some((k, v)) = w.split_once('=') else { continue };
            let Ok(v) = v.parse::<f32>() else { continue };
            if !v.is_finite() {
                continue;
            }
            match k.to_ascii_lowercase().as_str() {
                "bump" => entry.bump = Some(v.clamp(0.0, 4.0)),
                "rough" => entry.rough = Some(v.clamp(0.02, 1.0)),
                "metal" => entry.metal = Some(v.clamp(0.0, 1.0)),
                _ => {}
            }
        }
        out.insert(if lowercase { name.to_ascii_lowercase() } else { name.to_string() }, entry);
    }
    (out, bad)
}

static ENABLED: AtomicBool = AtomicBool::new(false);
/// The user's pack folder and its materials.tsv.
static TABLE: RwLock<Option<(PathBuf, HashMap<String, Line>)>> = RwLock::new(None);

/// Whether textures prepared from now on get detail maps
/// (Settings::material_detail; the next level's).
pub fn set_enabled(on: bool) {
    ENABLED.store(on, Ordering::Relaxed);
}

pub fn enabled() -> bool {
    ENABLED.load(Ordering::Relaxed)
}

/// Read `pack`/materials.tsv (at startup and on F5; None: no pack). The
/// number of lines not understood.
pub fn load_table(pack: Option<&Path>) -> usize {
    let (table, bad) = match pack {
        Some(dir) => {
            let (t, bad) = parse_table(&std::fs::read_to_string(dir.join(TABLE_FILE)).unwrap_or_default());
            (Some((dir.to_path_buf(), t)), bad)
        }
        None => (None, 0),
    };
    if let Ok(mut t) = TABLE.write() {
        *t = table;
    }
    bad
}

pub const TABLE_FILE: &str = "materials.tsv";

/// A texture's type: its materials.tsv line, or the label's guess (the
/// bool: guessed).
pub fn choose(name: &str, label: &str) -> (Line, bool) {
    if let Some(line) = TABLE.read().ok().and_then(|t| t.as_ref().and_then(|(_, m)| m.get(name).cloned())) {
        return (line, false);
    }
    (Line { kind: guess(label), bump: None, rough: None, metal: None }, true)
}

/// The map files beside a texture in the pack: normal, height, roughness,
/// metal.
const SUFFIXES: [&str; 4] = ["_n", "_h", "_r", "_m"];

fn map_path(name: &str, k: usize) -> Option<PathBuf> {
    let dir = TABLE.read().ok()?.as_ref()?.0.clone();
    Some(dir.join(format!("{name}{}.png", SUFFIXES[k])))
}

/// What a texture's detail map was made from (F5 compares it): its table
/// line, its map files and the file replacing the texture itself.
#[derive(Clone, Debug, PartialEq)]
pub struct Inputs {
    line: Option<Line>,
    maps: [Option<Applied>; 4],
    base: Option<Applied>,
}

fn inputs(name: &str) -> Inputs {
    let line = TABLE.read().ok().and_then(|t| t.as_ref().and_then(|(_, m)| m.get(name).cloned()));
    let maps = std::array::from_fn(|k| map_path(name, k).filter(|p| p.is_file()).and_then(|p| Applied::of(&p)));
    let base = crate::texture_pack::find(name).and_then(|p| Applied::of(&p));
    Inputs { line, maps, base }
}

/// A texture as the detail map is made from it: level 0 as drawn (GS
/// alpha, 0x80 opaque), and its size against the disc record's (an HD
/// or replaced texture's texels are finer).
pub struct Source<'a> {
    pub name: &'a str,
    pub label: &'a str,
    pub width: u32,
    pub height: u32,
    pub rgba: &'a [u8],
    pub record_width: u32,
    /// The record's flags (0x4 / 0x8: U / V clamp, otherwise it repeats).
    pub flags: u16,
}

/// A detail map: its levels (level 0 first, down to 1×1) and what it was
/// made from.
pub struct Detail {
    pub width: u32,
    pub height: u32,
    pub levels: Vec<Vec<u8>>,
    pub kind: Kind,
    pub guessed: bool,
    pub inputs: Inputs,
}

/// The largest detail map side: a bigger texture's height is taken from
/// a smaller copy (memory: a 4096² map with its chain is 85 MiB).
pub const MAX_SIDE: u32 = 1024;
/// Generated slope per unit of brightness change per texel, at bump 1 on
/// the disc's own texel size (NATIVE value, by eye).
const BUMP_SCALE: f32 = 4.0;

/// The flat map (type none): draws as without detail.
const FLAT: [u8; 4] = [128, 128, 255, 0];

/// DEBUG GDL_DEBUG_MATERIALS=1: each texture's name, label, type, whether
/// it was guessed and its alpha class (0 opaque, 1 cut-out, 2 blended; "-"
/// for a texture-modifier target without texels) on stderr.
pub fn report(name: &str, label: &str, kind: Kind, guessed: bool, class: &str) {
    if std::env::var_os("GDL_DEBUG_MATERIALS").is_some() {
        eprintln!("MATERIAL {name} {label} {} {} {class}", kind.name(), if guessed { "guess" } else { "table" });
    }
}

static WATER: AtomicBool = AtomicBool::new(false);

/// NATIVE (this fork, decisions/0016): whether the textures prepared from
/// now on mark water surfaces for gs.wgsl's water (Settings::water_effects;
/// the next level's).
pub fn set_water(on: bool) {
    WATER.store(on, Ordering::Relaxed);
}

pub fn water() -> bool {
    WATER.load(Ordering::Relaxed)
}

static PLANTS: AtomicBool = AtomicBool::new(false);

/// NATIVE (this fork, decisions/0017): whether the textures prepared from
/// now on mark plants for gs.wgsl's sway and light through leaves
/// (Settings::plant_effects; the next level's).
pub fn set_plants(on: bool) {
    PLANTS.store(on, Ordering::Relaxed);
}

pub fn plants() -> bool {
    PLANTS.load(Ordering::Relaxed)
}

/// Whether textures prepared now get a surface type (material detail,
/// water or plant effects).
pub fn typing() -> bool {
    enabled() || water() || plants()
}

/// GsParams depth.z of a surface type and alpha class (None: a texture
/// modifier's target, taken as see-through): 1 water (while water effects
/// are on); 2 foliage and 3 cut-out or see-through grass (while plant
/// effects are on: grass ground and hay bales stay still); else 0.
pub fn special(kind: Kind, class: Option<u8>) -> f32 {
    match kind {
        Kind::Water if water() => 1.0,
        Kind::Foliage if plants() => 2.0,
        Kind::Grass if plants() && class != Some(0) => 3.0,
        _ => 0.0,
    }
}

/// A texture's detail map.
pub fn make(src: &Source) -> Detail {
    let (line, guessed) = choose(src.name, src.label);
    let inputs = inputs(src.name);
    detail_of(src, line, guessed, inputs)
}

/// A detail map for a texture of the type `line` names, made from the
/// texture alone (no map files, no texture pack table): a model pack
/// file's material listed in its package's materials.tsv (model_pack,
/// decisions/0029).
pub fn make_for(src: &Source, line: Line) -> Detail {
    detail_of(src, line, false, Inputs { line: None, maps: [None, None, None, None], base: None })
}

fn detail_of(src: &Source, line: Line, guessed: bool, inputs: Inputs) -> Detail {
    let kind = line.kind;
    let p = line.params();
    let flat = |inputs| Detail { width: 1, height: 1, levels: vec![FLAT.to_vec()], kind, guessed, inputs };
    let read = |k: usize| inputs.maps[k].as_ref().and_then(|_| map_path(src.name, k)).and_then(|path| crate::texture_pack::read_png(&path));
    let (normal_file, height_file, rough_file, metal_file) = (read(0), read(1), read(2), read(3));
    if kind == Kind::None && normal_file.is_none() && height_file.is_none() && rough_file.is_none() && metal_file.is_none() {
        return flat(inputs);
    }
    if src.width == 0 || src.height == 0 || src.rgba.len() < (src.width * src.height * 4) as usize {
        return flat(inputs);
    }
    // The map's size: the normal file's, else the height file's, else the
    // texture's (at most MAX_SIDE).
    let (mut w, mut h) = normal_file.as_ref().or(height_file.as_ref()).map_or((src.width, src.height), |f| (f.0, f.1));
    while MAX_SIDE < w.max(h) {
        w = (w / 2).max(1);
        h = (h / 2).max(1);
    }
    let wrap = (src.flags & 0x4 == 0, src.flags & 0x8 == 0);
    let base = resample(src.width, src.height, src.rgba, w, h);
    let opaque: Vec<bool> = base.chunks_exact(4).map(|t| 1 < t[3]).collect();
    let heights: Vec<f32> = match &height_file {
        Some((fw, fh, px)) => resample(*fw, *fh, px, w, h).chunks_exact(4).map(|t| f32::from(t[0]) / 255.0).collect(),
        None => relief(&base, &opaque),
    };
    let normal = normal_file.as_ref().map(|(fw, fh, px)| resample(*fw, *fh, px, w, h));
    let rough = rough_file.as_ref().map(|(fw, fh, px)| resample(*fw, *fh, px, w, h));
    let metal = metal_file.as_ref().map(|(fw, fh, px)| resample(*fw, *fh, px, w, h));
    // Finer texels than the disc's need a steeper slope per texel for the
    // same bump.
    let strength = p.bump * BUMP_SCALE * (w as f32 / src.record_width.max(1) as f32);
    let mut out = vec![0u8; (w * h * 4) as usize];
    for y in 0..h {
        for x in 0..w {
            let i = (y * w + x) as usize;
            let t = &mut out[i * 4..i * 4 + 4];
            if !opaque[i] && normal.is_none() {
                t.copy_from_slice(&FLAT);
                continue;
            }
            let c = heights[i];
            // A neighbour's height (a clear texel's is the centre's: the
            // cut-out's edge is no slope).
            let at = |dx: i32, dy: i32| -> f32 {
                let (nx, ny) = (x as i32 + dx, y as i32 + dy);
                let nx = if wrap.0 { nx.rem_euclid(w as i32) } else { nx.clamp(0, w as i32 - 1) };
                let ny = if wrap.1 { ny.rem_euclid(h as i32) } else { ny.clamp(0, h as i32 - 1) };
                let j = (ny as u32 * w + nx as u32) as usize;
                if opaque[j] { heights[j] } else { c }
            };
            let (nx, ny) = match &normal {
                Some(n) => (n[i * 4], n[i * 4 + 1]),
                None => {
                    // Sobel: the slope to the right and downwards (rows).
                    let gx = (at(1, -1) + 2.0 * at(1, 0) + at(1, 1)) - (at(-1, -1) + 2.0 * at(-1, 0) + at(-1, 1));
                    let gy = (at(-1, 1) + 2.0 * at(0, 1) + at(1, 1)) - (at(-1, -1) + 2.0 * at(0, -1) + at(1, -1));
                    let (sx, sy) = (gx / 8.0 * strength, gy / 8.0 * strength);
                    // n = (−dh/dx, dh/drow, 1): the surface rising to the
                    // right faces left; rising downwards it faces the
                    // image's top (green up).
                    let len = (sx * sx + sy * sy + 1.0).sqrt();
                    (unorm(-sx / len), unorm(sy / len))
                }
            };
            let r = match &rough {
                Some(r) => r[i * 4],
                // Darker hollows a little rougher than the type, brighter
                // tops a little smoother (bumpy types only).
                None => to_u8((p.rough + if 0.0 < p.bump { (0.5 - c) * 0.3 } else { 0.0 }).clamp(0.03, 1.0)),
            };
            let m = match &metal {
                Some(m) => m[i * 4],
                None => to_u8(p.metal),
            };
            t.copy_from_slice(&[nx, ny, r, m]);
        }
    }
    let levels = chain(w, h, out);
    Detail { width: w, height: h, levels, kind, guessed, inputs }
}

/// A height from a texture's brightness, scaled by its own contrast: the
/// opaque texels' mean at 0.5 and two standard deviations either side at
/// 0 and 1, so a dark texture (G1's swamp ground) gets as much relief as a
/// bright one. A nearly flat colour is not amplified into noise.
fn relief(rgba: &[u8], opaque: &[bool]) -> Vec<f32> {
    let lum: Vec<f32> = rgba.chunks_exact(4).map(|t| (0.2126 * f32::from(t[0]) + 0.7152 * f32::from(t[1]) + 0.0722 * f32::from(t[2])) / 255.0).collect();
    let kept: Vec<f32> = lum.iter().zip(opaque).filter(|(_, o)| **o).map(|(l, _)| *l).collect();
    if kept.is_empty() {
        return lum;
    }
    let n = kept.len() as f32;
    let mean = kept.iter().sum::<f32>() / n;
    let sd = (kept.iter().map(|l| (l - mean) * (l - mean)).sum::<f32>() / n).sqrt().max(MIN_SPREAD);
    lum.iter().map(|l| (0.5 + (l - mean) / (4.0 * sd)).clamp(0.0, 1.0)).collect()
}

/// The smallest brightness spread `relief` scales up from.
const MIN_SPREAD: f32 = 0.04;

fn unorm(v: f32) -> u8 {
    to_u8(v * 0.5 + 0.5)
}

fn to_u8(v: f32) -> u8 {
    (v.clamp(0.0, 1.0) * 255.0).round() as u8
}

/// RGBA8 `src` (w × h) at `dw` × `dh`: bilinear, texel centres matched.
fn resample(w: u32, h: u32, src: &[u8], dw: u32, dh: u32) -> Vec<u8> {
    if (w, h) == (dw, dh) {
        return src[..(w * h * 4) as usize].to_vec();
    }
    let mut out = Vec::with_capacity((dw * dh * 4) as usize);
    for y in 0..dh {
        let fy = ((y as f32 + 0.5) * h as f32 / dh as f32 - 0.5).clamp(0.0, (h - 1) as f32);
        let (y0, ty) = (fy.floor() as u32, fy.fract());
        let y1 = (y0 + 1).min(h - 1);
        for x in 0..dw {
            let fx = ((x as f32 + 0.5) * w as f32 / dw as f32 - 0.5).clamp(0.0, (w - 1) as f32);
            let (x0, tx) = (fx.floor() as u32, fx.fract());
            let x1 = (x0 + 1).min(w - 1);
            let px = |xx: u32, yy: u32, c: usize| f32::from(src[((yy * w + xx) * 4) as usize + c]);
            for c in 0..4 {
                let top = px(x0, y0, c) * (1.0 - tx) + px(x1, y0, c) * tx;
                let bottom = px(x0, y1, c) * (1.0 - tx) + px(x1, y1, c) * tx;
                out.push((top * (1.0 - ty) + bottom * ty).round() as u8);
            }
        }
    }
    out
}

/// `level0` continued to 1×1 by 2×2 boxes (averaged normals grow
/// shorter, so distant bumps flatten: gs.wgsl rebuilds z).
fn chain(w: u32, h: u32, level0: Vec<u8>) -> Vec<Vec<u8>> {
    let mut levels = vec![level0];
    let (mut w, mut h) = (w, h);
    while 1 < w || 1 < h {
        let (nw, nh) = ((w / 2).max(1), (h / 2).max(1));
        let prev = levels.last().expect("level 0");
        let mut next = Vec::with_capacity((nw * nh * 4) as usize);
        for y in 0..nh {
            for x in 0..nw {
                for c in 0..4 {
                    let mut sum = 0u32;
                    for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                        let sx = (x * 2 + dx).min(w - 1);
                        let sy = (y * 2 + dy).min(h - 1);
                        sum += u32::from(prev[((sy * w + sx) * 4) as usize + c]);
                    }
                    next.push(((sum + 2) / 4) as u8);
                }
            }
        }
        levels.push(next);
        (w, h) = (nw, nh);
    }
    levels
}

/// A detail map as an image: trilinear, 16× anisotropic, the texture's
/// address modes.
pub fn image(d: &Detail, flags: u16) -> Image {
    let mut s = ImageSamplerDescriptor::linear();
    s.address_mode_u = if flags & 0x4 != 0 { ImageAddressMode::ClampToEdge } else { ImageAddressMode::Repeat };
    s.address_mode_v = if flags & 0x8 != 0 { ImageAddressMode::ClampToEdge } else { ImageAddressMode::Repeat };
    s.set_anisotropic_filter(16);
    crate::mip_image(d.width as u16, d.height as u16, d.levels.clone(), ImageSampler::Descriptor(s))
}

/// A detail map registered for F5.
struct Entry {
    base: AssetId<Image>,
    name: String,
    label: String,
    flags: u16,
    record_width: u16,
    inputs: Inputs,
}

static REGISTRY: Mutex<Option<HashMap<AssetId<Image>, Entry>>> = Mutex::new(None);

/// Remember a detail map and the base texture it belongs to (materials_for).
pub fn register(detail: AssetId<Image>, base: AssetId<Image>, name: String, label: String, flags: u16, record_width: u16, inputs: Inputs) {
    if let Ok(mut r) = REGISTRY.lock() {
        r.get_or_insert_with(HashMap::new).insert(detail, Entry { base, name, label, flags, record_width, inputs });
    }
}

/// F5: read materials.tsv again and make again every detail map whose
/// line, map files or texture file changed (after texture_reload put the
/// texture's new texels into its image). The number made again, and the
/// table lines not understood.
pub fn reload(images: &mut Assets<Image>) -> (usize, usize) {
    let bad = load_table(crate::texture_pack::pack_dir().as_deref());
    let Ok(mut r) = REGISTRY.lock() else { return (0, bad) };
    let Some(map) = r.as_mut() else { return (0, bad) };
    map.retain(|id, _| images.get(*id).is_some());
    let mut made = 0;
    for (id, e) in map.iter_mut() {
        if inputs(&e.name) == e.inputs {
            continue;
        }
        let Some(base) = images.get(e.base) else { continue };
        let (w, h) = (base.width(), base.height());
        let Some(rgba) = base.data.as_ref().and_then(|d| d.get(..(w * h * 4) as usize)).map(<[u8]>::to_vec) else { continue };
        let d = make(&Source { name: &e.name, label: &e.label, width: w, height: h, rgba: &rgba, record_width: e.record_width.into(), flags: e.flags });
        e.inputs = d.inputs.clone();
        if let Some(img) = images.get_mut(*id) {
            *img = image(&d, e.flags);
            made += 1;
        }
    }
    (made, bad)
}

/// The type and whether it was guessed, for the texture page
/// (texture_browser): the table's line, else the label's guess.
pub fn describe(name: &str, label: &str) -> (Kind, bool) {
    let (line, guessed) = choose(name, label);
    (line.kind, guessed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn labels_suggest_types() {
        for (label, kind) in [
            ("BARN_WALL", Kind::Wood),
            ("SAFEROCK3", Kind::Stone),
            ("NICEGLOW", Kind::None),
            ("YEL_SOR_FACE", Kind::Skin),
            ("TREE_BARK", Kind::Wood),
            ("TREE01", Kind::Foliage),
            ("C1WATER19", Kind::Water),
            ("LAVAB", Kind::Lava),
            ("YETICEBREATH00", Kind::Auto),
            ("BLUEPOOLFIRE", Kind::None),
            ("SANDBACK", Kind::Ground),
            ("WAR_ARMOR", Kind::Metal),
            ("FLOOR_CYL", Kind::Stone),
            // Run-together words (Tower labels, 2026-11-22).
            ("SKYWOODWALLSHARP", Kind::Wood),
            ("BLOCKYWALLSGRAYHIGHLIGHTS", Kind::Stone),
            ("WHITEMARBLEFLOOR", Kind::Stone),
            ("DESERTDIRTYBRICKS", Kind::Stone),
            ("MOUNTAINFOG", Kind::None),
            ("MOUNTHORIZONTHENEW", Kind::None),
            ("FOUNT_BOWL", Kind::Stone),
            ("GW_HAT", Kind::Cloth),
            ("YEL_SOR_THIGH", Kind::Skin),
            ("INFERNO", Kind::Auto),
            ("BASIC_WALL_WATEREDGE", Kind::Stone),
            // G1's plants (2026-11-22).
            ("CORN_SIDE", Kind::Foliage),
            ("MOSS_BRANCH", Kind::Foliage),
            ("GRASS_TUFFS", Kind::Grass),
            ("HAYSTACK_01", Kind::Grass),
            ("SKEL2_SPINE", Kind::Auto),
            ("SWATER23", Kind::Water),
            ("GEMSTONES-NA", Kind::Gem),
            ("PINK_GEM", Kind::Gem),
            ("MOHANDGEM_GRE", Kind::Gem),
        ] {
            assert_eq!(guess(label), kind, "{label}");
        }
    }

    /// Water, foliage and see-through grass are marked for their shading;
    /// grass ground (opaque) stays still.
    #[test]
    fn special_surfaces_follow_their_switches() {
        set_water(true);
        set_plants(true);
        assert_eq!(special(Kind::Water, Some(2)), 1.0);
        assert_eq!(special(Kind::Foliage, Some(0)), 2.0);
        assert_eq!(special(Kind::Grass, Some(2)), 3.0);
        assert_eq!(special(Kind::Grass, None), 3.0);
        assert_eq!(special(Kind::Grass, Some(0)), 0.0);
        assert_eq!(special(Kind::Stone, Some(0)), 0.0);
        set_water(false);
        set_plants(false);
        assert_eq!(special(Kind::Water, Some(2)), 0.0);
        assert_eq!(special(Kind::Foliage, Some(1)), 0.0);
    }

    #[test]
    fn table_lines_set_types_and_values() {
        let text = "# a comment\r\n0123456789abcdef stone\r\n0123456789ABCDE0\twood bump=2.5 rough=0.4 metal=9 # BARN\r\nnot-a-name stone\r\n0123456789abcdee marble\r\n\r\n";
        let (t, bad) = parse_table(text);
        assert_eq!(bad, 2);
        assert_eq!(t["0123456789abcdef"].kind, Kind::Stone);
        let w = &t["0123456789abcde0"];
        assert_eq!((w.kind, w.bump, w.rough, w.metal), (Kind::Wood, Some(2.5), Some(0.4), Some(1.0)));
        assert_eq!(w.params(), Params { bump: 2.5, rough: 0.4, metal: 1.0 });
    }

    fn texture(w: u32, h: u32, f: impl Fn(u32, u32) -> u8) -> Vec<u8> {
        (0..h).flat_map(|y| (0..w).map(move |x| (x, y))).flat_map(|(x, y)| [f(x, y), f(x, y), f(x, y), 0x80]).collect()
    }

    /// A surface rising to the right faces left (red under 128); rising
    /// downwards (rows) it faces the image's top (green over 128).
    #[test]
    fn generated_normals_follow_the_slope() {
        let right = texture(8, 8, |x, _| (x * 30) as u8);
        let d = make(&Source { name: "0000000000000000", label: "ROCK", width: 8, height: 8, rgba: &right, record_width: 8, flags: 0xc });
        let t = &d.levels[0][(3 * 8 + 4) * 4..][..4];
        assert!(t[0] < 110 && (126..=130).contains(&t[1]), "{t:?}");
        let down = texture(8, 8, |_, y| (y * 30) as u8);
        let d = make(&Source { name: "0000000000000000", label: "ROCK", width: 8, height: 8, rgba: &down, record_width: 8, flags: 0xc });
        let t = &d.levels[0][(4 * 8 + 3) * 4..][..4];
        assert!(146 < t[1] && (126..=130).contains(&t[0]), "{t:?}");
        assert_eq!(d.kind, Kind::Stone);
        assert!(d.guessed);
        // The chain reaches 1×1.
        assert_eq!(d.levels.len(), 4);
        assert_eq!(d.levels[3].len(), 4);
    }

    /// A model pack material (decisions/0029): the type from its line, not
    /// a guess from the name, and no texture pack files.
    #[test]
    fn model_pack_materials_take_their_line() {
        let right = texture(8, 8, |x, _| (x * 30) as u8);
        let (table, bad) = parse_named("HEAP_A metal\nHEAP_B gem rough=0.5 bump=0\n# a comment\nHEAP_C shiny\n", |n| n.len() <= 128, false);
        assert_eq!(bad, 1, "HEAP_C's type is not understood");
        assert!(parse_table("HEAP_A metal\n").0.is_empty(), "texture pack names are hashes");
        // A name that would guess wood (TREASURE) and none: the line decides.
        let src = |name| Source { name, label: "", width: 8, height: 8, rgba: &right, record_width: 8, flags: 0xc };
        let a = make_for(&src("CHEST_TREASURE"), table["HEAP_A"].clone());
        assert_eq!((a.kind, a.guessed), (Kind::Metal, false));
        let t = &a.levels[0][(3 * 8 + 4) * 4..][..4];
        assert!(t[0] < 110 && t[3] == 255, "{t:?}");
        let b = make_for(&src("CHEST_TREASURE"), table["HEAP_B"].clone());
        assert_eq!(b.kind, Kind::Gem);
        assert!(b.levels[0].chunks_exact(4).all(|t| t[3] == 0 && (125..=129).contains(&t[2])), "rough 0.5 everywhere (no bump), no metal");
        let none = make_for(&src("X"), Line { kind: Kind::None, bump: None, rough: None, metal: None });
        assert_eq!(none.levels, vec![FLAT.to_vec()]);
    }

    /// Type none is the flat 1×1 map; a cut-out's clear texels are flat
    /// and its edge is no slope.
    #[test]
    fn none_and_clear_texels_are_flat() {
        let px = texture(4, 4, |x, y| (x * 40 + y * 10) as u8);
        let d = make(&Source { name: "0000000000000000", label: "FIRE01", width: 4, height: 4, rgba: &px, record_width: 4, flags: 0 });
        assert_eq!((d.width, d.height, d.levels.clone()), (1, 1, vec![FLAT.to_vec()]));
        let mut cut = texture(4, 4, |_, _| 200);
        for (k, t) in cut.chunks_exact_mut(4).enumerate() {
            if k % 4 < 2 {
                t[3] = 0;
            }
        }
        let d = make(&Source { name: "0000000000000000", label: "BUSH", width: 4, height: 4, rgba: &cut, record_width: 4, flags: 0 });
        for (k, t) in d.levels[0].chunks_exact(4).enumerate() {
            assert!((127..=129).contains(&t[0]) && (127..=129).contains(&t[1]), "texel {k}: {t:?}");
            if k % 4 < 2 {
                assert_eq!(t, FLAT);
            }
        }
    }

    #[test]
    fn resample_keeps_flat_and_scales() {
        let src = texture(2, 2, |x, y| [10, 20, 30, 40][(y * 2 + x) as usize]);
        assert_eq!(resample(2, 2, &src, 2, 2), src);
        let up = resample(2, 2, &src, 4, 4);
        assert_eq!(up.len(), 64);
        assert_eq!(up[0], 10);
        assert_eq!(up[15 * 4], 40);
    }
}
