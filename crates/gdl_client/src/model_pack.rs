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
//! binary with its buffer in the file; triangle primitives with POSITION
//! and NORMAL (float), TEXCOORD_0 (float, or normalised unsigned byte or
//! short), indices of any unsigned size; node translation, rotation, scale
//! or matrix; a base colour texture as an embedded PNG and a base colour
//! factor; alphaMode; doubleSided. Space: the model export's (x negated
//! against the original, y up, one game unit per metre), so the triangle
//! order is kept.
//!
//! Light: a replacement is lit live, as the game lights items (the
//! level's sun and ambient, present_scene_light), wherever it stands. The
//! baked light of the scenery around it (models-plan option A) comes later.
use crate::{model_export, mip_image, texfilter, GsMaterial};
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
\r
Read when the game starts. Collision and gameplay stay the original's. Delete\r
a file to get the original back; F10 > Graphics > Model pack folder turns the\r
folder off.\r
";

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
}

type R<T> = Result<T, String>;

/// A decoded texture: width, height and RGBA with GS alpha.
pub type Texture = Arc<(u32, u32, Vec<u8>)>;

/// Decoded textures by their key ([`NewPart::texture_key`]): the files of a
/// package that embed one image (a flip-book's frames from
/// tools/blender/break_frames.py) decode it once.
#[derive(Default)]
pub struct Textures(HashMap<u64, Texture>);

/// A file's parts.
pub fn read_glb(bytes: &[u8]) -> R<Vec<NewPart>> {
    read_glb_with(bytes, &mut Textures::default())
}

/// A file's parts, with textures shared through `textures`.
pub fn read_glb_with(bytes: &[u8], textures: &mut Textures) -> R<Vec<NewPart>> {
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
    if let Some(buffers) = doc["buffers"].as_array()
        && buffers.iter().any(|b| b.get("uri").is_some())
    {
        return Err("a buffer outside the file (only .glb with its data inside)".into());
    }
    let g = Gltf { doc: &doc, bin, textures: RefCell::new(std::mem::take(&mut textures.0)) };
    let scene = doc["scene"].as_u64().unwrap_or(0) as usize;
    let roots: Vec<usize> = match doc["scenes"].get(scene) {
        Some(s) => s["nodes"].as_array().map(|n| n.iter().filter_map(|v| v.as_u64().map(|v| v as usize)).collect()).unwrap_or_default(),
        None => (0..doc["nodes"].as_array().map_or(0, Vec::len)).collect(),
    };
    let mut out = Vec::new();
    let mut vertices = 0;
    let walked = roots.into_iter().try_for_each(|root| g.node(root, Mat4::IDENTITY, 0, &mut out, &mut vertices));
    textures.0 = g.textures.into_inner();
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
    textures: RefCell<HashMap<u64, Texture>>,
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
            let (texture, texture_key, class, double_sided, clamp) = self.material(p["material"].as_u64().map(|m| m as usize))?;
            out.push(NewPart { positions, normals, uvs, indices, texture, texture_key, class, double_sided, clamp });
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
                let img = self.doc["images"].get(tex["source"].as_u64().ok_or("a texture without an image")? as usize).ok_or("image index")?;
                if img["mimeType"].as_str() != Some("image/png") {
                    return Err(format!("a {} texture: only PNG is read (fit_model.py writes PNG)", img["mimeType"].as_str().unwrap_or("non-embedded")));
                }
                Some(self.view(img["bufferView"].as_u64().ok_or("an image outside the file")? as usize)?)
            }
            None => None,
        };
        let mut hash = DefaultHasher::new();
        hash.write_u8(class);
        for f in factor {
            hash.write_u32(f.to_bits());
        }
        if let Some(bytes) = png {
            hash.write_usize(bytes.len());
            hash.write(bytes);
        }
        let key = hash.finish();
        let double_sided = m["doubleSided"].as_bool().unwrap_or(false);
        if let Some(t) = self.textures.borrow().get(&key) {
            return Ok((t.clone(), key, class, double_sided, clamp));
        }
        let (w, h, mut rgba) = match png {
            Some(bytes) => decode_png(bytes)?,
            None => (1, 1, vec![255; 4]),
        };
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
    let mut uploaded: HashMap<(u64, u16), Handle<Image>> = HashMap::new();
    for (index, stem, label, _) in model_export::object_files(objects) {
        let path = folder.join(format!("{stem}.glb"));
        if !path.is_file() {
            continue;
        }
        let Some(original) = parts.get(&index).and_then(|p| p.first()).and_then(|(_, m)| materials.get(m)).cloned() else {
            report(format!("{package}/{label}: the original has no drawn part; not replaced"));
            continue;
        };
        let mut made: HashMap<(u64, u16, u8, bool), Handle<GsMaterial>> = HashMap::new();
        let new = match crate::bounded_read(&path).map_err(|e| e.to_string()).and_then(|b| read_glb_with(&b, &mut textures)) {
            Ok(n) => n,
            Err(e) => {
                report(format!("{package}/{label}: {e}; the original stays"));
                continue;
            }
        };
        let mut list = Vec::new();
        let mut triangles = 0;
        for p in new {
            triangles += p.indices.len() / 3;
            let n = p.positions.len();
            let mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
                .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, p.positions)
                .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, p.normals)
                .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, p.uvs)
                .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, vec![[1.0f32; 4]; n])
                .with_inserted_indices(Indices::U32(p.indices));
            let flags: u16 = if p.clamp.0 { 4 } else { 0 } | if p.clamp.1 { 8 } else { 0 };
            let shared = (p.texture_key, flags, p.class, p.double_sided);
            if let Some(m) = made.get(&shared) {
                list.push((meshes.add(mesh), m.clone()));
                continue;
            }
            let image = uploaded
                .entry((p.texture_key, flags))
                .or_insert_with(|| {
                    let (w, h, rgba) = &*p.texture;
                    let key = texfilter::TexKey { flags, levels: 1, animated: false, borrowed: false };
                    let levels = texfilter::with_chain(*w, *h, vec![rgba.clone()], sharp);
                    let image = images.add(mip_image(*w as u16, *h as u16, levels, texfilter::sampler(key, sharp)));
                    texfilter::register(image.id(), key);
                    image
                })
                .clone();
            let mut m = original.clone();
            m.base = Some(image);
            m.lightmap = None;
            m.overlay = None;
            m.detail = None;
            m.lit = true;
            m.params.options.x = 1.0;
            m.params.options.y = 0.0;
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
            let m = materials.add(m);
            made.insert(shared, m.clone());
            list.push((meshes.add(mesh), m));
        }
        report(format!("{package}/{label}: {triangles} triangles in {} parts from {}", list.len(), path.display()));
        parts.insert(index, list);
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
            "materials": [{"alphaMode": "BLEND", "doubleSided": true, "pbrMetallicRoughness": {"baseColorTexture": {"index": 0}}}],
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
        assert_eq!(*p.texture, (2, 1, vec![255, 0, 0, 128, 0, 255, 0, 0]));
        // Files embedding one image share its decoding.
        let mut textures = Textures::default();
        let a = read_glb_with(&glb, &mut textures).unwrap();
        let b = read_glb_with(&glb, &mut textures).unwrap();
        assert!(Arc::ptr_eq(&a[0].texture, &b[0].texture) && a[0].texture_key == p.texture_key);
        // A JPEG image is refused with a message.
        let at = glb.windows(9).position(|w| w == b"image/png").unwrap();
        glb[at..at + 9].copy_from_slice(b"image/jpg");
        assert!(read_glb(&glb).is_err_and(|e| e.contains("only PNG")));
    }
}
