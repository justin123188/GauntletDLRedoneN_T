//! NATIVE (this fork, decisions/0022; docs/remaster/models-plan.md M5): the
//! model pack. `model_pack\<package>\<LABEL>.glb` beside the game replaces
//! object LABEL of that package wherever the game draws it (level scenery,
//! items, weapons, effects) with the file's geometry and base colour
//! textures. The names are the model export's (model_export::object_files):
//! an export's objects/<package>/<LABEL>.glb, refitted with
//! tools/blender/fit_model.py, goes to model_pack/<package>/<LABEL>.glb.
//! Gameplay never changes: collision, items and timing stay the original
//! data. A file that does not load leaves the original, with a message.
//!
//! What a file may hold (fit_model.py writes exactly this): glTF 2.0
//! binary with its buffer in the file or in one file beside it; triangle
//! primitives with POSITION and NORMAL (float), TEXCOORD_0 (float, or
//! normalised unsigned byte or short), indices of any unsigned size; node
//! translation, rotation, scale or matrix; a base colour texture as a PNG
//! (embedded, or a file beside it) and a base colour factor; alphaMode;
//! doubleSided. Files beside a .glb are named by plain relative paths in
//! its folder or below (tools/blender/share_data.py writes textures/ and
//! data/), and files a package's .glb files share are read and decoded
//! once (decisions/0026). Space: the model export's (x negated against the
//! original, y up, one game unit per metre), so the triangle order is kept.
//!
//! F5 in play (decisions/0033): a package folder whose files changed since
//! it was read (`signature`) is read again, and gdl_play puts the new
//! parts in place.
//!
//! Light: a replacement of an item or a character is lit live, as the game
//! lights items (the level's sun and ambient, present_scene_light). A
//! replacement of scenery drawn with baked light takes the original's
//! (decisions/0035, models-plan option A: light_transfer), so new ground
//! keeps the level's light, drawn through the original's lightmap pass
//! (decisions/0037).
//!
//! Material detail (decisions/0029): a package folder's `materials.tsv`
//! (the texture pack's table form: `<name> <type> [bump=… rough=…
//! metal=…]`) names a file's glTF materials by their `name`; while
//! Settings::material_detail is on, a listed material's base colour
//! texture gets a detail map made from it for that surface type
//! (material_maps::make_for), as the game's own textures get theirs, and
//! shines and takes bumps under the live lights. Unlisted materials draw
//! as before.
use crate::{material_maps, model_export, mip_image, texfilter, GsMaterial};
use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, PrimitiveTopology},
    prelude::*,
};
use gdl_formats::objects::Objects;
use serde_json::Value;
use std::{
    cell::RefCell,
    collections::HashMap,
    hash::{DefaultHasher, Hasher},
    path::{Path, PathBuf},
    sync::{Arc, Mutex, RwLock},
};

static DIR: RwLock<Option<PathBuf>> = RwLock::new(None);
static REPORT: Mutex<Vec<String>> = Mutex::new(Vec::new());

/// Limits of what one file may hold.
const MAX_VERTICES: usize = 1 << 20;
const MAX_TEXTURE: u32 = 4096;

/// The pack folder for this run (None: no model pack).
pub fn configure(dir: Option<PathBuf>) {
    if let Ok(mut d) = DIR.write() {
        *d = dir;
    }
}

pub fn dir() -> Option<PathBuf> {
    DIR.read().ok()?.clone()
}

pub fn active() -> bool {
    dir().is_some()
}

/// The model_pack folder beside the game.
pub fn default_dir() -> Option<PathBuf> {
    Some(crate::texture_pack::game_dir()?.join("model_pack"))
}

const README: &str = "Model pack (NATIVE, this fork): .glb files here replace the game's own\r
models, one object each.\r
\r
model_pack\\<package>\\<LABEL>.glb replaces object LABEL of that package, named\r
as gdl_play --export-models names it (objects\\<package>\\<LABEL>.glb). Fit a\r
new model to the original with tools\\blender\\fit_model.py, which writes the\r
file in the form the game reads (PNG textures, no compression).\r
A package's textures and data folders hold what its .glb files share\r
(tools\\blender\\share_data.py): keep them with the files. A package's\r
materials.tsv (lines \"<material name> <type>\", as the texture pack's) gives\r
the files' materials bumps and shine with Material detail on.\r
\r
Read when the game starts; F5 in play reads the folders whose files changed\r
again. Collision and gameplay stay the original's. Delete a file to get the\r
original back; F10 > Graphics > Model pack folder turns the folder off.\r
";

/// NATIVE (F5, decisions/0033): the state of a package's folder (None
/// without the model pack or the folder): every file below it, by its path
/// there, its length and its modification time, hashed. F5 reads a package
/// again when this changed.
pub fn signature(package: &str) -> Option<u64> {
    let folder = dir()?.join(model_export::package_dir(package));
    folder.is_dir().then(|| folder_signature(&folder))
}

/// `signature` of a folder.
fn folder_signature(folder: &Path) -> u64 {
    fn walk(root: &Path, at: &Path, depth: u32, out: &mut Vec<(String, u64, u128)>) {
        // A package's folder holds its files, textures/ and data/.
        if 4 < depth || 100_000 < out.len() {
            return;
        }
        let Ok(entries) = std::fs::read_dir(at) else { return };
        for e in entries.flatten() {
            let (path, Ok(meta)) = (e.path(), e.metadata()) else { continue };
            if meta.is_dir() {
                walk(root, &path, depth + 1, out);
                continue;
            }
            let rel = path.strip_prefix(root).map(|r| r.to_string_lossy().into_owned()).unwrap_or_default();
            let modified = meta.modified().ok().and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok()).map_or(0, |d| d.as_nanos());
            out.push((rel, meta.len(), modified));
        }
    }
    let mut files = Vec::new();
    walk(folder, folder, 0, &mut files);
    files.sort();
    let mut h = DefaultHasher::new();
    for (rel, len, modified) in files {
        h.write(rel.as_bytes());
        h.write_u8(0);
        h.write_u64(len);
        h.write_u128(modified);
    }
    h.finish()
}

/// The folder, made when missing, with its README.txt.
pub fn ensure_dir(dir: &Path) {
    if std::fs::create_dir_all(dir).is_ok() && !dir.join("README.txt").exists() {
        let _ = std::fs::write(dir.join("README.txt"), README);
    }
}

