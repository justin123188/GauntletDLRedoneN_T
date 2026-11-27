//! NATIVE (this fork, decisions/0035; models-plan "Lighting new models",
//! option A): a replaced piece of scenery takes the original's baked
//! light. The level's scenery carries its light baked: vertex colours (GS
//! 0x80 = 1.0) clamped at 0x80 by VU1 program B, times the lightmap's CLUT
//! alpha / 0x80 in the lightmap pass (gs.wgsl). A model pack file has
//! neither, so each of its vertices takes the light of the nearest point
//! of the original object's surface: min(colour, 1) there as its vertex
//! colour and, where the original is lightmapped, the lightmap there as a
//! coordinate into a ramp (`ramp_image`: texel i holds i, so the sample is
//! the lightmap's value, interpolated between vertices), and the part
//! draws through program B and the original's lightmap pass
//! (decisions/0037): its 16-bit frame write before the lightmap (4.5 / 255
//! at full colour), the point lights under the lightmap, ambient occlusion.
//! Presentation only.

use crate::ground_light::{GroundTri, Lightmap};
use crate::GsMaterial;
use bevy::asset::{Assets, Handle, RenderAssetUsages};
use bevy::image::{Image, ImageSampler, ImageSamplerDescriptor};
use bevy::math::{Vec2, Vec3};
use bevy::mesh::{Indices, Mesh, VertexAttributeValues};
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use std::collections::HashMap;

/// The search grid's cell (game units) and how many rings of cells are
/// searched before every triangle is tried.
const CELL: f32 = 2.0;
const MAX_RING: i32 = 6;

/// The lightmap ramp's width: texel i holds i (i / 0x80 as a lightmap).
pub const RAMP_WIDTH: u32 = 256;

/// The lightmap ramp's texels.
pub fn ramp_texels() -> Vec<u8> {
    (0..RAMP_WIDTH).map(|i| i as u8).collect()
}

/// The lightmap ramp, as the disc's lightmaps are made (R8, linear): a
/// vertex's lightmap value as a coordinate (`ramp_uv`) samples back as
/// that value, interpolated linearly between vertices (and by the smooth
/// lightmaps' B-spline, which keeps a straight ramp).
pub fn ramp_image() -> Image {
    let mut image = Image::new(
        Extent3d { width: RAMP_WIDTH, height: 1, depth_or_array_layers: 1 },
        TextureDimension::D2,
        ramp_texels(),
        TextureFormat::R8Unorm,
        RenderAssetUsages::default(),
    );
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor::linear());
    image
}

/// The ramp coordinate of a lightmap value (0x80 = 1.0).
pub fn ramp_uv(lightmap: f32) -> [f32; 2] {
    [(lightmap.clamp(0.0, 1.0) * 128.0 + 0.5) / RAMP_WIDTH as f32, 0.5]
}

/// An original object's surface with its baked light.
pub struct Source {
    tris: Vec<GroundTri>,
    lightmaps: Vec<Lightmap>,
    cells: HashMap<(i32, i32, i32), Vec<u32>>,
}

fn cell_of(v: Vec3) -> (i32, i32, i32) {
    let c = (v / CELL).floor();
    (c.x as i32, c.y as i32, c.z as i32)
}

