//! NATIVE (this fork, decisions/0021; docs/remaster/models-plan.md M4): the
//! model exporter. It writes the original models as glTF 2.0 binary files
//! (.glb) that Blender and other tools open: every object of a package in a
//! file of its own, and each level's scene — its placed objects, with the
//! collision and the water surfaces as layers of their own. Nothing here
//! changes the game: it reads the installed data and writes new files.
//!
//! - Space: the original coordinates with x negated (the renderer's
//!   MIRROR), so the files are right-handed with y up, as glTF wants, at one
//!   game unit per metre. The triangle order is kept: Bevy draws these same
//!   mirrored positions with counter-clockwise front faces, glTF's rule.
//! - Geometry: each part as the renderer builds it (crate::part_geometry),
//!   one primitive per part.
//! - Textures: level 0 of each record as PNG, alpha doubled to 0..255 as the
//!   texture dump writes it. Each material's extras give the texture's name
//!   as the texture pack keys it (16 hex digits) and its TEXDEF label.
//! - Lightmaps: the occlusion texture, on TEXCOORD_1, grey index / 16 (the
//!   lightmap pass multiplies by 8 × index / 128).
//! - Vertex colours: COLOR_0 (RGB) on the GS 0x80 = 1.0 scale, and the GS
//!   vertex alpha in _GDL_VERTEX_ALPHA (0 or 1).
//! - Collision: the triangles as the collision code places them
//!   (gdl_core::collision::Triangle::corners), dynamic objects at their rest
//!   translation. Water objects (flag 0x200) go in the WATER layer.
//! - manifest.json lists every file written, with the SHA-256 of each file
//!   and of the sources it came from.
use crate::{part_geometry, record_texdef, texa_alpha, texture_pack};
use gdl_formats::{objects::Objects, texdef::TextureTable, world::World};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    error::Error,
    path::{Path, PathBuf},
};

/// The exporter's version (manifest.json and each file's asset.generator).
pub const VERSION: &str = "gdl model export 1";

type R<T> = Result<T, Box<dyn Error>>;

fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes).iter().map(|b| format!("{b:02x}")).collect()
}

/// The renderer's handedness mirror (gdl_play MIRROR).
fn mirror(v: [f32; 3]) -> [f32; 3] {
    [-v[0], v[1], v[2]]
}

/// A glTF binary file being built: the JSON arrays and the one buffer.
#[derive(Default)]
struct Glb {
    bin: Vec<u8>,
    views: Vec<Value>,
    accessors: Vec<Value>,
    images: Vec<Value>,
    samplers: Vec<Value>,
    textures: Vec<Value>,
    materials: Vec<Value>,
    meshes: Vec<Value>,
    nodes: Vec<Value>,
    roots: Vec<usize>,
}

impl Glb {
    fn view(&mut self, bytes: &[u8], target: Option<u32>) -> usize {
        while !self.bin.len().is_multiple_of(4) {
            self.bin.push(0);
        }
        let mut v = json!({"buffer": 0, "byteOffset": self.bin.len(), "byteLength": bytes.len()});
        if let Some(t) = target {
            v["target"] = json!(t);
        }
        self.bin.extend_from_slice(bytes);
        self.views.push(v);
        self.views.len() - 1
    }

    /// A float accessor (SCALAR, VEC2, VEC3 or VEC4); positions carry bounds.
    fn floats<const N: usize>(&mut self, data: &[[f32; N]], bounds: bool) -> usize {
        let bytes: Vec<u8> = data.iter().flatten().flat_map(|f| f.to_le_bytes()).collect();
        let view = self.view(&bytes, Some(34962));
        let ty = match N {
            1 => "SCALAR",
            2 => "VEC2",
            3 => "VEC3",
            _ => "VEC4",
        };
        let mut a = json!({"bufferView": view, "componentType": 5126, "count": data.len(), "type": ty});
        if bounds {
            let min: Vec<f32> = (0..N).map(|k| data.iter().map(|v| v[k]).fold(f32::INFINITY, f32::min)).collect();
            let max: Vec<f32> = (0..N).map(|k| data.iter().map(|v| v[k]).fold(f32::NEG_INFINITY, f32::max)).collect();
            a["min"] = json!(min);
            a["max"] = json!(max);
        }
        self.accessors.push(a);
        self.accessors.len() - 1
    }

    fn indices(&mut self, data: &[u32]) -> usize {
        let bytes: Vec<u8> = data.iter().flat_map(|i| i.to_le_bytes()).collect();
        let view = self.view(&bytes, Some(34963));
        self.accessors.push(json!({"bufferView": view, "componentType": 5125, "count": data.len(), "type": "SCALAR"}));
        self.accessors.len() - 1
    }

    fn image_png(&mut self, png: &[u8], name: &str) -> usize {
        let view = self.view(png, None);
        self.images.push(json!({"bufferView": view, "mimeType": "image/png", "name": name}));
        self.images.len() - 1
    }