/// Replacement lines since the last call ("ITEMS_G/BAROBJIDLE15F01: …").
pub fn take_report() -> Vec<String> {
    REPORT.lock().map(|mut r| std::mem::take(&mut *r)).unwrap_or_default()
}

fn report(line: String) {
    eprintln!("model pack: {line}");
    if let Ok(mut r) = REPORT.lock() {
        r.push(line);
    }
}

/// One primitive of a replacement, in the original's space.
pub struct NewPart {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub indices: Vec<u32>,
    /// Width, height and RGBA with GS alpha (0x80 opaque), the base colour
    /// factor applied; a 1×1 texel of the factor without a texture.
    pub texture: Texture,
    /// The same for parts whose texture is the same: a hash of the image
    /// file's bytes, the colour factor and the alpha class.
    pub texture_key: u64,
    /// 0 opaque, 1 cut-out, 2 blended (the file's alphaMode).
    pub class: u8,
    pub double_sided: bool,
    /// U and V clamp to the edge (else repeat).
    pub clamp: (bool, bool),
    /// The glTF material's `name` (materials.tsv's key).
    pub material: Option<String>,
}

type R<T> = Result<T, String>;

/// A decoded texture: width, height and RGBA with GS alpha.
pub type Texture = Arc<(u32, u32, Vec<u8>)>;

/// A file's bytes and their hash.
type Shared = (u64, Arc<Vec<u8>>);

/// Decoded textures by their key ([`NewPart::texture_key`]), and the files
/// read beside a package's .glb files by path: the files of a package that
/// share one image (embedded in each, or one file beside them, as
/// tools/blender/share_data.py writes) decode it once.
#[derive(Default)]
pub struct Textures {
    decoded: HashMap<u64, Texture>,
    files: HashMap<PathBuf, Shared>,
    /// DIAGNOSTIC: the time spent decoding PNGs.
    decode_time: std::time::Duration,
}

fn content_hash(bytes: &[u8]) -> u64 {
    let mut h = DefaultHasher::new();
    h.write_usize(bytes.len());
    h.write(bytes);
    h.finish()
}

/// A file named by a .glb's `uri`, beside it: a relative path of plain names
/// (letters, digits, `_`, `-`, `.`) in its folder or below, never above it,
/// never another drive, scheme or encoded name.
fn beside_path(dir: Option<&Path>, uri: &str) -> R<PathBuf> {
    let dir = dir.ok_or("data outside the file")?;
    let plain = |c: &str| !c.is_empty() && c != "." && c != ".." && c.chars().all(|ch| ch.is_ascii_alphanumeric() || "_-.".contains(ch));
    if !uri.split('/').all(plain) {
        return Err(format!("{uri}: only a plain relative path beside the file is read"));
    }
    Ok(uri.split('/').fold(dir.to_path_buf(), |p, c| p.join(c)))
}

fn read_beside(files: &mut HashMap<PathBuf, Shared>, dir: Option<&Path>, uri: &str) -> R<Shared> {
    let path = beside_path(dir, uri)?;
    if let Some(hit) = files.get(&path) {
        return Ok(hit.clone());
    }
    let bytes = crate::bounded_read(&path).map_err(|e| format!("{uri}: {e}"))?;
    let got = (content_hash(&bytes), Arc::new(bytes));
    files.insert(path, got.clone());
    Ok(got)
}

/// A file's parts (data inside the file only).
pub fn read_glb(bytes: &[u8]) -> R<Vec<NewPart>> {
    read_glb_with(bytes, &mut Textures::default(), None)
}

/// A file's parts, with textures shared through `textures`; `beside` is the
/// file's folder, for a buffer or images in files beside it.
pub fn read_glb_with(bytes: &[u8], textures: &mut Textures, beside: Option<&Path>) -> R<Vec<NewPart>> {
    let word = |o: usize| -> R<u32> { Ok(u32::from_le_bytes(bytes.get(o..o + 4).ok_or("truncated file")?.try_into().unwrap())) };
    if bytes.get(0..4) != Some(b"glTF") || word(4)? != 2 {
        return Err("not a glTF 2.0 binary (.glb)".into());
    }
    let total = (word(8)? as usize).min(bytes.len());
    let (mut json, mut bin): (Option<&[u8]>, &[u8]) = (None, &[]);
    let mut at = 12;
    while at + 8 <= total {
        let (len, kind) = (word(at)? as usize, word(at + 4)?);
        let data = bytes.get(at + 8..at + 8 + len).ok_or("chunk past the end of the file")?;
        match kind {
            0x4E4F_534A => json = Some(data),
            0x004E_4942 => bin = data,
            _ => {}
        }
        at += 8 + len;
    }
    let doc: Value = serde_json::from_slice(json.ok_or("no JSON chunk")?).map_err(|e| format!("JSON: {e}"))?;
    let buffers = doc["buffers"].as_array().map(Vec::as_slice).unwrap_or(&[]);
    if 1 < buffers.len() {
        return Err("more than one buffer".into());
    }
    // The buffer in a file beside this one (share_data.py: one frame's
    // geometry for every kind of a barrel).
    let outside = match buffers.first().and_then(|b| b["uri"].as_str()) {
        Some(uri) => Some(read_beside(&mut textures.files, beside, uri)?),
        None => None,
    };
    let bin: &[u8] = outside.as_ref().map_or(bin, |(_, b)| b.as_slice());
    let g = Gltf {
        doc: &doc,
        bin,
        beside,
        textures: RefCell::new(std::mem::take(&mut textures.decoded)),
        files: RefCell::new(std::mem::take(&mut textures.files)),
        images: RefCell::new(HashMap::new()),
        decode_time: std::cell::Cell::new(std::time::Duration::ZERO),
    };
    let scene = doc["scene"].as_u64().unwrap_or(0) as usize;
    let roots: Vec<usize> = match doc["scenes"].get(scene) {
        Some(s) => s["nodes"].as_array().map(|n| n.iter().filter_map(|v| v.as_u64().map(|v| v as usize)).collect()).unwrap_or_default(),
        None => (0..doc["nodes"].as_array().map_or(0, Vec::len)).collect(),
    };
    let mut out = Vec::new();
    let mut vertices = 0;
    let walked = roots.into_iter().try_for_each(|root| g.node(root, Mat4::IDENTITY, 0, &mut out, &mut vertices));
    textures.decode_time += g.decode_time.get();
    textures.decoded = g.textures.into_inner();
    textures.files = g.files.into_inner();
    walked?;
    if out.is_empty() {
        return Err("no triangles".into());
    }
    Ok(out)
}

