//! NATIVE (this fork, decisions/0009-texture-workflow.md): F5 reloads the
//! texture pack while the game runs. materials_for registers each base
//! texture it makes with its name (texture_pack::name of the original
//! texels), its record size, the disc's level count, its alpha class and
//! the file that replaced it, if any. `reload` asks texture_pack again (the
//! user's pack, then the generated HD pack) and, where the answer changed
//! (another file, a file written since, or none), puts the new texels into
//! the same image: a replacement's one level, or the original's levels
//! (taken from the image the first time a reload replaces it). texfilter's
//! chain and sampler follow the new level count, and gdl_play prepares the
//! materials again two frames later (as texfilter's live switch).
//!
//! HUD images (gdl_play's HudSets decodes each when first drawn) are
//! reloaded by dropping those whose file changed (`changed_files`): they
//! are decoded again with the new file.
//!
//! Not reloaded: texture-modifier targets (their texels are a frame's
//! copy; the frames themselves are reloaded), HUD z masks (always the
//! disc's) and lightmaps. A
//! texture replaced when its level loaded cannot go back to the original
//! (it was never in memory): that waits for the next level. A replacement
//! whose alpha class (opaque, cut-out, blended) differs from the one the
//! material was made with keeps the material's until the level loads again.
use crate::texfilter::{self, TexKey};
use bevy::asset::{AssetId, Assets};
use bevy::image::Image;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::SystemTime;

/// A replacement file as it was when applied.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Applied {
    path: PathBuf,
    modified: Option<SystemTime>,
    len: u64,
}

impl Applied {
    pub fn of(path: &Path) -> Option<Self> {
        let m = std::fs::metadata(path).ok()?;
        Some(Self { path: path.to_path_buf(), modified: m.modified().ok(), len: m.len() })
    }
}

/// A registered texture.
struct Entry {
    name: String,
    width: u16,
    height: u16,
    /// The image's own levels when it shows the original (the disc's).
    levels: u8,
    /// The alpha class the material was made with.
    class: u8,
    applied: Option<Applied>,
    /// The original's levels, kept when a reload first replaces it.
    original: Option<Vec<Vec<u8>>>,
}

/// What a reload did.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Report {
    /// Textures that took a new or changed file.
    pub replaced: usize,
    /// Textures back to their originals (their file was removed).
    pub restored: usize,
    /// Files that could not be read (not PNG, too large).
    pub failed: usize,
    /// Textures whose file was removed but whose original is not in memory
    /// (replaced since their level loaded): back at the next level.
    pub later: usize,
    /// Changed textures whose transparency kind differs from their
    /// material's (shown right from the next level).
    pub alpha_changed: usize,
}

impl Report {
    /// One line for the screen.
    pub fn message(&self) -> String {
        if self.replaced + self.restored + self.failed + self.later == 0 {
            return "Textures reloaded: no changes".to_string();
        }
        let mut parts = Vec::new();
        if 0 < self.replaced {
            parts.push(format!("{} replaced", self.replaced));
        }
        if 0 < self.restored {
            parts.push(format!("{} back to the original", self.restored));
        }
        if 0 < self.failed {
            parts.push(format!("{} files unreadable", self.failed));
        }
        if 0 < self.later {
            parts.push(format!("{} back at the next level", self.later));
        }
        if 0 < self.alpha_changed {
            parts.push(format!("{} with new transparency: next level", self.alpha_changed));
        }
        format!("Textures reloaded: {}", parts.join(", "))
    }
}

#[derive(Default)]
struct Registry(HashMap<AssetId<Image>, Entry>);

static REGISTRY: Mutex<Option<Registry>> = Mutex::new(None);

/// Remember a base texture for `reload` (materials_for).
pub fn register(id: AssetId<Image>, name: String, width: u16, height: u16, levels: u8, class: u8, applied: Option<&Path>) {
    if let Ok(mut r) = REGISTRY.lock() {
        r.get_or_insert_with(Registry::default).insert(id, name, width, height, levels, class, applied.and_then(Applied::of));
    }
}

/// Reload every registered texture's replacement (F5). Images that no
/// longer exist are forgotten.
pub fn reload(images: &mut Assets<Image>) -> Report {
    let Ok(mut r) = REGISTRY.lock() else { return Report::default() };
    let Some(reg) = r.as_mut() else { return Report::default() };
    let (report, keys) = reg.reload(images, &crate::texture_pack::find, &texfilter::key, texfilter::sharp());
    for (id, key) in keys {
        texfilter::register(id, key);
    }
    texfilter::bump_generation();
    report
}

/// The replacement file a texture name has now (the user's pack, then the
/// generated HD pack), as `changed_files` compares it.
pub fn applied(name: &str) -> Option<Applied> {
    crate::texture_pack::find(name).and_then(|p| Applied::of(&p))
}

