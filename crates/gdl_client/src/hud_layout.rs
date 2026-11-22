//! NATIVE (this fork, decisions/0011): the HUD layout options. The four
//! player panels of the original 512×384 blit screen (boxes 128 units wide
//! at box_lx, along the bottom rows) can be drawn smaller
//! (Settings::hud_size, scaled toward the screen's bottom), spread over the
//! HUD's whole width in widescreen (hud_edges: player 1 at the left edge,
//! player 4 at the right) and left out for players not in the game
//! (hud_hide_empty). gdl_core tags what belongs to a box (hud::PanelPart);
//! gdl_play's present_hud moves it here. When the layout differs from the
//! original the 3D view continues under the panels (ps2_viewport) and each
//! panel drawn gets the black backing the original had below the 3D view,
//! so a panel looks as it did. Presentation only: the simulation never
//! reads it.
use gdl_core::game::hud::PanelPart;

/// A box's width in blit units (box_lx steps of 0x80).
pub const BOX_W: f32 = 128.0;
/// The blit screen's height.
pub const SCREEN_H: f32 = 384.0;
/// The panels' top (reset_players_sub's top bar, y 0x130).
pub const STRIP_TOP: f32 = 304.0;
/// The HUD sizes the overlay offers, percent.
pub const SIZES: [u32; 6] = [50, 60, 70, 80, 90, 100];

/// Where the four boxes go this frame.
#[derive(Clone, Debug, PartialEq)]
pub struct PanelLayout {
    /// The panels' scale (1: the original size).
    pub scale: f32,
    /// Each box's original left edge (box_lx).
    pub from: [f32; 4],
    /// Each box's left edge as drawn.
    pub to: [f32; 4],
    /// The boxes drawn.
    pub shown: [bool; 4],
    /// Where the original 3D view ended (blit y): below it the original
    /// frame was black under the panels.
    pub view_bottom: f32,
}

impl PanelLayout {
    /// `size` percent (50..100), `edges` (spread over `span`, the HUD
    /// camera's width in blit units centred on x 256), `hide_empty` with
    /// `in_use` (the boxes with a player).
    pub fn new(size: u32, edges: bool, hide_empty: bool, span: f32, box_lx: [f32; 4], in_use: [bool; 4], view_bottom: f32) -> Self {
        let scale = size.clamp(50, 100) as f32 / 100.0;
        let w = BOX_W * scale;
        let left = 256.0 - span.max(512.0) * 0.5;
        let to = std::array::from_fn(|p| if edges { left + p as f32 * (span.max(512.0) - w) / 3.0 } else { 256.0 + (box_lx[p] - 256.0) * scale });
        Self { scale, from: box_lx, to, shown: if hide_empty { in_use } else { [true; 4] }, view_bottom }
    }

    /// Whether anything differs from the original layout (the 3D view then
    /// continues under the panels).
    pub fn changed(&self) -> bool {
        self.scale != 1.0 || self.shown.contains(&false) || self.from.iter().zip(&self.to).any(|(a, b)| 1e-3 < (a - b).abs())
    }

    /// Where a blit (x, y, w, h) of `part` is drawn; None: not drawn (its
    /// box is hidden, or it lies above or below the screen, where scaling
    /// would bring it into view).
    pub fn place(&self, part: PanelPart, [x, y, w, h]: [f32; 4]) -> Option<[f32; 4]> {
        match part {
            PanelPart::Panel(p) => {
                let p = usize::from(p & 3);
                if !self.shown[p] || y + h <= 0.0 || SCREEN_H <= y {
                    return None;
                }
                let s = self.scale;
                Some([self.to[p] + (x - self.from[p]) * s, SCREEN_H - (SCREEN_H - y) * s, w * s, h * s])
            }
            // A help box keeps its size: it follows its box's centre and the
            // panels' top.
            PanelPart::Over(p) => {
                let p = usize::from(p & 3);
                let dx = self.to[p] + BOX_W * self.scale * 0.5 - (self.from[p] + BOX_W * 0.5);
                Some([x + dx, y + (SCREEN_H - STRIP_TOP) * (1.0 - self.scale), w, h])
            }
        }
    }