/// An accessor's view bytes from its first element, element stride,
/// count, component type, components and whether it is normalised.
type Access<'a> = (&'a [u8], usize, usize, u64, usize, bool);

struct Gltf<'a> {
    doc: &'a Value,
    bin: &'a [u8],
    beside: Option<&'a Path>,
    textures: RefCell<HashMap<u64, Texture>>,
    files: RefCell<HashMap<PathBuf, Shared>>,
    /// This file's images by index: their bytes and hash, found once.
    images: RefCell<HashMap<usize, Shared>>,
    /// DIAGNOSTIC: the time spent decoding this file's PNGs.
    decode_time: std::cell::Cell<std::time::Duration>,
}

impl Gltf<'_> {
    fn node(&self, index: usize, parent: Mat4, depth: usize, out: &mut Vec<NewPart>, vertices: &mut usize) -> R<()> {
        if 64 < depth {
            return Err("node hierarchy too deep".into());
        }
        let n = self.doc["nodes"].get(index).ok_or("node index")?;
        let local = if let Some(m) = n["matrix"].as_array() {
            let v: Vec<f32> = m.iter().filter_map(|x| x.as_f64().map(|x| x as f32)).collect();
            Mat4::from_cols_slice(v.get(..16).ok_or("node matrix")?)
        } else {
            let f = |k: &str, d: &[f32]| -> Vec<f32> { n[k].as_array().map(|a| a.iter().filter_map(|x| x.as_f64().map(|x| x as f32)).collect()).unwrap_or_else(|| d.to_vec()) };
            let (t, r, s) = (f("translation", &[0.0; 3]), f("rotation", &[0.0, 0.0, 0.0, 1.0]), f("scale", &[1.0; 3]));
            if t.len() != 3 || r.len() != 4 || s.len() != 3 {
                return Err("node transform".into());
            }
            Mat4::from_scale_rotation_translation(Vec3::from_slice(&s), Quat::from_slice(&r).normalize(), Vec3::from_slice(&t))
        };
        let world = parent * local;
        if let Some(mesh) = n["mesh"].as_u64() {
            self.mesh(mesh as usize, world, out, vertices)?;
        }
        for child in n["children"].as_array().into_iter().flatten() {
            self.node(child.as_u64().ok_or("child index")? as usize, world, depth + 1, out, vertices)?;
        }
        Ok(())
    }

    fn mesh(&self, index: usize, world: Mat4, out: &mut Vec<NewPart>, vertices: &mut usize) -> R<()> {
        let mesh = self.doc["meshes"].get(index).ok_or("mesh index")?;
        let normal_matrix = Mat3::from_mat4(world).inverse().transpose();
        let flip = world.determinant() < 0.0;
        for p in mesh["primitives"].as_array().ok_or("mesh primitives")? {
            if p["mode"].as_u64().unwrap_or(4) != 4 {
                return Err("only triangle lists are read".into());
            }
            let a = &p["attributes"];
            let positions = self.floats::<3>(a["POSITION"].as_u64().ok_or("a primitive without POSITION")? as usize)?;
            let normals = self.floats::<3>(a["NORMAL"].as_u64().ok_or("a primitive without NORMAL (fit_model.py writes them)")? as usize)?;
            let uvs = match a["TEXCOORD_0"].as_u64() {
                Some(t) => self.floats::<2>(t as usize)?,
                None => vec![[0.0; 2]; positions.len()],
            };
            if normals.len() != positions.len() || uvs.len() != positions.len() {
                return Err("attribute counts differ".into());
            }
            *vertices += positions.len();
            if MAX_VERTICES < *vertices {
                return Err(format!("more than {MAX_VERTICES} vertices"));
            }
            let mut indices = match p["indices"].as_u64() {
                Some(i) => self.indices(i as usize)?,
                None => (0..positions.len() as u32).collect(),
            };
            if !indices.len().is_multiple_of(3) || indices.iter().any(|&i| i as usize >= positions.len()) {
                return Err("bad triangle indices".into());
            }
            if flip {
                for t in indices.chunks_exact_mut(3) {
                    t.swap(1, 2);
                }
            }
            // The export's space back to the original's: x negated.
            let back = |v: Vec3| [-v.x, v.y, v.z];
            let positions = positions.iter().map(|&v| back(world.transform_point3(Vec3::from_array(v)))).collect();
            let normals = normals.iter().map(|&v| back((normal_matrix * Vec3::from_array(v)).normalize_or(Vec3::Y))).collect();
            let index = p["material"].as_u64().map(|m| m as usize);
            let (texture, texture_key, class, double_sided, clamp) = self.material(index)?;
            let material = index.and_then(|m| self.doc["materials"].get(m)).and_then(|m| m["name"].as_str()).map(str::to_owned);
            out.push(NewPart { positions, normals, uvs, indices, texture, texture_key, class, double_sided, clamp, material });
        }
        Ok(())
    }

    #[allow(clippy::type_complexity)]
    fn material(&self, index: Option<usize>) -> R<(Texture, u64, u8, bool, (bool, bool))> {
        let none = Value::Null;
        let m = match index {
            Some(i) => self.doc["materials"].get(i).ok_or("material index")?,
            None => &none,
        };
        let pbr = &m["pbrMetallicRoughness"];
        let factor: Vec<f32> = pbr["baseColorFactor"].as_array().map(|a| a.iter().filter_map(|x| x.as_f64().map(|x| x as f32)).collect()).unwrap_or_else(|| vec![1.0; 4]);
        let factor: [f32; 4] = factor.try_into().map_err(|_| "baseColorFactor")?;
        let class = match m["alphaMode"].as_str().unwrap_or("OPAQUE") {
            "MASK" => 1,
            "BLEND" => 2,
            _ => 0,
        };
        let mut clamp = (false, false);
        let png = match pbr["baseColorTexture"]["index"].as_u64() {
            Some(t) => {
                let tex = self.doc["textures"].get(t as usize).ok_or("texture index")?;
                if let Some(s) = tex["sampler"].as_u64().and_then(|s| self.doc["samplers"].get(s as usize)) {
                    clamp = (s["wrapS"].as_u64() == Some(33071), s["wrapT"].as_u64() == Some(33071));
                }
                let source = tex["source"].as_u64().ok_or("a texture without an image")? as usize;
                let img = self.doc["images"].get(source).ok_or("image index")?;
                let mime = img["mimeType"].as_str();
                let png_name = img["uri"].as_str().is_some_and(|u| u.to_ascii_lowercase().ends_with(".png"));
                if mime != Some("image/png") && !(mime.is_none() && png_name) {
                    return Err(format!("a {} texture: only PNG is read (fit_model.py writes PNG)", mime.unwrap_or("non-PNG")));
                }
                Some(self.image(source, img)?)
            }
            None => None,
        };
        let mut hash = DefaultHasher::new();
        hash.write_u8(class);
        for f in factor {
            hash.write_u32(f.to_bits());
        }
        if let Some((h, _)) = &png {
            hash.write_u64(*h);
        }
        let key = hash.finish();
        let double_sided = m["doubleSided"].as_bool().unwrap_or(false);
        if let Some(t) = self.textures.borrow().get(&key) {
            return Ok((t.clone(), key, class, double_sided, clamp));
        }
        let decoding = std::time::Instant::now();
        let (w, h, mut rgba) = match &png {
            Some((_, bytes)) => decode_png(bytes)?,
            None => (1, 1, vec![255; 4]),
        };
        self.decode_time.set(self.decode_time.get() + decoding.elapsed());
        for p in rgba.chunks_exact_mut(4) {
            for (c, f) in p.iter_mut().zip(factor) {
                *c = (f32::from(*c) * f.clamp(0.0, 1.0)).round() as u8;
            }
            // GS alpha (0x80 opaque); an opaque material has no alpha.
            p[3] = if class == 0 { 0x80 } else { ((u16::from(p[3]) * 128 + 127) / 255) as u8 };
        }
        let t = Arc::new((w, h, rgba));
        self.textures.borrow_mut().insert(key, t.clone());
        Ok((t, key, class, double_sided, clamp))
    }

    /// Image `source`'s PNG bytes and their hash: once per image of this
    /// file, and once per file beside it for the whole package.
    fn image(&self, source: usize, img: &Value) -> R<Shared> {
        if let Some(hit) = self.images.borrow().get(&source) {
            return Ok(hit.clone());
        }
        let got = match img["uri"].as_str() {
            Some(uri) => read_beside(&mut self.files.borrow_mut(), self.beside, uri)?,
            None => {
                let bytes = self.view(img["bufferView"].as_u64().ok_or("an image with no data")? as usize)?;
                (content_hash(bytes), Arc::new(bytes.to_vec()))
            }
        };
        self.images.borrow_mut().insert(source, got.clone());
        Ok(got)
    }

    fn view(&self, index: usize) -> R<&[u8]> {
        let v = self.doc["bufferViews"].get(index).ok_or("buffer view index")?;
        if v["buffer"].as_u64().unwrap_or(0) != 0 {
            return Err("a second buffer".into());
        }
        let start = v["byteOffset"].as_u64().unwrap_or(0) as usize;
        let len = v["byteLength"].as_u64().ok_or("buffer view length")? as usize;
        self.bin.get(start..start + len).ok_or_else(|| "buffer view past the data".into())
    }

    /// The accessor's data: see [`Access`].
    fn accessor(&self, index: usize) -> R<Access<'_>> {
        let a = self.doc["accessors"].get(index).ok_or("accessor index")?;
        if a.get("sparse").is_some() {
            return Err("sparse accessors are not read".into());
        }
        let components = match a["type"].as_str().unwrap_or("") {
            "SCALAR" => 1,
            "VEC2" => 2,
            "VEC3" => 3,
            "VEC4" => 4,
            t => return Err(format!("accessor type {t}")),
        };
        let ty = a["componentType"].as_u64().ok_or("component type")?;
        let size = match ty {
            5120 | 5121 => 1,
            5122 | 5123 => 2,
            5125 | 5126 => 4,
            _ => return Err(format!("component type {ty}")),
        };
        let count = a["count"].as_u64().ok_or("accessor count")? as usize;
        let view_index = a["bufferView"].as_u64().ok_or("an accessor without data")? as usize;
        let view = self.view(view_index)?;
        let stride = self.doc["bufferViews"][view_index]["byteStride"].as_u64().map_or(size * components, |s| s as usize);
        let offset = a["byteOffset"].as_u64().unwrap_or(0) as usize;
        if MAX_VERTICES * 3 < count || stride < size * components {
            return Err("accessor size".into());
        }
        let need = if count == 0 { 0 } else { (count - 1) * stride + size * components };
        let data = view.get(offset..offset + need).ok_or("accessor past its buffer view")?;
        Ok((data, stride, count, ty, components, a["normalized"].as_bool().unwrap_or(false)))
    }

    fn floats<const N: usize>(&self, index: usize) -> R<Vec<[f32; N]>> {
        let (data, stride, count, ty, components, normalised) = self.accessor(index)?;
        if components != N {
            return Err(format!("expected {N} components, found {components}"));
        }
        let read = |o: usize| -> f32 {
            match ty {
                5126 => f32::from_le_bytes(data[o..o + 4].try_into().unwrap()),
                5121 if normalised => f32::from(data[o]) / 255.0,
                5123 if normalised => f32::from(u16::from_le_bytes([data[o], data[o + 1]])) / 65535.0,
                5120 if normalised => (f32::from(data[o] as i8) / 127.0).max(-1.0),
                5122 if normalised => (f32::from(i16::from_le_bytes([data[o], data[o + 1]])) / 32767.0).max(-1.0),
                5120 => f32::from(data[o] as i8),
                5121 => f32::from(data[o]),
                5122 => f32::from(i16::from_le_bytes([data[o], data[o + 1]])),
                5123 => f32::from(u16::from_le_bytes([data[o], data[o + 1]])),
                _ => u32::from_le_bytes(data[o..o + 4].try_into().unwrap()) as f32,
            }
        };
        let size = match ty {
            5120 | 5121 => 1,
            5122 | 5123 => 2,
            _ => 4,
        };
        let out: Vec<[f32; N]> = (0..count).map(|k| std::array::from_fn(|c| read(k * stride + c * size))).collect();
        if out.iter().flatten().any(|v| !v.is_finite()) {
            return Err("a value that is not a number".into());
        }
        Ok(out)
    }

    fn indices(&self, index: usize) -> R<Vec<u32>> {
        let (data, stride, count, ty, components, _) = self.accessor(index)?;
        if components != 1 {
            return Err("indices are not scalars".into());
        }
        Ok((0..count)
            .map(|k| {
                let o = k * stride;
                match ty {
                    5121 => u32::from(data[o]),
                    5123 => u32::from(u16::from_le_bytes([data[o], data[o + 1]])),
                    _ => u32::from_le_bytes(data[o..o + 4].try_into().unwrap()),
                }
            })
            .collect())
    }
}

