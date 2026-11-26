//! NATIVE (this fork, decisions/0027): items take the light of the ground
//! they stand on. The game lights every item with the level's sun and
//! ambient only, wherever it stands (present_scene_light; no item carries
//! vertex colours or a lightmap, models-plan "Lighting new models"), so a
//! barrel in a dark corner is as bright as one in the open. Here the
//! scenery's baked light (vertex colour × lightmap, as gs.wgsl draws it) is
//! read under each item and compared with the level's light in the open:
//! an item in the open keeps the original's look, one in shade is dimmed
//! (the shader scales the sun and ambient on it, not the point lights:
//! torches and spells still light it fully). Presentation only.
//!
//! The constants are NATIVE values chosen by eye (decisions/0027).

use bevy::math::{Vec2, Vec3};
use std::collections::HashMap;
use std::sync::Arc;

/// A lightmap as the renderer holds it: one byte per texel, the GS CLUT
/// alpha (index × 8; 0x80 = 1.0), sampled bilinearly and repeating.
#[derive(Clone)]
pub struct Lightmap {
    pub width: u32,
    pub height: u32,
    pub texels: Arc<Vec<u8>>,
}

impl Lightmap {
    /// The lightmap's factor at `uv` (texel / 128), bilinear, repeating.
    pub fn sample(&self, uv: Vec2) -> f32 {
        let (w, h) = (self.width as i64, self.height as i64);
        if w == 0 || h == 0 || self.texels.len() < (w * h) as usize {
            return 1.0;
        }
        let x = uv.x * w as f32 - 0.5;
        let y = uv.y * h as f32 - 0.5;
        let (x0, y0) = (x.floor(), y.floor());
        let (fx, fy) = (x - x0, y - y0);
        let at = |i: i64, j: i64| f32::from(self.texels[(j.rem_euclid(h) * w + i.rem_euclid(w)) as usize]) / 128.0;
        let (i, j) = (x0 as i64, y0 as i64);
        let top = at(i, j) * (1.0 - fx) + at(i + 1, j) * fx;
        let bottom = at(i, j + 1) * (1.0 - fx) + at(i + 1, j + 1) * fx;
        top * (1.0 - fy) + bottom * fy
    }
}

/// A scenery triangle with its baked light: render-space corners, their
/// vertex colours (GS 0x80 = 1.0) and, when lightmapped, their lightmap UVs
/// and the lightmap's index.
#[derive(Clone)]
pub struct GroundTri {
    pub corners: [Vec3; 3],
    pub colours: [Vec3; 3],
    pub lightmap: Option<([Vec2; 3], usize)>,
}

/// Only surfaces at least this level count as ground (|normal.y|).
const MIN_UP: f32 = 0.35;
/// The grid's cell (game units) for the downward rays.
const CELL: f32 = 4.0;
/// A ray starts this far above the item's origin and reaches this far
/// below it (floating pickups; an origin a little in the floor).
const REACH_UP: f32 = 0.75;
const REACH_DOWN: f32 = 3.0;
/// The five rays: the item's place and four around it, this far out (a
/// barrel's radius is about 1.2), so a lightmap texel or a seam does not
/// decide alone.
const SPREAD: f32 = 0.8;
/// The level's light in the open: this share of its ground area is darker.
const OPEN_SHARE: f32 = 0.75;
/// How much of the ground's shade an item takes, and the least light it
/// keeps (readability first: an item in deep shade still reads).
pub const STRENGTH: f32 = 0.8;
pub const FLOOR: f32 = 0.45;

/// The light a point of the ground has, as one number (Rec. 601 luma of
/// the GS-scale colour).
pub fn luma(c: Vec3) -> f32 {
    0.299 * c.x + 0.587 * c.y + 0.114 * c.z
}

/// The factor on an item's sun and ambient for ground light `light` in a
/// level whose open ground has `open`: 1 in the open or brighter, down to
/// FLOOR in deep shade.
pub fn factor(light: f32, open: f32) -> f32 {
    if open.is_nan() || open <= 0.0 {
        return 1.0;
    }
    (1.0 - STRENGTH * (1.0 - light / open)).clamp(FLOOR, 1.0)
}

/// A level's ground: its triangles, its lightmaps and a grid over x/z.
pub struct Ground {
    tris: Vec<GroundTri>,
    lightmaps: Vec<Lightmap>,
    cells: HashMap<(i32, i32), Vec<u32>>,
    /// The light of the level's open ground (OPEN_SHARE of the ground area
    /// is darker); 0 when the level has no ground.
    pub open: f32,
}