    /// Box `p`'s backing as drawn: the rows the original 3D view left black
    /// (view_bottom to the screen's bottom) under its panel.
    pub fn backing(&self, p: usize) -> Option<[f32; 4]> {
        self.place(PanelPart::Panel(p as u8), [self.from[p], self.view_bottom, BOX_W, SCREEN_H - self.view_bottom])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LX: [f32; 4] = [0.0, 128.0, 256.0, 384.0];
    const VIEW: f32 = 384.0 * 384.0 / 448.0;

    #[test]
    fn the_original_layout_is_unchanged() {
        for edges in [false, true] {
            let l = PanelLayout::new(100, edges, true, 512.0, LX, [true; 4], VIEW);
            assert!(!l.changed());
            assert_eq!(l.place(PanelPart::Panel(2), [270.0, 320.0, 16.0, 8.0]), Some([270.0, 320.0, 16.0, 8.0]));
            assert_eq!(l.place(PanelPart::Over(2), [250.0, 240.0, 80.0, 24.0]), Some([250.0, 240.0, 80.0, 24.0]));
        }
        // Edges without widescreen, or a narrower span: the frame's edges.
        assert!(!PanelLayout::new(100, true, false, 400.0, LX, [false; 4], VIEW).changed());
    }

    #[test]
    fn smaller_panels_shrink_toward_the_bottom() {
        let l = PanelLayout::new(50, false, false, 512.0, LX, [true; 4], VIEW);
        assert!(l.changed());
        // Together, centred: box 0 from x 128, each 64 wide.
        assert_eq!(l.to, [128.0, 192.0, 256.0, 320.0]);
        assert_eq!(l.place(PanelPart::Panel(0), [0.0, 304.0, 128.0, 16.0]), Some([128.0, 344.0, 64.0, 8.0]));
        // The bottom row stays at the bottom; a help box follows the top.
        assert_eq!(l.place(PanelPart::Panel(3), [384.0, 376.0, 8.0, 8.0]), Some([320.0, 380.0, 4.0, 4.0]));
        assert_eq!(l.place(PanelPart::Over(0), [0.0, 240.0, 100.0, 24.0]), Some([128.0 + 32.0 - 64.0, 280.0, 100.0, 24.0]));
        // Off the screen stays off it (the level-up blit at y −0x143).
        assert_eq!(l.place(PanelPart::Panel(1), [128.0, -323.0, 28.0, 16.0]), None);
        assert_eq!(l.place(PanelPart::Panel(1), [128.0, 399.0, 128.0, 16.0]), None);
        let b = l.backing(2).unwrap();
        assert_eq!((b[0], b[2]), (256.0, 64.0));
        assert!((b[1] - (384.0 - (384.0 - VIEW) * 0.5)).abs() < 1e-4 && (b[1] + b[3] - 384.0).abs() < 1e-4);
    }

    #[test]
    fn edges_spread_over_the_widescreen_span() {
        // 1920×1080: the HUD camera is 512 × 1920/1542.86 wide.
        let span = 512.0 * 1920.0 / (1080.0 * 640.0 / 448.0);
        let l = PanelLayout::new(100, true, false, span, LX, [true; 4], VIEW);
        assert!(l.changed());
        let left = 256.0 - span * 0.5;
        assert!((l.to[0] - left).abs() < 1e-3 && (l.to[3] + 128.0 - (256.0 + span * 0.5)).abs() < 1e-3);
        assert!(((l.to[2] - l.to[1]) - (l.to[1] - l.to[0])).abs() < 1e-3);
        let help = l.place(PanelPart::Over(3), [300.0, 240.0, 100.0, 24.0]).unwrap();
        assert!((help[0] - (300.0 + l.to[3] - 384.0)).abs() < 1e-3 && help[1] == 240.0);
    }

    #[test]
    fn empty_boxes_are_hidden() {
        let solo = [true, false, false, false];
        let l = PanelLayout::new(100, false, true, 512.0, LX, solo, VIEW);
        assert!(l.changed());
        assert!(l.place(PanelPart::Panel(0), [8.0, 320.0, 8.0, 8.0]).is_some());
        assert_eq!(l.place(PanelPart::Panel(1), [136.0, 320.0, 8.0, 8.0]), None);
        assert!(l.backing(0).is_some() && l.backing(3).is_none());
        // Off: every box is drawn.
        assert!(!PanelLayout::new(100, false, false, 512.0, LX, solo, VIEW).changed());
    }
}