/// A PNG as 8-bit RGBA (straight alpha).
fn decode_png(bytes: &[u8]) -> R<(u32, u32, Vec<u8>)> {
    let mut decoder = png::Decoder::new(std::io::Cursor::new(bytes));
    decoder.set_transformations(png::Transformations::normalize_to_color8());
    let mut reader = decoder.read_info().map_err(|e| format!("PNG: {e}"))?;
    let (w, h) = (reader.info().width, reader.info().height);
    if w == 0 || h == 0 || MAX_TEXTURE < w || MAX_TEXTURE < h {
        return Err(format!("a {w}×{h} texture (at most {MAX_TEXTURE}²)"));
    }
    let mut buf = vec![0; reader.output_buffer_size().ok_or("PNG size")?];
    let info = reader.next_frame(&mut buf).map_err(|e| format!("PNG: {e}"))?;
    let px = (w * h) as usize;
    let rgba = match info.color_type {
        png::ColorType::Rgba => buf[..px * 4].to_vec(),
        png::ColorType::Rgb => buf[..px * 3].chunks_exact(3).flat_map(|p| [p[0], p[1], p[2], 255]).collect(),
        png::ColorType::GrayscaleAlpha => buf[..px * 2].chunks_exact(2).flat_map(|p| [p[0], p[0], p[0], p[1]]).collect(),
        png::ColorType::Grayscale => buf[..px].iter().flat_map(|&g| [g, g, g, 255]).collect(),
        png::ColorType::Indexed => return Err("an indexed PNG".into()),
    };
    Ok((w, h, rgba))
}