    fn node(&mut self, node: Value) -> usize {
        self.nodes.push(node);
        self.nodes.len() - 1
    }

    /// The file's bytes: the JSON chunk (space padded) and the BIN chunk.
    fn bytes(&self, extras: Value) -> R<Vec<u8>> {
        let mut doc = json!({
            "asset": {"version": "2.0", "generator": VERSION, "extras": extras},
            "scene": 0,
            "scenes": [{"nodes": self.roots}],
        });
        // glTF arrays, when present, must not be empty.
        for (key, list) in [
            ("nodes", &self.nodes),
            ("meshes", &self.meshes),
            ("materials", &self.materials),
            ("textures", &self.textures),
            ("samplers", &self.samplers),
            ("images", &self.images),
            ("accessors", &self.accessors),
            ("bufferViews", &self.views),
        ] {
            if !list.is_empty() {
                doc[key] = Value::Array(list.clone());
            }
        }
        let mut bin = self.bin.clone();
        while !bin.len().is_multiple_of(4) {
            bin.push(0);
        }
        if !bin.is_empty() {
            doc["buffers"] = json!([{"byteLength": bin.len()}]);
        }
        let mut text = serde_json::to_vec(&doc)?;
        while !text.len().is_multiple_of(4) {
            text.push(b' ');
        }
        let total = 12 + 8 + text.len() + if bin.is_empty() { 0 } else { 8 + bin.len() };
        let mut out = Vec::with_capacity(total);
        out.extend_from_slice(b"glTF");
        out.extend_from_slice(&2u32.to_le_bytes());
        out.extend_from_slice(&u32::try_from(total)?.to_le_bytes());
        out.extend_from_slice(&u32::try_from(text.len())?.to_le_bytes());
        out.extend_from_slice(b"JSON");
        out.extend_from_slice(&text);
        if !bin.is_empty() {
            out.extend_from_slice(&u32::try_from(bin.len())?.to_le_bytes());
            out.extend_from_slice(b"BIN\0");
            out.extend_from_slice(&bin);
        }
        Ok(out)
    }
}

/// A PNG in memory (8 bits per channel).
fn png(width: u32, height: u32, color: png::ColorType, data: &[u8]) -> R<Vec<u8>> {
    let mut out = Vec::new();
    {
        let mut e = png::Encoder::new(&mut out, width, height);
        e.set_color(color);
        e.set_depth(png::BitDepth::Eight);
        let mut w = e.write_header()?;
        w.write_image_data(data)?;
        w.finish()?;
    }
    Ok(out)
}

/// One decoded base texture of a package.
struct Texture {
    png: Vec<u8>,
    /// 0 opaque, 1 cut-out, 2 blended (materials_for's rule).
    class: u8,
    /// The texture pack's name for it.
    name: String,
}

/// A package's decoded images by bitmap record.
struct Images {
    base: HashMap<u16, Texture>,
    lightmaps: HashMap<u16, Vec<u8>>,
    /// Every record's TEXDEF label (the first candidate's), decoded or not.
    labels: HashMap<u16, String>,
    flags: HashMap<u16, u16>,
}

fn package_images(objects: &Objects, table: &TextureTable, textures: &[u8]) -> Images {
    let mut out = Images { base: HashMap::new(), lightmaps: HashMap::new(), labels: HashMap::new(), flags: HashMap::new() };
    for b in &objects.bitmap_records {
        let k = b.index as u16;
        out.flags.insert(k, b.flags);
        if b.format == 146 {
            if let Ok(indices) = b.decode_lightmap_indices(textures) {
                let grey: Vec<u8> = indices.iter().map(|&i| ((u32::from(i) * 255 + 8) / 16).min(255) as u8).collect();
                if let Ok(p) = png(b.width.into(), b.height.into(), png::ColorType::Grayscale, &grey) {
                    out.lightmaps.insert(k, p);
                }
            }
            continue;
        }
        if let Some(t) = table.bitmaps.iter().find(|t| t.texture_offset == b.texture_offset && t.format == b.format && t.width == b.width && t.height == b.height) {
            out.labels.insert(k, t.label.clone());
        }
        let Some(def) = record_texdef(table, b) else { continue };
        let Ok(mut rgba) = def.decode_level_rgba(textures, 0) else { continue };
        texa_alpha(b.format, &mut rgba);
        let class = if rgba.chunks_exact(4).all(|p| p[3] >= 0x80) {
            0
        } else if rgba.chunks_exact(4).all(|p| p[3] <= 1 || p[3] >= 0x80) {
            1
        } else {
            2
        };
        let name = texture_pack::name(b.width, b.height, &rgba);
        let doubled: Vec<u8> = rgba.chunks_exact(4).flat_map(|p| [p[0], p[1], p[2], (u16::from(p[3]) * 2).min(255) as u8]).collect();
        if let Ok(p) = png(b.width.into(), b.height.into(), png::ColorType::Rgba, &doubled) {
            out.base.insert(k, Texture { png: p, class, name });
        }
    }
    out
}