/// HUD images: of `entries` (key, texture name, the file applied when it
/// was decoded), those whose file is another now (another path, one
/// written since, or none), each with whether a file replaces it now.
pub fn changed_files<'a, K: Copy + 'a>(entries: impl IntoIterator<Item = (&'a K, &'a (String, Option<Applied>))>, find: &dyn Fn(&str) -> Option<PathBuf>) -> Vec<(K, bool)> {
    entries
        .into_iter()
        .filter_map(|(k, (name, was))| {
            let now = find(name).and_then(|p| Applied::of(&p));
            (now != *was).then_some((*k, now.is_some()))
        })
        .collect()
}

/// The alpha class materials_for gives level 0 (0 opaque, 1 cut-out, 2
/// blended).
fn alpha_class(rgba: &[u8]) -> u8 {
    crate::hd::census::alpha_class(rgba)
}

impl Registry {
    #[allow(clippy::too_many_arguments)]
    fn insert(&mut self, id: AssetId<Image>, name: String, width: u16, height: u16, levels: u8, class: u8, applied: Option<Applied>) {
        self.0.insert(id, Entry { name, width, height, levels: levels.max(1), class, applied, original: None });
    }

    /// The reload with its lookups given: `find` a texture's file now,
    /// `key` an image's texfilter key. The images' new keys are returned.
    fn reload(&mut self, images: &mut Assets<Image>, find: &dyn Fn(&str) -> Option<PathBuf>, key: &dyn Fn(AssetId<Image>) -> Option<TexKey>, sharp: bool) -> (Report, Vec<(AssetId<Image>, TexKey)>) {
        let mut report = Report::default();
        let mut keys = Vec::new();
        self.0.retain(|id, e| {
            let Some(image) = images.get(*id) else { return false };
            let Some(k) = key(*id) else { return true };
            if k.borrowed {
                return true;
            }
            let want = find(&e.name).and_then(|p| Applied::of(&p));
            if want == e.applied {
                return true;
            }
            let (w, h, levels) = match &want {
                Some(a) => match crate::texture_pack::read_png(&a.path) {
                    Some((w, h, px)) if w <= u32::from(u16::MAX) && h <= u32::from(u16::MAX) => (w as u16, h as u16, vec![px]),
                    _ => {
                        report.failed += 1;
                        return true;
                    }
                },
                None => match &e.original {
                    Some(o) => (e.width, e.height, o.clone()),
                    None => {
                        report.later += 1;
                        return true;
                    }
                },
            };
            if e.applied.is_none() && e.original.is_none() {
                e.original = texfilter::own_levels(image, e.levels);
                if e.original.is_none() {
                    report.failed += 1;
                    return true;
                }
            }
            if alpha_class(&levels[0]) != e.class {
                report.alpha_changed += 1;
            }
            let new_key = TexKey { levels: levels.len() as u8, ..k };
            let chain = if k.animated { levels } else { texfilter::with_chain(w.into(), h.into(), levels, sharp) };
            if let Some(image) = images.get_mut(*id) {
                *image = crate::mip_image(w, h, chain, texfilter::sampler(new_key, sharp));
            }
            keys.push((*id, new_key));
            if want.is_some() {
                report.replaced += 1;
            } else {
                report.restored += 1;
            }
            e.applied = want;
            true
        });
        (report, keys)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn png(path: &Path, w: u32, h: u32, rgba: &[u8]) {
        let mut e = png::Encoder::new(std::io::BufWriter::new(std::fs::File::create(path).unwrap()), w, h);
        e.set_color(png::ColorType::Rgba);
        e.set_depth(png::BitDepth::Eight);
        e.write_header().unwrap().write_image_data(rgba).unwrap();
    }

    /// A 2×2 original with two levels: replaced by a 4×4 file (one level),
    /// unchanged on a second reload, replaced again when the file changes,
    /// back to the original (both levels) when the file goes; a borrowed
    /// target is left alone; a texture replaced at load cannot go back.
    #[test]
    fn reload_replaces_and_restores() {
        let dir = std::env::temp_dir().join(format!("gdl-reload-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let mut images = Assets::<Image>::default();
        let original = vec![[10u8, 20, 30, 0x80].repeat(4), vec![40u8, 50, 60, 0x80]];
        let k = TexKey { flags: 0, levels: 2, animated: false, borrowed: false };
        let id = images.add(crate::mip_image(2, 2, original.clone(), texfilter::sampler(k, false))).id();
        let target = images.add(crate::mip_image(2, 2, original.clone(), texfilter::sampler(k, false))).id();
        let early = images.add(crate::mip_image(2, 2, original.clone(), texfilter::sampler(k, false))).id();
        let mut reg = Registry::default();
        reg.insert(id, "aaaa".into(), 2, 2, 2, 0, None);
        reg.insert(target, "bbbb".into(), 2, 2, 2, 0, None);
        let early_file = dir.join("cccc.png");
        png(&early_file, 2, 2, &[1, 2, 3, 255].repeat(4));
        reg.insert(early, "cccc".into(), 2, 2, 1, 0, Applied::of(&early_file));
        let keys: HashMap<AssetId<Image>, TexKey> = [(id, k), (target, TexKey { borrowed: true, ..k }), (early, TexKey { levels: 1, ..k })].into();
        let key = |i: AssetId<Image>| keys.get(&i).copied();
        let find = |n: &str| Some(dir.join(format!("{n}.png"))).filter(|p| p.is_file());
        // A new file for aaaa and bbbb: aaaa takes it (one level), the
        // borrowed target does not.
        png(&dir.join("aaaa.png"), 4, 4, &[200, 100, 0, 255].repeat(16));
        png(&dir.join("bbbb.png"), 4, 4, &[200, 100, 0, 255].repeat(16));
        let (r, new_keys) = reg.reload(&mut images, &find, &key, false);
        assert_eq!(r, Report { replaced: 1, ..Default::default() });
        assert_eq!(new_keys, [(id, TexKey { levels: 1, ..k })]);
        let im = images.get(id).unwrap();
        assert_eq!((im.width(), im.height(), im.texture_descriptor.mip_level_count), (4, 4, 1));
        assert_eq!(&im.data.as_ref().unwrap()[..4], &[200, 100, 0, 0x80]);
        assert_eq!(images.get(target).unwrap().width(), 2);
        // Nothing changed: no work.
        assert_eq!(reg.reload(&mut images, &find, &key, false).0, Report::default());
        // A changed file (another size) is taken again.
        png(&dir.join("aaaa.png"), 8, 8, &[0, 0, 255, 255].repeat(64));
        assert_eq!(reg.reload(&mut images, &find, &key, false).0.replaced, 1);
        assert_eq!(images.get(id).unwrap().width(), 8);
        // Removed: the original's two levels come back; the texture
        // replaced at load stays until its level loads again.
        std::fs::remove_file(dir.join("aaaa.png")).unwrap();
        std::fs::remove_file(&early_file).unwrap();
        let (r, _) = reg.reload(&mut images, &find, &key, false);
        assert_eq!((r.restored, r.later), (1, 1));
        let im = images.get(id).unwrap();
        assert_eq!((im.width(), im.texture_descriptor.mip_level_count), (2, 2));
        assert_eq!(im.data.as_deref(), Some(original.concat().as_slice()));
        // A file with transparency on an opaque material is counted.
        png(&dir.join("aaaa.png"), 2, 2, &[9, 9, 9, 0].repeat(4));
        assert_eq!(reg.reload(&mut images, &find, &key, false).0.alpha_changed, 1);
        // An image that no longer exists is forgotten.
        images.remove(id);
        reg.reload(&mut images, &find, &key, false);
        assert!(!reg.0.contains_key(&id));
        std::fs::remove_dir_all(&dir).unwrap();
    }

    /// HUD images: a new file, a rewritten file, a removed file and an
    /// unchanged one.
    #[test]
    fn hud_images_whose_file_changed() {
        let dir = std::env::temp_dir().join(format!("gdl-reload-hud-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let find = |n: &str| Some(dir.join(format!("{n}.png"))).filter(|p| p.is_file());
        for n in ["same", "rewritten", "removed"] {
            png(&dir.join(format!("{n}.png")), 2, 2, &[1, 2, 3, 255].repeat(4));
        }
        let entries: HashMap<u32, (String, Option<Applied>)> = [(1, "new"), (2, "same"), (3, "rewritten"), (4, "removed"), (5, "none")].into_iter().map(|(k, n)| (k, (n.to_string(), find(n).and_then(|p| Applied::of(&p))))).collect();
        assert!(changed_files(&entries, &find).is_empty());
        png(&dir.join("new.png"), 2, 2, &[1, 2, 3, 255].repeat(4));
        png(&dir.join("rewritten.png"), 4, 4, &[9, 9, 9, 255].repeat(16));
        std::fs::remove_file(dir.join("removed.png")).unwrap();
        let mut got = changed_files(&entries, &find);
        got.sort();
        assert_eq!(got, [(1, true), (3, true), (4, false)]);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn messages() {
        assert_eq!(Report::default().message(), "Textures reloaded: no changes");
        let r = Report { replaced: 3, restored: 1, ..Default::default() };
        assert_eq!(r.message(), "Textures reloaded: 3 replaced, 1 back to the original");
    }
}