/// Object parts by object index (gdl_client::object_meshes).
pub type Parts = HashMap<u32, Vec<(Handle<Mesh>, Handle<GsMaterial>)>>;

/// Replaces the parts of every object of `package` that has a file in the
/// pack. Each new part's material starts from the object's first original
/// material (its draw state), with the file's texture, no lightmap and live
/// light.
pub fn apply(package: &str, objects: &Objects, parts: &mut Parts, meshes: &mut Assets<Mesh>, images: &mut Assets<Image>, materials: &mut Assets<GsMaterial>) {
    let Some(dir) = dir() else { return };
    let folder = dir.join(model_export::package_dir(package));
    if !folder.is_dir() {
        return;
    }
    let sharp = texfilter::sharp();
    // A package's files that share a texture (a flip-book's frames) share
    // its decoding, its image and, per object, its material.
    let mut textures = Textures::default();
    // NATIVE (decisions/0029): the materials materials.tsv names get
    // detail maps, made once per texture and type (only while material
    // detail is on, as for the game's own textures: the next level's).
    // Their types mark water and plants for those effects while either is
    // on, with or without material detail, as the game's own textures'
    // types do (decisions/0045).
    let table = if material_maps::typing() { read_table(&folder, package) } else { HashMap::new() };
    let detail = material_maps::enabled();
    // DIAGNOSTIC (performance, with gdl_play's GDL_FRAME_LOG): the
    // package's time reading and decoding, making the textures' mipmaps
    // and detail maps, building the parts, and in all.
    let started = std::time::Instant::now();
    // 1. The files, read and decoded in order; the textures' work once per
    // texture (and per detail line), in the order the parts first use it.
    let mut loaded = Vec::new();
    let mut jobs: Vec<Job> = Vec::new();
    let mut job_of_mips: HashMap<(u64, u16), usize> = HashMap::new();
    let mut job_of_detail: HashMap<(u64, u16, DetailKey), usize> = HashMap::new();
    for (index, stem, label, _) in model_export::object_files(objects) {
        let path = folder.join(format!("{stem}.glb"));
        if !path.is_file() {
            continue;
        }
        let Some(original) = parts.get(&index).and_then(|p| p.first()).and_then(|(_, m)| materials.get(m)).cloned() else {
            report(format!("{package}/{label}: the original has no drawn part; not replaced"));
            continue;
        };
        let new = match crate::bounded_read(&path).map_err(|e| e.to_string()).and_then(|b| read_glb_with(&b, &mut textures, Some(&folder))) {
            Ok(n) => n,
            Err(e) => {
                report(format!("{package}/{label}: {e}; the original stays"));
                continue;
            }
        };
        for p in &new {
            let flags = address_flags(p);
            job_of_mips.entry((p.texture_key, flags)).or_insert_with(|| {
                jobs.push(Job::Mips(p.texture.clone()));
                jobs.len() - 1
            });
            if let Some(line) = detail_line(&table, p).filter(|_| detail) {
                job_of_detail.entry((p.texture_key, flags, detail_key(&line))).or_insert_with(|| {
                    jobs.push(Job::Detail(p.texture.clone(), flags, line, p.material.clone().unwrap_or_default()));
                    jobs.len() - 1
                });
            }
        }
        loaded.push((index, label, path, original, new));
    }
    let read = started.elapsed();
    // 2. The textures' work (pure: the sharp filter's mipmap chains and
    // the detail maps) on the machine's threads.
    let timed: Vec<(Done, std::time::Duration)> = run_parallel(&jobs, |job| {
        let at = std::time::Instant::now();
        let d = match job {
            Job::Mips(t) => Done::Mips(texfilter::with_chain(t.0, t.1, vec![t.2.clone()], sharp)),
            Job::Detail(t, flags, line, name) => {
                // The texture's own texels set the bump's scale (no disc
                // record behind it).
                let src = material_maps::Source { name, label: "", width: t.0, height: t.1, rgba: &t.2, record_width: t.0, flags: *flags };
                Done::Detail(Box::new(material_maps::make_for(&src, line.clone())))
            }
        };
        (d, at.elapsed())
    });
    let (longest_job, longest_what) = timed
        .iter()
        .zip(&jobs)
        .max_by_key(|((_, t), _)| *t)
        .map(|((_, t), j)| {
            let what = match j {
                Job::Mips(t) => format!("mipmaps {}x{}", t.0, t.1),
                Job::Detail(t, _, line, _) => format!("detail map {}x{} {}", t.0, t.1, line.kind.name()),
            };
            (*t, what)
        })
        .unwrap_or_default();
    let mut done: Vec<Option<Done>> = timed.into_iter().map(|(d, _)| Some(d)).collect();
    let work = started.elapsed() - read;
    let mut carry_time = std::time::Duration::ZERO;
    // 3. The parts, in order, each image added where it is first used (as
    // before the work went to threads).
    let mut uploaded: HashMap<(u64, u16), Handle<Image>> = HashMap::new();
    let mut details: HashMap<(u64, u16, DetailKey), Handle<Image>> = HashMap::new();
    let mut ramp: Option<Handle<Image>> = None;
    let files = loaded.len();
    // NATIVE (decisions/0035, 0037): scenery drawn with baked light takes
    // the original's (light_transfer: each vertex the light of the nearest
    // point of the original's surface, drawn as the scenery draws: the
    // vertex colour, and the lightmap through the ramp where the original
    // has one); items and characters keep the live light. Each object's
    // surface first, in order (it reads the original's assets); then every
    // vertex's light on the machine's threads, in runs of CARRY_RUN
    // vertices (pure work, the same result as one at a time;
    // decisions/0049).
    let carrying = std::time::Instant::now();
    let sources: Vec<Option<crate::light_transfer::Source>> = loaded
        .iter()
        .map(|(index, _, _, original, _)| {
            let baked = !original.lit && original.params.light_dir.w.abs() < 0.5 && !original.draw.additive;
            if baked { parts.get(index).and_then(|l| crate::light_transfer::source_of(l, meshes, materials, images)) } else { None }
        })
        .collect();
    let mut runs: Vec<(usize, usize, usize, usize)> = Vec::new();
    for (f, (_, _, _, _, new)) in loaded.iter().enumerate() {
        if sources[f].is_none() {
            continue;
        }
        for (i, p) in new.iter().enumerate() {
            for start in (0..p.positions.len()).step_by(CARRY_RUN) {
                runs.push((f, i, start, (start + CARRY_RUN).min(p.positions.len())));
            }
        }
    }
    let lit: Vec<Carried> = run_parallel(&runs, |&(f, i, start, end)| {
        let s = sources[f].as_ref().expect("a run only for a file with a surface");
        loaded[f].4[i].positions[start..end].iter().map(|v| s.baked_at(Vec3::from_array(*v))).collect()
    });
    let mut carried: HashMap<(usize, usize), Carried> = HashMap::new();
    for (&(f, i, _, _), run) in runs.iter().zip(lit) {
        carried.entry((f, i)).or_default().extend(run);
    }
    carry_time += carrying.elapsed();
    for (f, ((index, label, path, original, new), source)) in loaded.into_iter().zip(sources).enumerate() {
        let mut made: HashMap<MadeKey, Handle<GsMaterial>> = HashMap::new();
        let mut list = Vec::new();
        let mut triangles = 0;
        for (i, p) in new.into_iter().enumerate() {
            triangles += p.indices.len() / 3;
            let n = p.positions.len();
            let flags = address_flags(&p);
            let line = detail_line(&table, &p);
            let (colours, lightmap_uvs): (Vec<[f32; 4]>, Option<Vec<[f32; 2]>>) = match carried.remove(&(f, i)) {
                Some(at) => {
                    let uvs = at.iter().any(|(_, lm)| lm.is_some()).then(|| at.iter().map(|(_, lm)| crate::light_transfer::ramp_uv(lm.unwrap_or(1.0))).collect());
                    (at.iter().map(|(c, _)| c.extend(1.0).to_array()).collect(), uvs)
                }
                None => (vec![[1.0f32; 4]; n], None),
            };
            let lightmapped = lightmap_uvs.is_some();
            let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
                .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, p.positions)
                .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, p.normals)
                .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, p.uvs)
                .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colours)
                .with_inserted_indices(Indices::U32(p.indices));
            if let Some(uvs) = lightmap_uvs {
                mesh.insert_attribute(Mesh::ATTRIBUTE_UV_1, uvs);
            }
            let detail_key = line.as_ref().map(detail_key);
            let shared = (p.texture_key, flags, p.class, p.double_sided, detail_key, lightmapped);
            if let Some(m) = made.get(&shared) {
                list.push((meshes.add(mesh), m.clone()));
                continue;
            }
            let image = uploaded
                .entry((p.texture_key, flags))
                .or_insert_with(|| {
                    let (w, h, _) = &*p.texture;
                    let Some(Done::Mips(levels)) = done[job_of_mips[&(p.texture_key, flags)]].take() else { unreachable!("each texture's mipmaps are made once") };
                    let key = texfilter::TexKey { flags, levels: 1, animated: false, borrowed: false };
                    let image = images.add(mip_image(*w as u16, *h as u16, levels, texfilter::sampler(key, sharp)));
                    texfilter::register(image.id(), key);
                    image
                })
                .clone();
            let mut m = original.clone();
            m.base = Some(image);
            // The original's lightmap pass, its light from the ramp.
            m.lightmap = lightmapped.then(|| ramp.get_or_insert_with(|| images.add(crate::light_transfer::ramp_image())).clone());
            m.overlay = None;
            m.detail = None;
            // Lit live as items are, unless it carries the original's light.
            m.lit = source.is_none();
            m.params.options.x = 1.0;
            m.params.options.y = if lightmapped { 1.0 } else { 0.0 };
            m.params.scale.z = 0.0;
            m.params.scale.w = 0.0;
            m.params.depth.y = 0.0;
            m.params.depth.z = 0.0;
            m.alpha_mode = match p.class {
                0 => AlphaMode::Opaque,
                1 => AlphaMode::Mask(0.5),
                _ => AlphaMode::Blend,
            };
            m.draw.cull = !p.double_sided;
            if let (Some(line), Some(k)) = (line, detail_key) {
                if detail {
                    let detail = details
                        .entry((p.texture_key, flags, k))
                        .or_insert_with(|| {
                            let Some(Done::Detail(d)) = done[job_of_detail[&(p.texture_key, flags, k)]].take() else { unreachable!("each detail map is made once") };
                            images.add(material_maps::image(&d, flags))
                        })
                        .clone();
                    m.detail = Some(detail);
                    m.params.depth.y = 1.0;
                }
                // Water and plants, while their effects are on.
                m.params.depth.z = material_maps::special(line.kind, Some(p.class));
            }
            let m = materials.add(m);
            made.insert(shared, m.clone());
            list.push((meshes.add(mesh), m));
        }
        let light = if source.is_some() { ", the original's light carried over" } else { "" };
        report(format!("{package}/{label}: {triangles} triangles in {} parts from {}{light}", list.len(), path.display()));
        parts.insert(index, list);
    }
    if std::env::var_os("GDL_FRAME_LOG").is_some() {
        let ms = |d: std::time::Duration| d.as_secs_f64() * 1000.0;
        eprintln!(
            "PERF model_pack {package} {files} files: read and decode {:.0} ms (PNG decoding {:.0}), mipmaps and detail maps {:.0} ms ({} jobs, the longest {:.0}: {longest_what}), parts {:.0} ms (light carried {:.0}), all {:.0} ms",
            ms(read),
            ms(textures.decode_time),
            ms(work),
            jobs.len(),
            ms(longest_job),
            ms(started.elapsed() - read - work),
            ms(carry_time),
            ms(started.elapsed())
        );
    }
}