/// Triangle and part counts of what went into a file.
#[derive(Default, Clone, Copy)]
struct Stats {
    parts: usize,
    triangles: usize,
}

/// A file being written from one package's images: textures, materials and
/// object meshes are added once each.
struct Writer<'a> {
    glb: Glb,
    images: &'a Images,
    textures: HashMap<u16, usize>,
    lightmaps: HashMap<u16, usize>,
    samplers: HashMap<u16, usize>,
    materials: HashMap<(u16, Option<u16>), usize>,
    meshes: HashMap<u32, (usize, Stats)>,
    unsupported: Vec<String>,
}

impl<'a> Writer<'a> {
    fn new(images: &'a Images) -> Self {
        Self {
            glb: Glb::default(),
            images,
            textures: HashMap::new(),
            lightmaps: HashMap::new(),
            samplers: HashMap::new(),
            materials: HashMap::new(),
            meshes: HashMap::new(),
            unsupported: Vec::new(),
        }
    }

    /// The sampler for record flags: 0x4 clamps U, 0x8 clamps V, else repeat.
    fn sampler(&mut self, flags: u16) -> usize {
        let key = flags & 0xc;
        if let Some(&s) = self.samplers.get(&key) {
            return s;
        }
        let wrap = |clamp: bool| if clamp { 33071 } else { 10497 };
        self.glb.samplers.push(json!({"magFilter": 9729, "minFilter": 9987, "wrapS": wrap(flags & 4 != 0), "wrapT": wrap(flags & 8 != 0)}));
        let s = self.glb.samplers.len() - 1;
        self.samplers.insert(key, s);
        s
    }

    fn texture(&mut self, record: u16) -> Option<usize> {
        if let Some(&t) = self.textures.get(&record) {
            return Some(t);
        }
        let tex = self.images.base.get(&record)?;
        let image = self.glb.image_png(&tex.png, &tex.name);
        let sampler = self.sampler(self.images.flags.get(&record).copied().unwrap_or(0));
        self.glb.textures.push(json!({"source": image, "sampler": sampler}));
        let t = self.glb.textures.len() - 1;
        self.textures.insert(record, t);
        Some(t)
    }

    fn lightmap(&mut self, record: u16) -> Option<usize> {
        if let Some(&t) = self.lightmaps.get(&record) {
            return Some(t);
        }
        let png = self.images.lightmaps.get(&record)?;
        let image = self.glb.image_png(png, &format!("lightmap {record}"));
        let sampler = self.sampler(0);
        self.glb.textures.push(json!({"source": image, "sampler": sampler}));
        let t = self.glb.textures.len() - 1;
        self.lightmaps.insert(record, t);
        Some(t)
    }

    fn material(&mut self, record: u16, lightmap: Option<u16>) -> usize {
        if let Some(&m) = self.materials.get(&(record, lightmap)) {
            return m;
        }
        let label = self.images.labels.get(&record).cloned();
        let texture = self.texture(record);
        let mut m = json!({
            "name": label.clone().unwrap_or_else(|| format!("record {record}")),
            "pbrMetallicRoughness": {"metallicFactor": 0.0, "roughnessFactor": 1.0},
        });
        let mut extras = json!({"record": record, "label": label});
        match (texture, self.images.base.get(&record)) {
            (Some(t), Some(tex)) => {
                m["pbrMetallicRoughness"]["baseColorTexture"] = json!({"index": t});
                match tex.class {
                    0 => {}
                    1 => {
                        m["alphaMode"] = json!("MASK");
                        m["alphaCutoff"] = json!(0.5);
                    }
                    _ => m["alphaMode"] = json!("BLEND"),
                }
                extras["texture"] = json!(tex.name);
                extras["alpha_class"] = json!(tex.class);
            }
            _ => {
                // No texels of its own (a texture modifier's target, or an
                // undecoded record): the game's magenta placeholder.
                m["pbrMetallicRoughness"]["baseColorFactor"] = json!([1.0, 0.0, 0.8, 1.0]);
                extras["texture"] = Value::Null;
            }
        }
        if let Some(l) = lightmap.and_then(|l| self.lightmap(l)) {
            m["occlusionTexture"] = json!({"index": l, "texCoord": 1});
            extras["lightmap_record"] = json!(lightmap);
        }
        m["extras"] = json!({"gdl": extras});
        self.glb.materials.push(m);
        let k = self.glb.materials.len() - 1;
        self.materials.insert((record, lightmap), k);
        k
    }