/// The barycentric weights of the point of triangle a, b, c nearest to `p`
/// (Ericson, Real-Time Collision Detection 5.1.5).
pub fn closest_weights(p: Vec3, a: Vec3, b: Vec3, c: Vec3) -> Vec3 {
    let (ab, ac, ap) = (b - a, c - a, p - a);
    let (d1, d2) = (ab.dot(ap), ac.dot(ap));
    if d1 <= 0.0 && d2 <= 0.0 {
        return Vec3::X;
    }
    let bp = p - b;
    let (d3, d4) = (ab.dot(bp), ac.dot(bp));
    if d3 >= 0.0 && d4 <= d3 {
        return Vec3::Y;
    }
    let vc = d1 * d4 - d3 * d2;
    if vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0 {
        let v = d1 / (d1 - d3);
        return Vec3::new(1.0 - v, v, 0.0);
    }
    let cp = p - c;
    let (d5, d6) = (ab.dot(cp), ac.dot(cp));
    if d6 >= 0.0 && d5 <= d6 {
        return Vec3::Z;
    }
    let vb = d5 * d2 - d1 * d6;
    if vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0 {
        let w = d2 / (d2 - d6);
        return Vec3::new(1.0 - w, 0.0, w);
    }
    let va = d3 * d6 - d5 * d4;
    if va <= 0.0 && (d4 - d3) >= 0.0 && (d5 - d6) >= 0.0 {
        let w = (d4 - d3) / ((d4 - d3) + (d5 - d6));
        return Vec3::new(0.0, 1.0 - w, w);
    }
    let denom = 1.0 / (va + vb + vc);
    let (v, w) = (vb * denom, vc * denom);
    Vec3::new(1.0 - v - w, v, w)
}

impl Source {
    /// The surface of these triangles (degenerate ones dropped); None
    /// without any.
    pub fn new(tris: Vec<GroundTri>, lightmaps: Vec<Lightmap>) -> Option<Self> {
        let tris: Vec<GroundTri> = tris.into_iter().filter(|t| (t.corners[1] - t.corners[0]).cross(t.corners[2] - t.corners[0]).length() > 1e-6).collect();
        if tris.is_empty() {
            return None;
        }
        let mut cells: HashMap<(i32, i32, i32), Vec<u32>> = HashMap::new();
        for (k, t) in tris.iter().enumerate() {
            let (lo, hi) = (cell_of(t.corners[0].min(t.corners[1]).min(t.corners[2])), cell_of(t.corners[0].max(t.corners[1]).max(t.corners[2])));
            for i in lo.0..=hi.0 {
                for j in lo.1..=hi.1 {
                    for l in lo.2..=hi.2 {
                        cells.entry((i, j, l)).or_default().push(k as u32);
                    }
                }
            }
        }
        Some(Self { tris, lightmaps, cells })
    }

    /// The nearest triangle to `p` and the weights of its nearest point.
    fn nearest(&self, p: Vec3) -> (usize, Vec3) {
        let mut best = (f32::INFINITY, 0, Vec3::X);
        let try_tri = |k: usize, best: &mut (f32, usize, Vec3)| {
            let t = &self.tris[k];
            let w = closest_weights(p, t.corners[0], t.corners[1], t.corners[2]);
            let d = (t.corners[0] * w.x + t.corners[1] * w.y + t.corners[2] * w.z).distance_squared(p);
            if d < best.0 {
                *best = (d, k, w);
            }
        };
        let c = cell_of(p);
        for r in 0..=MAX_RING {
            for i in -r..=r {
                for j in -r..=r {
                    for l in -r..=r {
                        if i.abs().max(j.abs()).max(l.abs()) != r {
                            continue;
                        }
                        if let Some(list) = self.cells.get(&(c.0 + i, c.1 + j, c.2 + l)) {
                            for &k in list {
                                try_tri(k as usize, &mut best);
                            }
                        }
                    }
                }
            }
            // Every cell of the next ring is at least r cells away.
            let reach = r as f32 * CELL;
            if best.0 <= reach * reach {
                return (best.1, best.2);
            }
        }
        for k in 0..self.tris.len() {
            try_tri(k, &mut best);
        }
        (best.1, best.2)
    }

    /// The baked light at the surface point nearest to `p`: the vertex
    /// colour there clamped at 1 (0x80) and, where that triangle is
    /// lightmapped, the lightmap there.
    pub fn baked_at(&self, p: Vec3) -> (Vec3, Option<f32>) {
        let (k, w) = self.nearest(p);
        let t = &self.tris[k];
        let colour = (t.colours[0] * w.x + t.colours[1] * w.y + t.colours[2] * w.z).min(Vec3::ONE);
        let lm = t.lightmap.as_ref().and_then(|(uv, i)| self.lightmaps.get(*i).map(|l| l.sample(uv[0] * w.x + uv[1] * w.y + uv[2] * w.z)));
        (colour, lm)
    }