/// Vertices per run of the carried light's work on the threads.
const CARRY_RUN: usize = 4096;

/// Each vertex's carried light: the colour and, where the original's
/// triangle has one, the lightmap.
type Carried = Vec<(Vec3, Option<f32>)>;

/// A part's texture address modes as TEXDEF flags (0x4 U clamp, 0x8 V).
fn address_flags(p: &NewPart) -> u16 {
    (if p.clamp.0 { 4 } else { 0 }) | (if p.clamp.1 { 8 } else { 0 })
}

/// The materials.tsv line of a part's material, unless none or type none.
fn detail_line(table: &HashMap<String, material_maps::Line>, p: &NewPart) -> Option<material_maps::Line> {
    p.material.as_deref().and_then(|n| table.get(n)).filter(|l| l.kind != material_maps::Kind::None).cloned()
}

/// A texture's work for the threads: its mipmap chain, or a detail map
/// (the address flags, the line, the material's name).
enum Job {
    Mips(Texture),
    Detail(Texture, u16, material_maps::Line, String),
}

enum Done {
    Mips(Vec<Vec<u8>>),
    Detail(Box<material_maps::Detail>),
}

/// `f` over `jobs` on up to the machine's threads (pure work), the results
/// in the jobs' order.
fn run_parallel<J: Sync, R: Send>(jobs: &[J], f: impl Fn(&J) -> R + Sync) -> Vec<R> {
    let threads = std::thread::available_parallelism().map_or(1, |n| n.get()).min(jobs.len());
    if threads <= 1 {
        return jobs.iter().map(f).collect();
    }
    let next = std::sync::atomic::AtomicUsize::new(0);
    let mut out: Vec<Option<R>> = (0..jobs.len()).map(|_| None).collect();
    std::thread::scope(|s| {
        let workers: Vec<_> = (0..threads)
            .map(|_| {
                s.spawn(|| {
                    let mut mine = Vec::new();
                    loop {
                        let i = next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        let Some(job) = jobs.get(i) else { break };
                        mine.push((i, f(job)));
                    }
                    mine
                })
            })
            .collect();
        for w in workers {
            for (i, r) in w.join().expect("a model pack texture thread") {
                out[i] = Some(r);
            }
        }
    });
    out.into_iter().map(|r| r.expect("every job ran")).collect()
}