    /// The object's mesh (one primitive per part), once per file; None when
    /// no part decodes.
    fn object_mesh(&mut self, objects: &Objects, index: u32, name: &str) -> Option<(usize, Stats)> {
        if let Some(&m) = self.meshes.get(&index) {
            return Some(m);
        }
        let object = objects.objects.iter().find(|o| o.index == index)?;
        let mut primitives = Vec::new();
        let mut stats = Stats::default();
        for part in &object.parts {
            let g = match part_geometry(part) {
                Ok(g) if !g.indices.is_empty() => g,
                Ok(_) => continue,
                Err(e) => {
                    self.unsupported.push(format!("{name} part {}: {e}", part.index));
                    continue;
                }
            };
            let positions: Vec<[f32; 3]> = g.positions.iter().map(|&p| mirror(p)).collect();
            let normals: Vec<[f32; 3]> = g.normals.iter().map(|&n| mirror(n)).collect();
            let mut attributes = json!({
                "POSITION": self.glb.floats(&positions, true),
                "NORMAL": self.glb.floats(&normals, false),
                "TEXCOORD_0": self.glb.floats(&g.uvs, false),
            });
            let lightmap = g.lightmap_uvs.as_ref().filter(|_| self.images.lightmaps.contains_key(&part.lightmap_index)).map(|uvs| {
                attributes["TEXCOORD_1"] = json!(self.glb.floats(uvs, false));
                part.lightmap_index
            });
            if g.coloured {
                let rgb: Vec<[f32; 3]> = g.colors.iter().map(|c| [c[0], c[1], c[2]]).collect();
                let alpha: Vec<[f32; 1]> = g.colors.iter().map(|c| [c[3]]).collect();
                attributes["COLOR_0"] = json!(self.glb.floats(&rgb, false));
                attributes["_GDL_VERTEX_ALPHA"] = json!(self.glb.floats(&alpha, false));
            }
            let indices = self.glb.indices(&g.indices);
            let material = self.material(part.texture_index, lightmap);
            primitives.push(json!({"attributes": attributes, "indices": indices, "material": material, "extras": {"gdl": {"part": part.index, "lod_k": part.lod_k}}}));
            stats.parts += 1;
            stats.triangles += g.indices.len() / 3;
        }
        if primitives.is_empty() {
            return None;
        }
        self.glb.meshes.push(json!({"name": name, "primitives": primitives}));
        let m = (self.glb.meshes.len() - 1, stats);
        self.meshes.insert(index, m);
        Some(m)
    }

    /// A layer material of the scene (collision, water): see-through and
    /// double sided.
    fn layer_material(&mut self, name: &str, colour: [f32; 4]) -> usize {
        self.glb.materials.push(json!({
            "name": name,
            "pbrMetallicRoughness": {"baseColorFactor": colour, "metallicFactor": 0.0, "roughnessFactor": 1.0},
            "alphaMode": "BLEND",
            "doubleSided": true,
        }));
        self.glb.materials.len() - 1
    }
}

/// One package's files to read (paths as the configuration gives them).
pub struct PackageSource {
    /// The folder name: a level's name, or the configuration's package key.
    pub name: String,
    pub objects: PathBuf,
    pub texdef: PathBuf,
    pub textures: PathBuf,
}

/// A level: its own package and its WORLDS file.
pub struct LevelSource {
    pub package: PackageSource,
    pub worlds: PathBuf,
    /// The level's title (config "title").
    pub title: String,
}

/// A loaded package with its sources' hashes.
struct Loaded {
    objects: Objects,
    images: Images,
    sources: Vec<(String, String)>,
}

fn load(p: &PackageSource) -> R<Loaded> {
    let mut sources = Vec::new();
    let mut read = |path: &Path| -> R<Vec<u8>> {
        let b = crate::bounded_read(path).map_err(|e| format!("{}: {e}", path.display()))?;
        sources.push((path.display().to_string(), sha256(&b)));
        Ok(b)
    };
    let objects = Objects::parse(&read(&p.objects)?)?;
    let table = TextureTable::parse(&read(&p.texdef)?)?;
    let textures = read(&p.textures)?;
    let images = package_images(&objects, &table, &textures);
    Ok(Loaded { objects, images, sources })
}

/// A file name for a label: characters Windows refuses become '_', and
/// device names (CON, AUX, …) get a trailing '_'.
fn file_stem(label: &str) -> String {
    let mut s: String = label.chars().map(|c| if c.is_ascii_alphanumeric() || "-_#+(). ".contains(c) { c } else { '_' }).collect();
    while s.ends_with(['.', ' ']) {
        s.pop();
    }
    if s.is_empty() {
        s.push('_');
    }
    let upper = s.to_ascii_uppercase();
    let base = upper.split('.').next().unwrap_or("");
    let reserved = matches!(base, "CON" | "PRN" | "AUX" | "NUL")
        || ((base.starts_with("COM") || base.starts_with("LPT")) && base.len() == 4 && base.as_bytes()[3].is_ascii_digit());
    if reserved {
        s.push('_');
    }
    s
}