fn cell(v: f32) -> i32 {
    (v / CELL).floor() as i32
}

impl Ground {
    /// The ground of these triangles (steep ones are dropped).
    pub fn new(tris: Vec<GroundTri>, lightmaps: Vec<Lightmap>) -> Self {
        let tris: Vec<GroundTri> = tris
            .into_iter()
            .filter(|t| {
                let n = (t.corners[1] - t.corners[0]).cross(t.corners[2] - t.corners[0]);
                let len = n.length();
                len > 1e-6 && (n.y / len).abs() >= MIN_UP
            })
            .collect();
        let mut cells: HashMap<(i32, i32), Vec<u32>> = HashMap::new();
        for (k, t) in tris.iter().enumerate() {
            let (lo, hi) = (t.corners[0].min(t.corners[1]).min(t.corners[2]), t.corners[0].max(t.corners[1]).max(t.corners[2]));
            for i in cell(lo.x)..=cell(hi.x) {
                for j in cell(lo.z)..=cell(hi.z) {
                    cells.entry((i, j)).or_default().push(k as u32);
                }
            }
        }
        let mut g = Self { tris, lightmaps, cells, open: 0.0 };
        // The open ground's light: the area-weighted OPEN_SHARE quantile of
        // the triangles' light at their centres.
        let mut lit: Vec<(f32, f32)> = g
            .tris
            .iter()
            .map(|t| {
                let c = t.corners;
                let area = 0.5 * ((c[1].x - c[0].x) * (c[2].z - c[0].z) - (c[2].x - c[0].x) * (c[1].z - c[0].z)).abs();
                (luma(g.light(t, Vec3::splat(1.0 / 3.0))), area)
            })
            .filter(|(_, a)| *a > 0.0)
            .collect();
        lit.sort_by(|a, b| a.0.total_cmp(&b.0));
        let total: f32 = lit.iter().map(|(_, a)| a).sum();
        let mut acc = 0.0;
        for (l, a) in lit {
            acc += a;
            if acc >= total * OPEN_SHARE {
                g.open = l;
                break;
            }
        }
        g
    }

    pub fn triangles(&self) -> usize {
        self.tris.len()
    }

    /// The baked light of triangle `t` at barycentric weights `w`.
    fn light(&self, t: &GroundTri, w: Vec3) -> Vec3 {
        let colour = t.colours[0] * w.x + t.colours[1] * w.y + t.colours[2] * w.z;
        let lm = t.lightmap.as_ref().and_then(|(uv, i)| self.lightmaps.get(*i).map(|l| l.sample(uv[0] * w.x + uv[1] * w.y + uv[2] * w.z))).unwrap_or(1.0);
        colour * lm
    }

    /// The ground's light at (x, z): the highest ground between `top` and
    /// `bottom` straight below.
    pub fn light_below(&self, x: f32, z: f32, top: f32, bottom: f32) -> Option<Vec3> {
        let list = self.cells.get(&(cell(x), cell(z)))?;
        let mut best: Option<(f32, Vec3)> = None;
        for &k in list {
            let t = &self.tris[k as usize];
            let [a, b, c] = t.corners;
            let d = (b.x - a.x) * (c.z - a.z) - (c.x - a.x) * (b.z - a.z);
            if d.abs() < 1e-9 {
                continue;
            }
            let wb = ((x - a.x) * (c.z - a.z) - (c.x - a.x) * (z - a.z)) / d;
            let wc = ((b.x - a.x) * (z - a.z) - (x - a.x) * (b.z - a.z)) / d;
            let wa = 1.0 - wb - wc;
            const EDGE: f32 = -1e-4;
            if wa < EDGE || wb < EDGE || wc < EDGE {
                continue;
            }
            let y = a.y * wa + b.y * wb + c.y * wc;
            if y > top || y < bottom || best.is_some_and(|(h, _)| h >= y) {
                continue;
            }
            best = Some((y, self.light(t, Vec3::new(wa, wb, wc))));
        }
        best.map(|(_, l)| l)
    }