/// A materials.tsv line as a key: its type and values.
type DetailKey = (material_maps::Kind, Option<u32>, Option<u32>, Option<u32>);

/// An object's parts drawn with one material: the texture, its address
/// modes, the alpha class, double-sidedness, the detail line and whether
/// the carried light has a lightmap (the ramp).
type MadeKey = (u64, u16, u8, bool, Option<DetailKey>, bool);

fn detail_key(l: &material_maps::Line) -> DetailKey {
    (l.kind, l.bump.map(f32::to_bits), l.rough.map(f32::to_bits), l.metal.map(f32::to_bits))
}

/// A package folder's materials.tsv (none: an empty table); lines not
/// understood are reported.
fn read_table(folder: &Path, package: &str) -> HashMap<String, material_maps::Line> {
    let path = folder.join(material_maps::TABLE_FILE);
    if !path.is_file() {
        return HashMap::new();
    }
    match crate::bounded_read(&path) {
        Ok(bytes) => {
            // Names as the files' materials have them (any word up to 128
            // bytes; case kept).
            let (table, bad) = material_maps::parse_named(&String::from_utf8_lossy(&bytes), |name| name.len() <= 128, false);
            if 0 < bad {
                report(format!("{package}/{}: {bad} lines not understood", material_maps::TABLE_FILE));
            }
            table
        }
        Err(e) => {
            report(format!("{package}/{}: {e}", material_maps::TABLE_FILE));
            HashMap::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Synthetic: a one-triangle glb with a node scale, a 2×1 PNG and
    /// BLEND reads back in the original's space (x negated) with GS alpha.
    #[test]
    fn reads_a_minimal_glb() {
        let png = {
            let mut out = Vec::new();
            {
                let mut e = png::Encoder::new(&mut out, 2, 1);
                e.set_color(png::ColorType::Rgba);
                e.set_depth(png::BitDepth::Eight);
                let mut w = e.write_header().unwrap();
                w.write_image_data(&[255, 0, 0, 255, 0, 255, 0, 0]).unwrap();
            }
            out
        };
        let mut bin = Vec::new();
        for v in [[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]] {
            bin.extend(v.iter().flat_map(|f| f.to_le_bytes()));
        }
        for _ in 0..3 {
            bin.extend([0.0f32, 0.0, 1.0].iter().flat_map(|f| f.to_le_bytes()));
        }
        bin.extend([0u16, 1, 2].iter().flat_map(|i| i.to_le_bytes()));
        bin.extend([0, 0]);
        let png_at = bin.len();
        bin.extend(&png);
        while !bin.len().is_multiple_of(4) {
            bin.push(0);
        }
        let doc = serde_json::json!({
            "asset": {"version": "2.0"},
            "scene": 0, "scenes": [{"nodes": [0]}],
            "nodes": [{"mesh": 0, "scale": [2.0, 2.0, 2.0]}],
            "meshes": [{"primitives": [{"attributes": {"POSITION": 0, "NORMAL": 1}, "indices": 2, "material": 0}]}],
            "materials": [{"name": "GOLD_HEAP", "alphaMode": "BLEND", "doubleSided": true, "pbrMetallicRoughness": {"baseColorTexture": {"index": 0}}}],
            "textures": [{"source": 0}],
            "images": [{"bufferView": 3, "mimeType": "image/png"}],
            "accessors": [
                {"bufferView": 0, "componentType": 5126, "count": 3, "type": "VEC3"},
                {"bufferView": 1, "componentType": 5126, "count": 3, "type": "VEC3"},
                {"bufferView": 2, "componentType": 5123, "count": 3, "type": "SCALAR"}
            ],
            "bufferViews": [
                {"buffer": 0, "byteOffset": 0, "byteLength": 36},
                {"buffer": 0, "byteOffset": 36, "byteLength": 36},
                {"buffer": 0, "byteOffset": 72, "byteLength": 6},
                {"buffer": 0, "byteOffset": png_at, "byteLength": png.len()}
            ],
            "buffers": [{"byteLength": bin.len()}]
        });
        let mut json = serde_json::to_vec(&doc).unwrap();
        while !json.len().is_multiple_of(4) {
            json.push(b' ');
        }
        let mut glb = Vec::new();
        glb.extend(b"glTF");
        glb.extend(2u32.to_le_bytes());
        glb.extend(((12 + 8 + json.len() + 8 + bin.len()) as u32).to_le_bytes());
        glb.extend((json.len() as u32).to_le_bytes());
        glb.extend(b"JSON");
        glb.extend(&json);
        glb.extend((bin.len() as u32).to_le_bytes());
        glb.extend(b"BIN\0");
        glb.extend(&bin);
        let parts = read_glb(&glb).unwrap();
        assert_eq!(parts.len(), 1);
        let p = &parts[0];
        assert_eq!(p.positions, vec![[-0.0, 0.0, 0.0], [-2.0, 0.0, 0.0], [-0.0, 2.0, 0.0]]);
        assert_eq!(p.normals[0], [-0.0, 0.0, 1.0]);
        assert_eq!(p.indices, vec![0, 1, 2]);
        assert_eq!(p.uvs, vec![[0.0, 0.0]; 3]);
        assert_eq!((p.class, p.double_sided), (2, true));
        assert_eq!(p.material.as_deref(), Some("GOLD_HEAP"));
        assert_eq!(*p.texture, (2, 1, vec![255, 0, 0, 128, 0, 255, 0, 0]));
        // Files embedding one image share its decoding.
        let mut textures = Textures::default();
        let a = read_glb_with(&glb, &mut textures, None).unwrap();
        let b = read_glb_with(&glb, &mut textures, None).unwrap();
        assert!(Arc::ptr_eq(&a[0].texture, &b[0].texture) && a[0].texture_key == p.texture_key);
        // The same file with its buffer and image beside it (share_data.py):
        // the same parts, the image shared with the embedded one; a path
        // out of the folder, or no folder, is refused.
        let dir = std::env::temp_dir().join(format!("gdl-model-pack-test-{}", std::process::id()));
        std::fs::create_dir_all(dir.join("data")).unwrap();
        std::fs::create_dir_all(dir.join("textures")).unwrap();
        std::fs::write(dir.join("data").join("g.bin"), &bin).unwrap();
        std::fs::write(dir.join("textures").join("t.png"), &png).unwrap();
        let beside = |buffer: &str, image: &str| {
            let mut d = doc.clone();
            d["buffers"][0]["uri"] = serde_json::json!(buffer);
            d["images"][0] = serde_json::json!({"uri": image});
            let mut j = serde_json::to_vec(&d).unwrap();
            while !j.len().is_multiple_of(4) {
                j.push(b' ');
            }
            let mut g = Vec::new();
            g.extend(b"glTF");
            g.extend(2u32.to_le_bytes());
            g.extend(((12 + 8 + j.len()) as u32).to_le_bytes());
            g.extend((j.len() as u32).to_le_bytes());
            g.extend(b"JSON");
            g.extend(&j);
            g
        };
        let outside = beside("data/g.bin", "textures/t.png");
        let c = read_glb_with(&outside, &mut textures, Some(&dir)).unwrap();
        assert_eq!((&c[0].positions, &c[0].indices), (&p.positions, &p.indices));
        assert!(Arc::ptr_eq(&a[0].texture, &c[0].texture));
        assert!(read_glb(&outside).is_err_and(|e| e.contains("outside the file")));
        for bad in ["../t.png", "textures/../t.png", "C:/t.png", "/t.png", "textures\\t.png", "data:t.png", "http://host/t.png"] {
            assert!(read_glb_with(&beside("data/g.bin", bad), &mut Textures::default(), Some(&dir)).is_err_and(|e| e.contains("plain relative path")), "{bad}");
        }
        let _ = std::fs::remove_dir_all(&dir);
        // A JPEG image is refused with a message.
        let at = glb.windows(9).position(|w| w == b"image/png").unwrap();
        glb[at..at + 9].copy_from_slice(b"image/jpg");
        assert!(read_glb(&glb).is_err_and(|e| e.contains("only PNG")));
    }

    #[test]
    fn a_folder_signature_follows_its_files() {
        let dir = std::env::temp_dir().join(format!("gdl-model-pack-sig-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("textures")).unwrap();
        let empty = folder_signature(&dir);
        std::fs::write(dir.join("A.glb"), b"one").unwrap();
        let one = folder_signature(&dir);
        assert_ne!(empty, one);
        assert_eq!(one, folder_signature(&dir), "unchanged: the same");
        // A file beside it (textures/), a longer file, removed files.
        std::fs::write(dir.join("textures").join("t.png"), b"t").unwrap();
        let two = folder_signature(&dir);
        assert_ne!(one, two);
        std::fs::write(dir.join("A.glb"), b"one more").unwrap();
        assert_ne!(two, folder_signature(&dir));
        std::fs::remove_file(dir.join("textures").join("t.png")).unwrap();
        std::fs::remove_file(dir.join("A.glb")).unwrap();
        assert_eq!(empty, folder_signature(&dir));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