/// The package's folder: each '/' part of its name as a file stem.
fn package_dir(name: &str) -> PathBuf {
    name.split('/').map(file_stem).collect()
}

/// Every object's labels: its own definitions', else "<LABEL>+k" for the
/// k-th frame object after a definition with frames.
fn object_labels(objects: &Objects) -> BTreeMap<u32, Vec<String>> {
    let mut own: BTreeMap<u32, Vec<String>> = BTreeMap::new();
    for d in &objects.definitions {
        if let Ok(i) = u32::try_from(d.object_index) {
            own.entry(i).or_default().push(d.label.clone());
        }
    }
    let mut frames = BTreeMap::new();
    for d in &objects.definitions {
        let Ok(base) = u32::try_from(d.object_index) else { continue };
        for k in 1..u32::try_from(d.frames).unwrap_or(0) {
            if !own.contains_key(&(base + k)) {
                frames.entry(base + k).or_insert_with(|| vec![format!("{}+{k}", d.label)]);
            }
        }
    }
    own.extend(frames);
    own
}

/// One line of manifest.json.
fn entry(path: &Path, out: &Path, kind: &str, bytes: &[u8], fields: Value) -> Value {
    let rel = path.strip_prefix(out).unwrap_or(path).to_string_lossy().replace('\\', "/");
    let mut v = json!({"path": rel, "kind": kind, "sha256": sha256(bytes), "bytes": bytes.len()});
    if let (Some(v), Value::Object(f)) = (v.as_object_mut(), fields) {
        v.extend(f);
    }
    v
}

fn write(path: &Path, bytes: &[u8]) -> R<()> {
    if let Some(d) = path.parent() {
        std::fs::create_dir_all(d)?;
    }
    let tmp = path.with_extension("glb.tmp");
    std::fs::write(&tmp, bytes)?;
    std::fs::rename(&tmp, path)?;
    Ok(())
}

/// Every object of a package, one file each, in OUT/objects/<package>/.
fn export_objects(out: &Path, name: &str, p: &Loaded, manifest: &mut Vec<Value>, unsupported: &mut Vec<String>) -> R<()> {
    let dir = out.join("objects").join(package_dir(name));
    let labels = object_labels(&p.objects);
    let mut used = HashSet::new();
    for object in &p.objects.objects {
        let names = labels.get(&object.index).cloned().unwrap_or_default();
        let label = names.first().cloned().unwrap_or_else(|| format!("#{}", object.index));
        let mut w = Writer::new(&p.images);
        let found = w.object_mesh(&p.objects, object.index, &label);
        unsupported.append(&mut w.unsupported);
        let Some((mesh, stats)) = found else { continue };
        let node = w.glb.node(json!({
            "name": label,
            "mesh": mesh,
            "extras": {"gdl": {"package": name, "object": object.index, "labels": names, "radius": object.radius, "flags": object.flags}},
        }));
        w.glb.roots.push(node);
        let mut stem = file_stem(&label);
        if !used.insert(stem.to_ascii_lowercase()) {
            stem = format!("{stem}~{}", object.index);
            used.insert(stem.to_ascii_lowercase());
        }
        let path = dir.join(format!("{stem}.glb"));
        let bytes = w.glb.bytes(json!({"gdl": {"package": name, "object": object.index, "space": SPACE}}))?;
        write(&path, &bytes)?;
        manifest.push(entry(&path, out, "object", &bytes, json!({"package": name, "object": object.index, "labels": names, "parts": stats.parts, "triangles": stats.triangles})));
    }
    Ok(())
}

const SPACE: &str = "original x negated, y up, 1 unit = 1 m";

/// What a level's scene holds (manifest.json and the command's report).
#[derive(Default, Clone, Copy, Debug)]
pub struct SceneCounts {
    pub placed: usize,
    pub hidden: usize,
    pub collision_triangles: usize,
    pub water_triangles: usize,
    /// Triangles the static grid lists for an object beyond its node's range
    /// (exported with that object).
    pub grid_only_triangles: usize,
    /// Item instances' own triangles (not exported yet).
    pub item_triangles: usize,
    /// Triangle records that no node, grid entry or item instance uses
    /// (not exported).
    pub unreferenced_triangles: usize,
}