    /// The ground light under an item at render-space `at` (its origin):
    /// the mean light of the rays that meet ground, as one number.
    pub fn under(&self, at: Vec3) -> Option<f32> {
        let mut sum = 0.0;
        let mut n = 0;
        for (dx, dz) in [(0.0, 0.0), (SPREAD, 0.0), (-SPREAD, 0.0), (0.0, SPREAD), (0.0, -SPREAD)] {
            if let Some(l) = self.light_below(at.x + dx, at.z + dz, at.y + REACH_UP, at.y - REACH_DOWN) {
                sum += luma(l);
                n += 1;
            }
        }
        (0 < n).then(|| sum / n as f32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn quad(x0: f32, z0: f32, size: f32, y: f32, colour: f32) -> Vec<GroundTri> {
        let c = Vec3::splat(colour);
        let p = |x: f32, z: f32| Vec3::new(x, y, z);
        vec![
            GroundTri { corners: [p(x0, z0), p(x0 + size, z0), p(x0 + size, z0 + size)], colours: [c; 3], lightmap: None },
            GroundTri { corners: [p(x0, z0), p(x0 + size, z0 + size), p(x0, z0 + size)], colours: [c; 3], lightmap: None },
        ]
    }

    #[test]
    fn open_ground_shade_and_floors() {
        // Three quarters of the ground lit 1.0, a quarter in shade 0.25,
        // and a bridge at y 5 lit 0.5 over part of the lit ground.
        let mut tris = quad(0.0, 0.0, 30.0, 0.0, 1.0);
        tris.extend(quad(-10.0, 0.0, 10.0, 0.0, 0.25));
        tris.extend(quad(20.0, 20.0, 4.0, 5.0, 0.5));
        // A wall: steep, never ground.
        tris.push(GroundTri { corners: [Vec3::new(5.0, 0.0, 5.0), Vec3::new(5.0, 10.0, 5.0), Vec3::new(5.0, 0.0, 9.0)], colours: [Vec3::ZERO; 3], lightmap: None });
        let g = Ground::new(tris, Vec::new());
        assert_eq!(g.triangles(), 6);
        assert!((g.open - 1.0).abs() < 1e-6, "open {}", g.open);
        // In the open: unchanged.
        let open = g.under(Vec3::new(10.0, 0.0, 10.0)).unwrap();
        assert!((open - 1.0).abs() < 1e-6);
        assert_eq!(factor(open, g.open), 1.0);
        // In shade: dimmed, not below the floor.
        let shade = g.under(Vec3::new(-5.0, 0.0, 5.0)).unwrap();
        assert!((shade - 0.25).abs() < 1e-6);
        assert_eq!(factor(shade, g.open), (1.0 - STRENGTH * 0.75).max(FLOOR));
        assert!((factor(0.5, g.open) - (1.0 - STRENGTH * 0.5)).abs() < 1e-6);
        assert_eq!(factor(0.0, g.open), FLOOR);
        // On the bridge: the bridge's light, not the ground's below it; under
        // the bridge: the ground's.
        assert!((g.under(Vec3::new(22.0, 5.0, 22.0)).unwrap() - 0.5).abs() < 1e-6);
        assert!((g.under(Vec3::new(22.0, 0.0, 22.0)).unwrap() - 1.0).abs() < 1e-6);
        // Off the ground: nothing to say.
        assert!(g.under(Vec3::new(100.0, 0.0, 100.0)).is_none());
        // Brighter than the open: unchanged.
        assert_eq!(factor(2.0, 1.0), 1.0);
        assert_eq!(factor(0.5, 0.0), 1.0);
    }

    #[test]
    fn lightmaps_scale_the_colour() {
        // A 2 × 1 lightmap: 0x80 (1.0) then 0x40 (0.5), repeating.
        let lm = Lightmap { width: 2, height: 1, texels: Arc::new(vec![0x80, 0x40]) };
        assert!((lm.sample(Vec2::new(0.25, 0.5)) - 1.0).abs() < 1e-6);
        assert!((lm.sample(Vec2::new(0.75, 0.5)) - 0.5).abs() < 1e-6);
        assert!((lm.sample(Vec2::new(0.5, 0.5)) - 0.75).abs() < 1e-6);
        assert!((lm.sample(Vec2::new(1.25, 0.5)) - 1.0).abs() < 1e-6);
        let c = Vec3::new(0.8, 0.6, 0.4);
        let p = |x: f32, z: f32| Vec3::new(x, 0.0, z);
        let uv = [Vec2::new(0.75, 0.5); 3];
        let tris = vec![GroundTri { corners: [p(0.0, 0.0), p(8.0, 0.0), p(0.0, 8.0)], colours: [c; 3], lightmap: Some((uv, 0)) }];
        let g = Ground::new(tris, vec![lm]);
        let l = g.light_below(1.0, 1.0, 1.0, -1.0).unwrap();
        assert!((l - c * 0.5).length() < 1e-6);
    }
}