    /// The baked light at the surface point nearest to `p` as one colour:
    /// the vertex colour there clamped at 1, times the lightmap there.
    pub fn light_at(&self, p: Vec3) -> Vec3 {
        let (colour, lm) = self.baked_at(p);
        colour * lm.unwrap_or(1.0)
    }
}

/// A part list's surface with its baked light, in the parts' own frame:
/// each mesh's triangles with its vertex colours (1.0 without) and, when
/// its material draws a lightmap, its lightmap UVs and texels. None when
/// no part has a mesh with triangles.
pub fn source_of(parts: &[(Handle<Mesh>, Handle<GsMaterial>)], meshes: &Assets<Mesh>, materials: &Assets<GsMaterial>, images: &Assets<Image>) -> Option<Source> {
    let mut tris = Vec::new();
    let mut lightmaps = Vec::new();
    let mut lightmap_of: HashMap<bevy::asset::AssetId<Image>, Option<usize>> = HashMap::new();
    for (mesh, material) in parts {
        let (Some(mesh), Some(m)) = (meshes.get(mesh), materials.get(material)) else { continue };
        let Some(VertexAttributeValues::Float32x3(pos)) = mesh.attribute(Mesh::ATTRIBUTE_POSITION) else { continue };
        let colours = match mesh.attribute(Mesh::ATTRIBUTE_COLOR) {
            Some(VertexAttributeValues::Float32x4(c)) if c.len() == pos.len() => Some(c),
            _ => None,
        };
        let uvs = match mesh.attribute(Mesh::ATTRIBUTE_UV_1) {
            Some(VertexAttributeValues::Float32x2(u)) if u.len() == pos.len() => Some(u),
            _ => None,
        };
        let lightmap = match (&m.lightmap, uvs) {
            (Some(h), Some(uv)) if 0.5 < m.params.options.y => (*lightmap_of.entry(h.id()).or_insert_with(|| {
                let img = images.get(h)?;
                let (w, hh) = (img.width(), img.height());
                let data = img.data.as_ref().filter(|d| d.len() == (w * hh) as usize)?;
                lightmaps.push(Lightmap { width: w, height: hh, texels: std::sync::Arc::new(data.clone()) });
                Some(lightmaps.len() - 1)
            }))
            .map(|i| (uv, i)),
            _ => None,
        };
        let idx: Vec<u32> = match mesh.indices() {
            Some(Indices::U32(i)) => i.clone(),
            Some(Indices::U16(i)) => i.iter().map(|&k| u32::from(k)).collect(),
            None => (0..pos.len() as u32).collect(),
        };
        let colour = |k: u32| colours.map_or(Vec3::ONE, |c| Vec3::new(c[k as usize][0], c[k as usize][1], c[k as usize][2]));
        for t in idx.chunks_exact(3) {
            if t.iter().any(|&k| k as usize >= pos.len()) {
                continue;
            }
            tris.push(GroundTri {
                corners: [t[0], t[1], t[2]].map(|k| Vec3::from_array(pos[k as usize])),
                colours: [colour(t[0]), colour(t[1]), colour(t[2])],
                lightmap: lightmap.map(|(uv, i)| ([t[0], t[1], t[2]].map(|k| Vec2::from_array(uv[k as usize])), i)),
            });
        }
    }
    Source::new(tris, lightmaps)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn the_nearest_point_of_a_triangle() {
        let (a, b, c) = (Vec3::ZERO, Vec3::new(2.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 2.0));
        let at = |w: Vec3| a * w.x + b * w.y + c * w.z;
        // Above the inside: straight down.
        assert!((at(closest_weights(Vec3::new(0.5, 3.0, 0.5), a, b, c)) - Vec3::new(0.5, 0.0, 0.5)).length() < 1e-6);
        // Beyond a corner, beyond an edge.
        assert_eq!(closest_weights(Vec3::new(-1.0, 0.0, -1.0), a, b, c), Vec3::X);
        assert!((at(closest_weights(Vec3::new(1.0, 0.0, -1.0), a, b, c)) - Vec3::new(1.0, 0.0, 0.0)).length() < 1e-6);
        assert!((at(closest_weights(Vec3::new(2.0, 0.0, 2.0), a, b, c)) - Vec3::new(1.0, 0.0, 1.0)).length() < 1e-6);
    }

    #[test]
    fn a_new_vertex_takes_the_nearest_baked_light() {
        // Two floor triangles: one with colours 0.5 / 1.0 / 1.5 across it,
        // one lightmapped at 0x40 (0.5) far off.
        let tri = |o: Vec3, colours: [Vec3; 3], lightmap| GroundTri { corners: [o, o + Vec3::new(4.0, 0.0, 0.0), o + Vec3::new(0.0, 0.0, 4.0)], colours, lightmap };
        let lm = Lightmap { width: 1, height: 1, texels: Arc::new(vec![0x40]) };
        let s = Source::new(
            vec![
                tri(Vec3::ZERO, [Vec3::splat(0.5), Vec3::splat(1.0), Vec3::splat(1.5)], None),
                tri(Vec3::new(100.0, 0.0, 0.0), [Vec3::ONE; 3], Some(([Vec2::splat(0.5); 3], 0))),
            ],
            vec![lm],
        )
        .unwrap();
        // At the first corner, above it: its colour.
        assert!((s.light_at(Vec3::new(0.0, 0.3, 0.0)) - Vec3::splat(0.5)).length() < 1e-5);
        // Half way to the bright corner: interpolated (0.5 → 1.0).
        assert!((s.light_at(Vec3::new(2.0, -0.2, 0.0)) - Vec3::splat(0.75)).length() < 1e-5);
        // At the 1.5 corner: clamped at 1, as program B does.
        assert!((s.light_at(Vec3::new(0.0, 0.0, 4.0)) - Vec3::ONE).length() < 1e-5);
        // Beside the far triangle (outside the searched rings of the first):
        // its lightmap, 0x40 / 0x80.
        assert!((s.light_at(Vec3::new(101.0, 0.5, 1.0)) - Vec3::splat(0.5)).length() < 1e-5);
        // Degenerate triangles only: nothing to take light from.
        assert!(Source::new(vec![GroundTri { corners: [Vec3::ZERO; 3], colours: [Vec3::ONE; 3], lightmap: None }], Vec::new()).is_none());
        // Apart: the colour, and the lightmap where there is one.
        assert_eq!(s.baked_at(Vec3::new(0.0, 0.3, 0.0)).1, None);
        let (c, lm) = s.baked_at(Vec3::new(101.0, 0.5, 1.0));
        assert!((c - Vec3::ONE).length() < 1e-5 && (lm.unwrap() - 0.5).abs() < 1e-5);
    }

    #[test]
    fn the_ramp_samples_back_as_the_lightmap() {
        let ramp = Lightmap { width: RAMP_WIDTH, height: 1, texels: Arc::new(ramp_texels()) };
        for k in 0..=128 {
            let v = k as f32 / 128.0;
            let [u, w] = ramp_uv(v);
            assert!((ramp.sample(Vec2::new(u, w)) - v).abs() < 1e-5, "{v}");
        }
        // Between two vertices: linear, as the vertex colours are.
        let (a, b) = (ramp_uv(0.25), ramp_uv(0.75));
        assert!((ramp.sample(Vec2::new((a[0] + b[0]) / 2.0, 0.5)) - 0.5).abs() < 1e-5);
        assert_eq!(ramp_image().texture_descriptor.format, TextureFormat::R8Unorm);
    }
}