/// A level's scene, OUT/levels/<name>.glb: SCENERY (the placed objects as
/// spawn_level draws them), COLLISION and WATER.
fn export_scene(out: &Path, level: &LevelSource, p: &Loaded, world_bytes: &[u8], manifest: &mut Vec<Value>, unsupported: &mut Vec<String>) -> R<SceneCounts> {
    let name = &level.package.name;
    let world = World::parse(world_bytes)?;
    let collision = gdl_core::collision::CollisionWorld::from_world(&world).map_err(|e| format!("{name} collision: {e:?}"))?;
    let nodes = &world.nodes;
    let labels: HashMap<&str, u32> = p.objects.definitions.iter().filter_map(|d| Some((d.label.as_str(), u32::try_from(d.object_index).ok()?))).collect();
    let mut w = Writer::new(&p.images);
    let mut counts = SceneCounts::default();
    // CreateWorldNode's MB flags (spawn_level): record +0x18, | 4 for file
    // flags & 0x1001 == 1, | 1 (not drawn) for file flag 0x8000.
    let mb_flags = |i: usize| {
        let n = &nodes[i];
        let mut f = n.render_flags;
        if n.flags & 0x1001 == 1 {
            f |= 4;
        }
        if n.flags & 0x8000 != 0 {
            f |= 1;
        }
        f
    };
    let mut scenery = Vec::new();
    let placements = world.placements()?;
    let parent: HashMap<usize, usize> = placements.iter().filter_map(|p| p.parent.map(|q| (p.node_index as usize, q as usize))).collect();
    for pl in &placements {
        let i = pl.node_index as usize;
        let node = &nodes[i];
        // NewWorldObject: a model only on non-particle nodes with +0x28 == 1
        // (or file flag 0x80000000).
        let has_object = node.flags & 0x800 == 0 && (i32::from_le_bytes(node.raw[40..44].try_into()?) == 1 || node.flags & 0x80000000 != 0);
        if !has_object {
            continue;
        }
        let Some(&object) = labels.get(node.label.as_str()) else { continue };
        let Some((mesh, _)) = w.object_mesh(&p.objects, object, &node.label) else { continue };
        // pbTraverseDrawObjects: flag 2 hides a subtree, flag 1 the node.
        let mut hidden = mb_flags(i) & 3 != 0;
        let mut camera_facing = mb_flags(i) & 0xf000000 != 0;
        let mut k = parent.get(&i).copied();
        while let Some(a) = k {
            hidden |= mb_flags(a) & 2 != 0;
            camera_facing |= mb_flags(a) & 0xf000000 != 0;
            k = parent.get(&a).copied();
        }
        counts.placed += 1;
        counts.hidden += usize::from(hidden);
        let n = w.glb.node(json!({
            "name": node.label,
            "mesh": mesh,
            "translation": mirror(pl.summed_position),
            "extras": {"gdl": {"world_node": i, "object": object, "flags": node.flags, "mb_flags": mb_flags(i), "hidden": hidden, "camera_facing": camera_facing}},
        }));
        scenery.push(n);
    }
    unsupported.append(&mut w.unsupported);
    // The collision and water layers: one mesh per object with triangles,
    // three vertices each with the face normal.
    let solid = w.layer_material("GDL collision", [1.0, 0.35, 0.1, 0.4]);
    let wet = w.layer_material("GDL water", [0.1, 0.45, 1.0, 0.4]);
    // Each object's triangles: its node's range (+0x38, count +0x36) and
    // every triangle the static grid lists for it (an i16 offset from the
    // object's first triangle, WorldCollide's cell walk).
    let mut owned: BTreeMap<usize, std::collections::BTreeSet<usize>> = BTreeMap::new();
    for (i, node) in nodes.iter().enumerate() {
        if 0 < node.collision_count && 0 <= node.collision_index {
            let first = node.collision_index as usize;
            owned.entry(i).or_default().extend(first..first + node.collision_count as usize);
        }
    }
    let in_ranges: usize = owned.values().map(|s| s.len()).sum();
    let g = &world.grid;
    let read = |o: usize| g.contents.get(o..o + 2).map(|b| i16::from_le_bytes([b[0], b[1]]));
    for row in &g.rows {
        for gx in row.min_x..=row.max_x.max(row.min_x) {
            if row.max_x < row.min_x {
                break;
            }
            let Some(&cell) = g.cells.get(usize::from(gx - row.min_x) + row.cell_offset as usize) else { continue };
            let mut offset = (cell & 0x3f_ffff) as usize;
            for _ in 0..cell >> 22 {
                let (Some(object), Some(count)) = (read(offset), read(offset + 2)) else { break };
                if count == 0 {
                    continue;
                }
                let object = usize::try_from(object).map_err(|_| "grid object index")?;
                let first = collision.objects.get(object).ok_or("grid object index")?.first_triangle;
                for k in 0..count.max(0) as usize {
                    let t = read(offset + 4 + 2 * k).ok_or("grid contents")?;
                    let t = usize::try_from(first + i32::from(t)).map_err(|_| "grid triangle index")?;
                    owned.entry(object).or_default().insert(t);
                }
                offset += 6 + (count.max(1) as usize - 1) * 2;
            }
        }
    }
    let owned_total: usize = owned.values().map(|s| s.len()).sum();
    counts.grid_only_triangles = owned_total - in_ranges;
    let (mut col, mut water) = (Vec::new(), Vec::new());
    for (&i, list) in &owned {
        let node = &nodes[i];
        let ob = &collision.objects[i];
        let dynamic = ob.flags & 0x100_1000 != 0;
        let offset = if dynamic {
            let m = collision.object_rest_matrix(i).map_err(|e| format!("{name} object {i}: {e:?}"))?;
            [m[3][0], m[3][1], m[3][2]]
        } else {
            [0.0; 3]
        };
        let first = node.collision_index;
        let mut positions = Vec::new();
        let mut normals = Vec::new();
        for &t in list {
            let tri = collision.triangles.get(t).ok_or("collision triangle index")?;
            let c = tri.corners().map_err(|e| format!("{name} triangle {t}: {e:?}"))?;
            // (c1 − c0) × (c2 − c0) points along the normal (BTriLineCol's
            // edge tests); mirrored, c0, c2, c1 keep it the front face.
            for v in [c[0], c[2], c[1]] {
                positions.push(mirror([v[0] + offset[0], v[1] + offset[1], v[2] + offset[2]]));
                normals.push(mirror(tri.normal));
            }
        }
        let is_water = ob.flags & 0x200 != 0;
        let count = positions.len() / 3;
        let indices: Vec<u32> = (0..positions.len() as u32).collect();
        let attributes = json!({"POSITION": w.glb.floats(&positions, true), "NORMAL": w.glb.floats(&normals, false)});
        let idx = w.glb.indices(&indices);
        let layer = if is_water { "WATER" } else { "COL" };
        w.glb.meshes.push(json!({"name": format!("{layer}_{}", node.label), "primitives": [{"attributes": attributes, "indices": idx, "material": if is_water { wet } else { solid }}]}));
        let mesh = w.glb.meshes.len() - 1;
        let n = w.glb.node(json!({
            "name": format!("{layer}_{}", node.label),
            "mesh": mesh,
            "extras": {"gdl": {"world_node": i, "flags": node.flags, "runtime_flags": ob.flags, "dynamic": dynamic, "first_triangle": first, "triangles": count}},
        }));
        if is_water {
            counts.water_triangles += count;
            water.push(n);
        } else {
            counts.collision_triangles += count;
            col.push(n);
        }
    }
    let mut covered = vec![false; collision.triangles.len()];
    for &t in owned.values().flatten() {
        if let Some(c) = covered.get_mut(t) {
            *c = true;
        }
    }
    // NewItem: an item instance's own triangles (+4 first, +6 count), in
    // item space (ItemObjectCol). Not exported yet: placing them needs the
    // item's matrix.
    for inst in &world.items {
        let (Some(first), Some(count)) = (inst.raw.get(4..6), inst.raw.get(6..8)) else { continue };
        let (first, count) = (i16::from_le_bytes([first[0], first[1]]), u16::from_le_bytes([count[0], count[1]]));
        let Ok(first) = usize::try_from(first) else { continue };
        for t in first..first + usize::from(count) {
            if let Some(c) = covered.get_mut(t)
                && !*c
            {
                *c = true;
                counts.item_triangles += 1;
            }
        }
    }
    counts.unreferenced_triangles = covered.iter().filter(|c| !**c).count();
    for (layer, children) in [("SCENERY", scenery), ("COLLISION", col), ("WATER", water)] {
        if children.is_empty() {
            continue;
        }
        let root = w.glb.node(json!({"name": layer, "children": children}));
        w.glb.roots.push(root);
    }
    let path = out.join("levels").join(format!("{}.glb", file_stem(name)));
    let bytes = w.glb.bytes(json!({"gdl": {"level": name, "title": level.title, "space": SPACE}}))?;
    write(&path, &bytes)?;
    manifest.push(entry(
        &path,
        out,
        "level",
        &bytes,
        json!({
            "level": name,
            "title": level.title,
            "worlds_sha256": sha256(world_bytes),
            "placed": counts.placed,
            "hidden": counts.hidden,
            "collision_triangles": counts.collision_triangles,
            "water_triangles": counts.water_triangles,
            "grid_only_triangles": counts.grid_only_triangles,
            "item_triangles": counts.item_triangles,
            "unreferenced_triangles": counts.unreferenced_triangles,
        }),
    ));
    Ok(counts)
}

/// What the exporter wrote.
#[derive(Default, Debug)]
pub struct Report {
    pub files: usize,
    pub bytes: u64,
    pub levels: Vec<(String, SceneCounts)>,
    /// Parts that did not decode (they are left out).
    pub unsupported: Vec<String>,
}

const README: &str = "Models exported from your copy of Gauntlet: Dark Legacy (gdl_play --export-models).\r
\r
objects\\<package>\\<LABEL>.glb  every object of that package, one file each\r
levels\\<LEVEL>.glb              a level: SCENERY (what is drawn), COLLISION (where\r
                                  anyone can walk or bump; never drawn) and WATER\r
manifest.json                    every file with its SHA-256 and its sources\r
\r
The files are glTF 2.0 binary (File > Import > glTF 2.0 in Blender). One game\r
unit is one metre. The textures are the originals; each material's custom\r
properties give the texture's name as texture_pack\\ uses it.\r
\r
These files are made from the game disc: keep them to yourself, and do not\r
upload or share them. Exporting again replaces them.\r
";

/// Exports `packages` (their objects) and `levels` (their objects and
/// scenes) into `out`. Refuses a folder that holds anything but an earlier
/// export (no manifest.json).
pub fn export(out: &Path, packages: &[PackageSource], levels: &[LevelSource], progress: &mut dyn FnMut(&str)) -> R<Report> {
    if out.exists() && std::fs::read_dir(out)?.next().is_some() && !out.join("manifest.json").is_file() {
        return Err(format!("{} holds other files: choose an empty folder or an earlier export", out.display()).into());
    }
    std::fs::create_dir_all(out)?;
    let mut manifest = Vec::new();
    let mut sources: BTreeMap<String, String> = BTreeMap::new();
    let mut report = Report::default();
    for level in levels {
        progress(&format!("level {}", level.package.name));
        let p = load(&level.package)?;
        sources.extend(p.sources.iter().cloned());
        let world_bytes = crate::bounded_read(&level.worlds).map_err(|e| format!("{}: {e}", level.worlds.display()))?;
        sources.insert(level.worlds.display().to_string(), sha256(&world_bytes));
        export_objects(out, &level.package.name, &p, &mut manifest, &mut report.unsupported)?;
        let counts = export_scene(out, level, &p, &world_bytes, &mut manifest, &mut report.unsupported)?;
        report.levels.push((level.package.name.clone(), counts));
    }
    for package in packages {
        progress(&format!("package {}", package.name));
        let p = load(package)?;
        sources.extend(p.sources.iter().cloned());
        export_objects(out, &package.name, &p, &mut manifest, &mut report.unsupported)?;
    }
    report.files = manifest.len();
    report.bytes = manifest.iter().filter_map(|m| m["bytes"].as_u64()).sum();
    let doc = json!({"generator": VERSION, "space": SPACE, "sources": sources, "unsupported": report.unsupported, "files": manifest});
    std::fs::write(out.join("manifest.json"), serde_json::to_vec_pretty(&doc)?)?;
    std::fs::write(out.join("README.txt"), README)?;
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Synthetic: a one-triangle file has glTF's binary layout (12-byte
    /// header, 4-byte aligned chunks, lengths that add up) and its JSON
    /// names the accessors it added.
    #[test]
    fn glb_layout() {
        let mut g = Glb::default();
        let pos = g.floats(&[[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]], true);
        let idx = g.indices(&[0, 1, 2]);
        g.meshes.push(json!({"primitives": [{"attributes": {"POSITION": pos}, "indices": idx}]}));
        let n = g.node(json!({"mesh": 0}));
        g.roots.push(n);
        let b = g.bytes(json!({})).unwrap();
        assert_eq!(&b[0..4], b"glTF");
        assert_eq!(u32::from_le_bytes(b[8..12].try_into().unwrap()) as usize, b.len());
        let json_len = u32::from_le_bytes(b[12..16].try_into().unwrap()) as usize;
        assert_eq!(&b[16..20], b"JSON");
        assert!(json_len.is_multiple_of(4));
        let doc: Value = serde_json::from_slice(&b[20..20 + json_len]).unwrap();
        assert_eq!(doc["accessors"][0]["max"], json!([1.0, 1.0, 0.0]));
        assert_eq!(doc["accessors"][1]["count"], json!(3));
        assert!(doc.get("images").is_none());
        let bin_len = u32::from_le_bytes(b[20 + json_len..24 + json_len].try_into().unwrap()) as usize;
        assert_eq!(&b[24 + json_len..28 + json_len], b"BIN\0");
        assert_eq!(doc["buffers"][0]["byteLength"], json!(bin_len));
        assert_eq!(28 + json_len + bin_len, b.len());
        // 36 bytes of positions, then 12 of indices.
        assert_eq!(doc["bufferViews"][1]["byteOffset"], json!(36));
    }

    #[test]
    fn file_stems_are_safe_on_windows() {
        assert_eq!(file_stem("E2ROCKFALL#03"), "E2ROCKFALL#03");
        assert_eq!(file_stem("A/B:C*"), "A_B_C_");
        assert_eq!(file_stem("AUX"), "AUX_");
        assert_eq!(file_stem("com1"), "com1_");
        assert_eq!(file_stem("COMET"), "COMET");
        assert_eq!(file_stem("END. "), "END");
        assert_eq!(package_dir("MONSTERS/GOLEM/LEVELG"), PathBuf::from("MONSTERS").join("GOLEM").join("LEVELG"));
    }
}
