//! The native settings overlay (M19, NATIVE: not an original menu; F10):
//! the enhancement settings and player 1's keys, changed in place and
//! stored in settings.json. The game waits while it is open. This module
//! is the overlay's state and text; gdl_play draws it and feeds it keys.
use crate::controls::{ACTIONS, Bindings};
use crate::settings::{CameraMode, Graphics, Prompts, Settings, TextureFilter};
use bevy::input::keyboard::KeyCode;

/// The overlay's input, already decoded from the keyboard.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Input {
    Up,
    Down,
    Left,
    Right,
    Accept,
    Back,
    /// Backspace/Delete: an action's default keys.
    Reset,
    /// Any key, while a binding waits for one.
    Key(KeyCode),
    /// NATIVE (this fork): a controller button, while a controller binding
    /// waits for one.
    Pad(bevy::input::gamepad::GamepadButton),
}

/// What an input did.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Effect {
    None,
    /// Settings or bindings changed (store them).
    Changed,
    Closed,
    /// NATIVE (this fork): the Save textures row; gdl_play starts or stops
    /// saving (not a setting: off at every start).
    SaveTextures,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Row {
    Widescreen,
    Blend,
    Vsync,
    FpsCap,
    FrameQueue,
    Fullscreen,
    Msaa,
    ReduceFlashes,
    ShowFps,
    Prompts,
    Camera,
    CameraRails,
    CameraInvert,
    Graphics,
    Hud,
    Controls,
    Pad,
    Original,
    Close,
}

const MAIN: [Row; 19] = [
    Row::Widescreen,
    Row::Blend,
    Row::Vsync,
    Row::FpsCap,
    Row::FrameQueue,
    Row::Fullscreen,
    Row::Msaa,
    Row::ReduceFlashes,
    Row::ShowFps,
    Row::Prompts,
    Row::Camera,
    Row::CameraRails,
    Row::CameraInvert,
    Row::Graphics,
    Row::Hud,
    Row::Controls,
    Row::Pad,
    Row::Original,
    Row::Close,
];
const FPS_CAPS: [u32; 7] = [0, 30, 60, 75, 120, 144, 240];

/// The graphics page's rows (post-1.0 enhancements).
#[derive(Clone, Copy, Debug, PartialEq)]
enum GRow {
    Preset,
    TextureFilter,
    Sharpen,
    FullColour,
    SmoothLightmaps,
    MoreLights,
    TorchLights,
    Shadows,
    AmbientOcclusion,
    Bloom,
    MaterialDetail,
    Water,
    Plants,
    GemGlow,
    GemGlass,
    GroundLight,
    HdTextures,
    TexturePack,
    ModelPack,
    SaveTextures,
    Back,
}

const GRAPHICS: [GRow; 21] = [
    GRow::Preset,
    GRow::TextureFilter,
    GRow::Sharpen,
    GRow::FullColour,
    GRow::SmoothLightmaps,
    GRow::MoreLights,
    GRow::TorchLights,
    GRow::Shadows,
    GRow::AmbientOcclusion,
    GRow::Bloom,
    GRow::MaterialDetail,
    GRow::Water,
    GRow::Plants,
    GRow::GemGlow,
    GRow::GemGlass,
    GRow::GroundLight,
    GRow::HdTextures,
    GRow::TexturePack,
    GRow::ModelPack,
    GRow::SaveTextures,
    GRow::Back,
];
const SHARPEN: [u32; 6] = [0, 20, 40, 60, 80, 100];

/// NATIVE (this fork, decisions/0011): the HUD layout page's rows.
#[derive(Clone, Copy, Debug, PartialEq)]
enum HRow {
    Size,
    Edges,
    HideEmpty,
    Back,
}

const HUD: [HRow; 4] = [HRow::Size, HRow::Edges, HRow::HideEmpty, HRow::Back];

/// The generated HD texture pack's state, as gdl_play reports it.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum HdStatus {
    #[default]
    Missing,
    Ready,
    /// The generator runs: textures done of total (total 0: scanning).
    Generating { done: usize, total: usize },
    Failed(String),
}

/// NATIVE (this fork): the texture folders as gdl_play reports them (the
/// Texture pack folder and Save textures rows).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TextureStatus {
    /// The replacement folder in use since startup (None: none).
    pub pack: Option<String>,
    /// Where textures are being saved (None: not saving).
    pub dump: Option<String>,
    /// Textures saved since startup.
    pub saved: usize,
}

#[derive(Default, bevy::prelude::Resource)]
pub struct Overlay {
    /// The HD pack's state (the HD textures row).
    pub hd: HdStatus,
    /// The texture folders' state (NATIVE, this fork).
    pub textures: TextureStatus,
    pub open: bool,
    /// Closed with keys still down: the game ignores the keyboard until
    /// every key is released (Enter would otherwise press Start).
    pub held: bool,
    controls: bool,
    /// The controller buttons page (NATIVE, this fork).
    pad: bool,
    /// The action waiting for a controller button.
    pad_capture: Option<usize>,
    /// The graphics page.
    graphics: bool,
    /// The HUD layout page (NATIVE, this fork).
    hud: bool,
    cursor: usize,
    /// The action waiting for a key.
    capture: Option<usize>,
}

fn on(v: bool) -> &'static str {
    if v { "On" } else { "Off" }
}

impl Overlay {
    pub fn toggle(&mut self) -> Effect {
        if self.open {
            *self = Self { held: true, ..self.fresh() };
            Effect::Closed
        } else {
            *self = Self { open: true, ..self.fresh() };
            Effect::None
        }
    }

    /// A closed overlay on the main page that keeps what gdl_play reports.
    fn fresh(&self) -> Self {
        Self { hd: self.hd.clone(), textures: self.textures.clone(), ..Self::default() }
    }

    /// Whether the next key goes to a binding.
    pub fn capturing(&self) -> bool {
        self.capture.is_some()
    }

    /// Whether the next controller button goes to a binding (NATIVE).
    pub fn capturing_pad(&self) -> bool {
        self.pad_capture.is_some()
    }

    /// The overlay's lines, the cursor's marked with "> ".
    pub fn lines(&self, s: &Settings, b: &Bindings) -> Vec<String> {
        let mark = |k: usize| if k == self.cursor { "> " } else { "  " };
        let mut out = Vec::new();
        if self.controls {
            out.push("PLAYER 1 KEYS (native)".to_string());
            out.push(String::new());
            for (k, (a, keys)) in ACTIONS.iter().zip(&b.keys).enumerate() {
                let names = if self.capture == Some(k) {
                    "press a key... (Escape / pad B: cancel)".to_string()
                } else {
                    keys.iter().map(|&k| crate::controls::key_name(k)).collect::<Vec<_>>().join(", ")
                };
                out.push(format!("{}{:<18} {}", mark(k), a.label, names));
            }
            out.push(format!("{}Back", mark(ACTIONS.len())));
            out.push(String::new());
            out.push("Enter/pad A: bind  Backspace: default  Escape/pad B: back".to_string());
            return out;
        }
        if self.pad {
            out.push("CONTROLLER BUTTONS (native; every controller)".to_string());
            out.push(String::new());
            for (k, (a, buttons)) in ACTIONS.iter().zip(&b.pad).enumerate() {
                let names = if self.pad_capture == Some(k) {
                    "press a controller button... (Escape: cancel)".to_string()
                } else if buttons.is_empty() {
                    "none".to_string()
                } else {
                    buttons.iter().map(|&p| crate::controls::pad_label(p)).collect::<Vec<_>>().join(", ")
                };
                out.push(format!("{}{:<18} {}", mark(k), a.label, names));
            }
            out.push(format!("{}Standard layout", mark(ACTIONS.len())));
            out.push(format!("{}Back", mark(ACTIONS.len() + 1)));
            out.push(String::new());
            out.push("While a level plays. Menus, the select screen, the shop and the".to_string());
            out.push("map keep the standard buttons. Moving a button swaps the two.".to_string());
            out.push("Enter/pad A: bind  Backspace: standard  Escape/pad B: back".to_string());
            return out;
        }
        if self.hud {
            out.push("HUD LAYOUT (native)".to_string());
            out.push(String::new());
            for (k, row) in HUD.iter().enumerate() {
                let text = match row {
                    HRow::Size => format!("HUD size             {} %", s.hud_size),
                    HRow::Edges => format!("Panels at the edges  {}", on(s.hud_edges)),
                    HRow::HideEmpty => format!("Hide empty panels    {}", on(s.hud_hide_empty)),
                    HRow::Back => "Back".to_string(),
                };
                out.push(format!("{}{text}", mark(k)));
            }
            out.push(String::new());
            match HUD.get(self.cursor) {
                Some(HRow::Size) => {
                    out.push("HUD size: the player panels smaller, at the bottom of the screen;".to_string());
                    out.push("the game view shows in the space they leave.".to_string());
                }
                Some(HRow::Edges) => {
                    out.push("Panels at the edges: in the widescreen view, player 1's panel at".to_string());
                    out.push("the left edge and player 4's at the right, the others between.".to_string());
                }
                Some(HRow::HideEmpty) => {
                    out.push("Hide empty panels: no panel for a player not in the game; it".to_string());
                    out.push("comes back when someone joins.".to_string());
                }
                _ => {}
            }
            out.push("Arrows / d-pad: choose   Enter / pad A / Left / Right: change".to_string());
            out.push("Escape / pad B: back".to_string());
            return out;
        }
        if self.graphics {
            out.push("GRAPHICS (native enhancements)".to_string());
            out.push(String::new());
            for (k, row) in GRAPHICS.iter().enumerate() {
                let text = match row {
                    GRow::Preset => format!(
                        "Preset               {}",
                        match s.graphics() {
                            Graphics::Original => "Original",
                            Graphics::Enhanced => "Enhanced",
                            Graphics::Custom => "Custom",
                        }
                    ),
                    GRow::TextureFilter => format!(
                        "Texture filtering    {}",
                        match s.texture_filter {
                            TextureFilter::Original => "Original",
                            TextureFilter::Sharp => "Sharp",
                        }
                    ),
                    GRow::Sharpen => format!("Sharpening           {}", if s.sharpen == 0 { "Off".to_string() } else { format!("{} %", s.sharpen) }),
                    GRow::FullColour => format!("Colour precision     {}", if s.full_colour { "Full" } else { "Original 16-bit" }),
                    GRow::SmoothLightmaps => format!("Smooth lightmaps     {}", on(s.smooth_lightmaps)),
                    GRow::MoreLights => format!("Dynamic lights       {}", if s.more_lights { "Up to 32" } else { "Original 8" }),
                    GRow::TorchLights => format!("Torch light          {}", on(s.torch_lights)),
                    GRow::Shadows => format!("Character shadows    {}", if s.shadows { "Shadow maps" } else { "Original" }),
                    GRow::AmbientOcclusion => format!("Ambient occlusion    {}", on(s.ambient_occlusion)),
                    GRow::Bloom => format!("Bloom                {}", on(s.bloom)),
                    // NATIVE (this fork, decisions/0015).
                    GRow::MaterialDetail => format!("Material detail      {}", on(s.material_detail)),
                    // NATIVE (this fork, decisions/0016).
                    GRow::Water => format!("Water effects        {}", on(s.water_effects)),
                    // NATIVE (this fork, decisions/0017).
                    GRow::Plants => format!("Plant effects        {}", on(s.plant_effects)),
                    // NATIVE (this fork, decisions/0018).
                    GRow::GemGlow => format!("Gem and potion glow  {}", on(s.gem_glow)),
                    // NATIVE (this fork, decisions/0053).
                    GRow::GemGlass => format!("See-through crystals {}", on(s.gem_glass)),
                    // NATIVE (this fork, decisions/0027).
                    GRow::GroundLight => format!("Items in shade       {}", on(s.item_ground_light)),
                    GRow::HdTextures => format!(
                        "HD textures          {}",
                        match (s.hd_textures, &self.hd) {
                            (false, _) => "Off".to_string(),
                            (true, HdStatus::Ready) => "On".to_string(),
                            (true, HdStatus::Missing) => "On (starting...)".to_string(),
                            (true, HdStatus::Generating { total: 0, .. }) => "On (reading the disc...)".to_string(),
                            (true, HdStatus::Generating { done, total }) => format!("On (generating {} %)", done * 100 / total),
                            (true, HdStatus::Failed(e)) => format!("On (failed: {})", e.chars().take(40).collect::<String>()),
                        }
                    ),
                    // NATIVE (this fork): the folder is read at startup.
                    GRow::TexturePack => {
                        let want = s.texture_pack.is_some() || s.texture_pack_folder;
                        let state = if s.texture_pack.is_some() { "Off (a path is set)" } else { on(s.texture_pack_folder) };
                        let later = if want != self.textures.pack.is_some() { " (next start)" } else { "" };
                        format!("Texture pack folder  {state}{later}")
                    }
                    // NATIVE (this fork, decisions/0022): read at startup.
                    GRow::ModelPack => {
                        let later = if s.model_pack_folder != crate::model_pack::active() { " (next start)" } else { "" };
                        format!("Model pack folder    {}{later}", on(s.model_pack_folder))
                    }
                    GRow::SaveTextures => {
                        format!("Save textures        {}", if self.textures.dump.is_some() { format!("On ({} saved)", self.textures.saved) } else { "Off".to_string() })
                    }
                    GRow::Back => "Back".to_string(),
                };
                out.push(format!("{}{text}", mark(k)));
            }
            out.push(String::new());
            if GRAPHICS.get(self.cursor) == Some(&GRow::TexturePack) {
                out.push("Texture pack: PNG files in texture_pack beside the game replace".to_string());
                out.push("its textures (names as in texture_dump). F5 in play reloads them.".to_string());
            }
            if GRAPHICS.get(self.cursor) == Some(&GRow::ModelPack) {
                out.push("Model pack: .glb files in model_pack beside the game replace its".to_string());
                out.push("objects (names as in the model export). Read at startup.".to_string());
            }
            if GRAPHICS.get(self.cursor) == Some(&GRow::SaveTextures) {
                out.push("Save textures: textures loaded from now on go to texture_dump".to_string());
                out.push("beside the game. Loading is slower meanwhile; off at next start.".to_string());
            }
            if GRAPHICS.get(self.cursor) == Some(&GRow::MaterialDetail) {
                out.push("Material detail: bump and shine on stone, wood, metal and more,".to_string());
                out.push("lit by torches, spells and the sun. Turned on, from the next level.".to_string());
            }
            if GRAPHICS.get(self.cursor) == Some(&GRow::Water) {
                out.push("Water effects: ripples, glints from the sun and torches, a sheen,".to_string());
                out.push("soft shores (with ambient occlusion). Turned on, from the next level.".to_string());
            }
            if GRAPHICS.get(self.cursor) == Some(&GRow::Plants) {
                out.push("Plant effects: leaves, moss and grass sway in the wind; torches and".to_string());
                out.push("spells shine through leaves. Turned on, from the next level.".to_string());
            }
            if GRAPHICS.get(self.cursor) == Some(&GRow::GemGlow) {
                out.push("Gem and potion glow: crystals, runestones, gargoyle items and magic".to_string());
                out.push("potions glow softly in their colour, and flare when picked up;".to_string());
                out.push("the crystals' aura breathes softly, with turning rays and glints.".to_string());
            }
            if GRAPHICS.get(self.cursor) == Some(&GRow::GemGlass) {
                out.push("See-through crystals: the crystals' shells drawn as glass, clearer".to_string());
                out.push("through the middle, their glowing cores showing inside.".to_string());
            }
            if GRAPHICS.get(self.cursor) == Some(&GRow::GroundLight) {
                out.push("Items in shade: barrels, chests and pickups take the ground's light".to_string());
                out.push("and colour: darker in shade, still lit by torches and spells.".to_string());
            }
            if GRAPHICS.get(self.cursor) == Some(&GRow::HdTextures) {
                out.push("HD textures: made once from your disc (several minutes, about 2 GB);".to_string());
                out.push("they show from the next level. Effects and skies stay original.".to_string());
            }
            out.push("Arrows / d-pad: choose   Enter / pad A / Left / Right: change".to_string());
            out.push("Escape / pad B: back".to_string());
            return out;
        }
        out.push("SETTINGS (native enhancements)".to_string());
        out.push(String::new());
        for (k, row) in MAIN.iter().enumerate() {
            let text = match row {
                Row::Widescreen => format!("Widescreen view      {}", on(s.widescreen)),
                Row::Blend => format!("Motion blending      {}", on(s.interpolate)),
                Row::Vsync => format!("VSync                {}", on(s.vsync)),
                Row::FpsCap => format!("Frame cap            {}", if s.fps_cap == 0 { "Off".to_string() } else { s.fps_cap.to_string() }),
                Row::FrameQueue => format!(
                    "Frames queued        {} ({})",
                    s.frame_queue,
                    match s.frame_queue {
                        1 => "lowest latency",
                        2 => "default",
                        _ => "more buffering",
                    }
                ),
                Row::Fullscreen => format!("Fullscreen (F11)     {}", on(s.fullscreen)),
                Row::Msaa => format!(
                    "Anti-aliasing        {}",
                    if s.smaa {
                        "SMAA".to_string()
                    } else if s.msaa <= 1 {
                        "Off".to_string()
                    } else if s.ambient_occlusion {
                        format!("SMAA (for {}x MSAA: ambient occlusion)", s.msaa)
                    } else {
                        format!("{}x MSAA", s.msaa)
                    }
                ),
                Row::ReduceFlashes => format!("Reduce flashes       {}", on(s.reduce_flashes)),
                Row::ShowFps => format!("Frame rate (F3)      {}", on(s.show_fps)),
                Row::Prompts => format!(
                    "Button prompts       {}",
                    match s.button_prompts {
                        Prompts::Auto => "Automatic",
                        Prompts::PlayStation => "PlayStation",
                        Prompts::Xbox => "Xbox",
                    }
                ),
                Row::Camera => format!(
                    "Camera               {}",
                    match s.camera {
                        CameraMode::Original => "Original",
                        CameraMode::Free => "Free (right stick)",
                    }
                ),
                Row::CameraRails => format!("  Turns with level   {}", on(s.camera_follow_rails)),
                Row::CameraInvert => format!("  Invert turning     {}", on(s.camera_invert)),
                Row::Graphics => "Graphics...".to_string(),
                Row::Hud => "HUD layout...".to_string(),
                Row::Controls => "Player 1 keys...".to_string(),
                Row::Pad => "Controller buttons...".to_string(),
                Row::Original => "Original presentation".to_string(),
                Row::Close => "Close".to_string(),
            };
            out.push(format!("{}{text}", mark(k)));
        }
        out.push(String::new());
        if matches!(MAIN.get(self.cursor), Some(Row::Camera | Row::CameraRails | Row::CameraInvert)) {
            out.push("Free camera: right stick turns (left/right) and zooms (up/down; the".to_string());
            out.push("view tilts with the zoom). Click the stick: back to the level's".to_string());
            out.push("direction; hold: the zoom too. Numpad 4/6, 8/2, 5. Off in network play.".to_string());
        }
        // NATIVE (this fork): the pack in use (gdl_play reports it), else the setting's path.
        let pack = self.textures.pack.clone().or_else(|| s.texture_pack.as_ref().map(|p| p.display().to_string()));
        out.push(format!("Texture pack: {}", pack.as_deref().unwrap_or("none")));
        out.push("Arrows / d-pad: choose   Enter / pad A / Left / Right: change".to_string());
        out.push("Escape / pad B / F10: close".to_string());
        out
    }

    /// Apply one input.
    pub fn input(&mut self, i: Input, s: &mut Settings, b: &mut Bindings) -> Effect {
        if !self.open {
            return Effect::None;
        }
        if let Some(a) = self.capture {
            return match i {
                Input::Back => {
                    self.capture = None;
                    Effect::None
                }
                Input::Key(k) if crate::controls::BINDABLE.contains(&k) => {
                    self.capture = None;
                    b.bind(a, k);
                    s.keys = b.to_settings();
                    Effect::Changed
                }
                _ => Effect::None,
            };
        }
        if let Some(a) = self.pad_capture {
            return match i {
                Input::Back => {
                    self.pad_capture = None;
                    Effect::None
                }
                Input::Pad(p) if crate::controls::PAD_BINDABLE.contains(&p) => {
                    self.pad_capture = None;
                    b.bind_pad(a, p);
                    s.pad = b.pad_to_settings();
                    Effect::Changed
                }
                _ => Effect::None,
            };
        }
        let rows = if self.controls {
            ACTIONS.len() + 1
        } else if self.pad {
            ACTIONS.len() + 2
        } else if self.graphics {
            GRAPHICS.len()
        } else if self.hud {
            HUD.len()
        } else {
            MAIN.len()
        };
        match i {
            Input::Up => {
                self.cursor = (self.cursor + rows - 1) % rows;
                Effect::None
            }
            Input::Down => {
                self.cursor = (self.cursor + 1) % rows;
                Effect::None
            }
            Input::Back if self.controls => {
                *self = Self { open: true, cursor: MAIN.iter().position(|r| *r == Row::Controls).unwrap_or(0), ..self.fresh() };
                Effect::None
            }
            Input::Back if self.pad => {
                *self = Self { open: true, cursor: MAIN.iter().position(|r| *r == Row::Pad).unwrap_or(0), ..self.fresh() };
                Effect::None
            }
            _ if self.pad => {
                let at = self.cursor;
                match (i, at.cmp(&ACTIONS.len())) {
                    (Input::Accept, std::cmp::Ordering::Less) => {
                        self.pad_capture = Some(at);
                        Effect::None
                    }
                    // Backspace: the action's standard button (swapped in).
                    (Input::Reset, std::cmp::Ordering::Less) => {
                        match ACTIONS[at].pad.first() {
                            Some(&p) => b.bind_pad(at, p),
                            None => b.pad[at].clear(),
                        }
                        s.pad = b.pad_to_settings();
                        Effect::Changed
                    }
                    (Input::Accept, std::cmp::Ordering::Equal) => {
                        b.reset_pad();
                        s.pad = b.pad_to_settings();
                        Effect::Changed
                    }
                    (Input::Accept, std::cmp::Ordering::Greater) => self.input(Input::Back, s, b),
                    _ => Effect::None,
                }
            }
            Input::Back if self.graphics => {
                *self = Self { open: true, cursor: MAIN.iter().position(|r| *r == Row::Graphics).unwrap_or(0), ..self.fresh() };
                Effect::None
            }
            Input::Back if self.hud => {
                *self = Self { open: true, cursor: MAIN.iter().position(|r| *r == Row::Hud).unwrap_or(0), ..self.fresh() };
                Effect::None
            }
            Input::Accept | Input::Left | Input::Right if self.hud => {
                match HUD[self.cursor] {
                    HRow::Size => {
                        let sizes = crate::hud_layout::SIZES;
                        let at = sizes.iter().position(|&v| v == s.hud_size).unwrap_or(sizes.len() - 1);
                        s.hud_size = sizes[if i == Input::Left { (at + sizes.len() - 1) % sizes.len() } else { (at + 1) % sizes.len() }];
                    }
                    HRow::Edges => s.hud_edges = !s.hud_edges,
                    HRow::HideEmpty => s.hud_hide_empty = !s.hud_hide_empty,
                    HRow::Back => {
                        if i != Input::Accept {
                            return Effect::None;
                        }
                        return self.input(Input::Back, s, b);
                    }
                }
                Effect::Changed
            }
            Input::Back => self.toggle(),
            Input::Accept | Input::Left | Input::Right if self.graphics => {
                let back = i == Input::Left;
                match GRAPHICS[self.cursor] {
                    GRow::Preset => {
                        let next = if s.graphics() == Graphics::Enhanced { Graphics::Original } else { Graphics::Enhanced };
                        *s = s.with_graphics(next);
                    }
                    GRow::TextureFilter => {
                        s.texture_filter = if s.texture_filter == TextureFilter::Sharp { TextureFilter::Original } else { TextureFilter::Sharp };
                    }
                    GRow::Sharpen => {
                        let at = SHARPEN.iter().position(|&v| v == s.sharpen).unwrap_or(0);
                        s.sharpen = SHARPEN[if back { (at + SHARPEN.len() - 1) % SHARPEN.len() } else { (at + 1) % SHARPEN.len() }];
                    }
                    GRow::FullColour => s.full_colour = !s.full_colour,
                    GRow::SmoothLightmaps => s.smooth_lightmaps = !s.smooth_lightmaps,
                    GRow::MoreLights => s.more_lights = !s.more_lights,
                    GRow::TorchLights => s.torch_lights = !s.torch_lights,
                    GRow::Shadows => s.shadows = !s.shadows,
                    GRow::AmbientOcclusion => s.ambient_occlusion = !s.ambient_occlusion,
                    GRow::Bloom => s.bloom = !s.bloom,
                    GRow::MaterialDetail => s.material_detail = !s.material_detail,
                    GRow::Water => s.water_effects = !s.water_effects,
                    GRow::Plants => s.plant_effects = !s.plant_effects,
                    GRow::GemGlow => s.gem_glow = !s.gem_glow,
                    GRow::GemGlass => s.gem_glass = !s.gem_glass,
                    GRow::GroundLight => s.item_ground_light = !s.item_ground_light,
                    GRow::HdTextures => s.hd_textures = !s.hd_textures,
                    GRow::TexturePack => s.texture_pack_folder = !s.texture_pack_folder,
                    GRow::ModelPack => s.model_pack_folder = !s.model_pack_folder,
                    GRow::SaveTextures => return Effect::SaveTextures,
                    GRow::Back => {
                        if i != Input::Accept {
                            return Effect::None;
                        }
                        return self.input(Input::Back, s, b);
                    }
                }
                Effect::Changed
            }
            _ if self.controls => {
                if self.cursor == ACTIONS.len() {
                    if i == Input::Accept {
                        return self.input(Input::Back, s, b);
                    }
                    return Effect::None;
                }
                match i {
                    Input::Accept => {
                        self.capture = Some(self.cursor);
                        Effect::None
                    }
                    Input::Reset => {
                        b.keys[self.cursor] = ACTIONS[self.cursor].defaults.to_vec();
                        // Defaults may be taken by another action meanwhile.
                        for &k in ACTIONS[self.cursor].defaults {
                            for (j, keys) in b.keys.iter_mut().enumerate() {
                                if j != self.cursor {
                                    keys.retain(|&x| x != k);
                                }
                            }
                        }
                        s.keys = b.to_settings();
                        Effect::Changed
                    }
                    _ => Effect::None,
                }
            }
            Input::Accept | Input::Left | Input::Right => {
                let back = i == Input::Left;
                match MAIN[self.cursor] {
                    Row::Widescreen => s.widescreen = !s.widescreen,
                    Row::Blend => s.interpolate = !s.interpolate,
                    Row::Vsync => s.vsync = !s.vsync,
                    Row::Fullscreen => s.fullscreen = !s.fullscreen,
                    Row::ReduceFlashes => s.reduce_flashes = !s.reduce_flashes,
                    Row::ShowFps => s.show_fps = !s.show_fps,
                    Row::Camera => {
                        s.camera = match s.camera {
                            CameraMode::Original => CameraMode::Free,
                            CameraMode::Free => CameraMode::Original,
                        }
                    }
                    Row::CameraRails => s.camera_follow_rails = !s.camera_follow_rails,
                    Row::CameraInvert => s.camera_invert = !s.camera_invert,
                    Row::Prompts => {
                        let order = [Prompts::Auto, Prompts::PlayStation, Prompts::Xbox];
                        let at = order.iter().position(|&p| p == s.button_prompts).unwrap_or(0);
                        s.button_prompts = order[if back { (at + 2) % 3 } else { (at + 1) % 3 }];
                    }
                    Row::FpsCap => {
                        let at = FPS_CAPS.iter().position(|&c| c == s.fps_cap).unwrap_or(0);
                        s.fps_cap = FPS_CAPS[if back { (at + FPS_CAPS.len() - 1) % FPS_CAPS.len() } else { (at + 1) % FPS_CAPS.len() }];
                    }
                    Row::FrameQueue => {
                        s.frame_queue = if back { (s.frame_queue + 1) % 3 + 1 } else { s.frame_queue % 3 + 1 };
                    }
                    Row::Msaa => {
                        // Off, 2x, 4x MSAA, SMAA.
                        let order = [(1, false), (2, false), (4, false), (1, true)];
                        let at = order.iter().position(|&m| m == (s.msaa, s.smaa)).unwrap_or(0);
                        (s.msaa, s.smaa) = order[if back { (at + 3) % 4 } else { (at + 1) % 4 }];
                    }
                    Row::Graphics => {
                        if i != Input::Accept {
                            return Effect::None;
                        }
                        *self = Self { open: true, graphics: true, ..self.fresh() };
                        return Effect::None;
                    }
                    Row::Hud => {
                        if i != Input::Accept {
                            return Effect::None;
                        }
                        *self = Self { open: true, hud: true, ..self.fresh() };
                        return Effect::None;
                    }
                    Row::Controls => {
                        if i != Input::Accept {
                            return Effect::None;
                        }
                        *self = Self { open: true, controls: true, ..self.fresh() };
                        return Effect::None;
                    }
                    Row::Pad => {
                        if i != Input::Accept {
                            return Effect::None;
                        }
                        *self = Self { open: true, pad: true, ..self.fresh() };
                        return Effect::None;
                    }
                    Row::Original => {
                        if i != Input::Accept {
                            return Effect::None;
                        }
                        *s = s.with_baseline_presentation();
                    }
                    Row::Close => {
                        if i != Input::Accept {
                            return Effect::None;
                        }
                        return self.toggle();
                    }
                }
                Effect::Changed
            }
            _ => Effect::None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn open() -> (Overlay, Settings, Bindings) {
        let mut o = Overlay::default();
        o.toggle();
        (o, Settings::default(), Bindings::default())
    }

    #[test]
    fn rows_toggle_and_cycle() {
        let (mut o, mut s, mut b) = open();
        assert_eq!(o.input(Input::Accept, &mut s, &mut b), Effect::Changed);
        assert!(s.widescreen);
        for _ in 0..3 {
            o.input(Input::Down, &mut s, &mut b);
        }
        o.input(Input::Right, &mut s, &mut b);
        assert_eq!(s.fps_cap, 30);
        o.input(Input::Left, &mut s, &mut b);
        o.input(Input::Left, &mut s, &mut b);
        assert_eq!(s.fps_cap, 240);
        assert_eq!(o.input(Input::Back, &mut s, &mut b), Effect::Closed);
        assert!(!o.open);
    }

    #[test]
    fn binding_a_key_stores_it() {
        let (mut o, mut s, mut b) = open();
        while o.lines(&s, &b).iter().all(|l| !l.starts_with("> Player 1 keys")) {
            o.input(Input::Down, &mut s, &mut b);
        }
        o.input(Input::Accept, &mut s, &mut b);
        let cross = ACTIONS.iter().position(|a| a.name == "cross").unwrap();
        for _ in 0..cross {
            o.input(Input::Down, &mut s, &mut b);
        }
        o.input(Input::Accept, &mut s, &mut b);
        assert!(o.capturing());
        assert_eq!(o.input(Input::Key(KeyCode::KeyN), &mut s, &mut b), Effect::Changed);
        assert_eq!(b.of("cross"), &[KeyCode::KeyN]);
        assert_eq!(s.keys.get("cross"), Some(&vec!["KeyN".to_string()]));
        assert_eq!(o.input(Input::Reset, &mut s, &mut b), Effect::Changed);
        assert!(s.keys.is_empty());
    }

    #[test]
    fn button_prompts_cycle() {
        let (mut o, mut s, mut b) = open();
        while o.lines(&s, &b).iter().all(|l| !l.starts_with("> Button prompts")) {
            o.input(Input::Down, &mut s, &mut b);
        }
        assert_eq!(o.input(Input::Right, &mut s, &mut b), Effect::Changed);
        assert_eq!(s.button_prompts, Prompts::PlayStation);
        o.input(Input::Accept, &mut s, &mut b);
        assert_eq!(s.button_prompts, Prompts::Xbox);
        o.input(Input::Left, &mut s, &mut b);
        o.input(Input::Left, &mut s, &mut b);
        assert_eq!(s.button_prompts, Prompts::Auto);
        assert!(o.lines(&s, &b).iter().any(|l| l == "> Button prompts       Automatic"));
    }

    #[test]
    fn graphics_page_presets_and_rows() {
        let (mut o, s, mut b) = open();
        let mut s = s.with_graphics(Graphics::Original);
        while o.lines(&s, &b).iter().all(|l| !l.starts_with("> Graphics...")) {
            o.input(Input::Down, &mut s, &mut b);
        }
        o.input(Input::Accept, &mut s, &mut b);
        assert!(o.lines(&s, &b).iter().any(|l| l == "> Preset               Original"));
        assert_eq!(o.input(Input::Accept, &mut s, &mut b), Effect::Changed);
        assert_eq!(s.graphics(), Graphics::Enhanced);
        // Sharpening steps, then the preset reads Custom.
        o.input(Input::Down, &mut s, &mut b);
        o.input(Input::Down, &mut s, &mut b);
        o.input(Input::Right, &mut s, &mut b);
        assert_eq!(s.sharpen, 60);
        assert_eq!(s.graphics(), Graphics::Custom);
        o.input(Input::Up, &mut s, &mut b);
        o.input(Input::Up, &mut s, &mut b);
        o.input(Input::Accept, &mut s, &mut b);
        assert_eq!(s.graphics(), Graphics::Enhanced);
        o.input(Input::Accept, &mut s, &mut b);
        assert_eq!(s.graphics(), Graphics::Original);
        // Back returns to the main page at "Graphics...".
        o.input(Input::Back, &mut s, &mut b);
        assert!(o.open && o.lines(&s, &b).iter().any(|l| l == "> Graphics..."));
    }

    /// NATIVE (this fork, decisions/0012): the controller buttons page.
    #[test]
    fn controller_buttons_page() {
        use bevy::input::gamepad::GamepadButton;
        let (mut o, mut s, mut b) = open();
        o.cursor = MAIN.iter().position(|r| *r == Row::Pad).unwrap();
        assert!(o.lines(&s, &b).iter().any(|l| l == "> Controller buttons..."));
        o.input(Input::Accept, &mut s, &mut b);
        assert!(o.lines(&s, &b).iter().any(|l| l == "  Cross (attack)     A / Cross"));
        let cross = ACTIONS.iter().position(|a| a.name == "cross").unwrap();
        for _ in 0..cross {
            o.input(Input::Down, &mut s, &mut b);
        }
        o.input(Input::Accept, &mut s, &mut b);
        assert!(o.capturing_pad() && !o.capturing());
        // Keys and unbindable buttons wait; RB binds and swaps.
        assert_eq!(o.input(Input::Key(KeyCode::KeyN), &mut s, &mut b), Effect::None);
        assert_eq!(o.input(Input::Pad(GamepadButton::Mode), &mut s, &mut b), Effect::None);
        assert_eq!(o.input(Input::Pad(GamepadButton::RightTrigger), &mut s, &mut b), Effect::Changed);
        assert!(!o.capturing_pad());
        assert_eq!(s.pad.get("cross"), Some(&vec!["RightTrigger".to_string()]));
        assert!(o.lines(&s, &b).iter().any(|l| l == "  R1 (strafe)        A / Cross"));
        // Escape cancels a wait; Backspace gives the standard button back.
        o.input(Input::Accept, &mut s, &mut b);
        assert_eq!(o.input(Input::Back, &mut s, &mut b), Effect::None);
        assert!(o.open && !o.capturing_pad());
        assert_eq!(o.input(Input::Reset, &mut s, &mut b), Effect::Changed);
        assert!(s.pad.is_empty());
        // Standard layout, then Back to the main page.
        o.input(Input::Accept, &mut s, &mut b);
        o.input(Input::Pad(GamepadButton::West), &mut s, &mut b);
        assert!(!s.pad.is_empty());
        o.cursor = ACTIONS.len();
        assert_eq!(o.input(Input::Accept, &mut s, &mut b), Effect::Changed);
        assert!(s.pad.is_empty() && b == Bindings::default());
        o.input(Input::Down, &mut s, &mut b);
        o.input(Input::Accept, &mut s, &mut b);
        assert!(o.open && o.lines(&s, &b).iter().any(|l| l == "> Controller buttons..."));
    }

    /// NATIVE (this fork, decisions/0011): the HUD layout page.
    #[test]
    fn hud_layout_rows() {
        let (mut o, mut s, mut b) = open();
        o.cursor = MAIN.iter().position(|r| *r == Row::Hud).unwrap();
        assert!(o.lines(&s, &b).iter().any(|l| l == "> HUD layout..."));
        assert_eq!(o.input(Input::Accept, &mut s, &mut b), Effect::None);
        assert!(o.lines(&s, &b).iter().any(|l| l == "> HUD size             100 %"));
        // Left: smaller; Right and Enter: larger, from 100 round to 50.
        assert_eq!(o.input(Input::Left, &mut s, &mut b), Effect::Changed);
        assert_eq!(s.hud_size, 90);
        o.input(Input::Right, &mut s, &mut b);
        o.input(Input::Accept, &mut s, &mut b);
        assert_eq!(s.hud_size, 50);
        o.input(Input::Down, &mut s, &mut b);
        assert!(o.lines(&s, &b).iter().any(|l| l == "> Panels at the edges  On"));
        o.input(Input::Accept, &mut s, &mut b);
        o.input(Input::Down, &mut s, &mut b);
        o.input(Input::Right, &mut s, &mut b);
        assert!(!s.hud_edges && !s.hud_hide_empty);
        assert!(o.lines(&s, &b).iter().any(|l| l.starts_with("Hide empty panels: no panel")));
        // Back returns to the main page at "HUD layout..."; Original
        // presentation restores the original layout.
        o.input(Input::Back, &mut s, &mut b);
        assert!(o.open && o.lines(&s, &b).iter().any(|l| l == "> HUD layout..."));
        let back = Settings { hud_size: 60, hud_edges: true, ..s.clone() }.with_baseline_presentation();
        assert!(back.hud_size == 100 && !back.hud_edges && !back.hud_hide_empty);
    }

    /// NATIVE (this fork): the texture folder rows. The folder is a stored
    /// setting read at startup; Save textures asks gdl_play and shows what
    /// it reports, also after a page change.
    #[test]
    fn texture_folder_rows() {
        let (mut o, mut s, mut b) = open();
        o.textures = TextureStatus { pack: Some("C:\\game\\texture_pack".into()), dump: None, saved: 0 };
        assert!(o.lines(&s, &b).iter().any(|l| l == "Texture pack: C:\\game\\texture_pack"));
        o.cursor = MAIN.iter().position(|r| *r == Row::Graphics).unwrap();
        o.input(Input::Accept, &mut s, &mut b);
        let row = |o: &Overlay, s: &Settings, name: &str| o.lines(s, &Bindings::default()).into_iter().find(|l| l.contains(name)).unwrap();
        o.cursor = GRAPHICS.iter().position(|r| *r == GRow::TexturePack).unwrap();
        assert!(row(&o, &s, "Texture pack folder").ends_with("folder  On"));
        assert_eq!(o.input(Input::Accept, &mut s, &mut b), Effect::Changed);
        assert!(!s.texture_pack_folder);
        assert!(row(&o, &s, "Texture pack folder").ends_with("Off (next start)"));
        assert!(o.lines(&s, &b).iter().any(|l| l.starts_with("Texture pack: PNG files")));
        o.cursor = GRAPHICS.iter().position(|r| *r == GRow::SaveTextures).unwrap();
        assert!(row(&o, &s, "Save textures  ").ends_with("Off"));
        let before = s.clone();
        assert_eq!(o.input(Input::Accept, &mut s, &mut b), Effect::SaveTextures);
        assert_eq!(s, before);
        o.textures.dump = Some("C:\\game\\texture_dump".into());
        o.textures.saved = 12;
        assert!(row(&o, &s, "Save textures  ").ends_with("On (12 saved)"));
        o.input(Input::Back, &mut s, &mut b);
        assert_eq!(o.textures.saved, 12);
        // The main page's Original presentation turns the folder off too.
        assert!(!Settings::default().with_baseline_presentation().texture_pack_folder);
        // NATIVE (this fork, decisions/0022): the model pack row.
        o.cursor = MAIN.iter().position(|r| *r == Row::Graphics).unwrap();
        o.input(Input::Accept, &mut s, &mut b);
        o.cursor = GRAPHICS.iter().position(|r| *r == GRow::ModelPack).unwrap();
        assert!(row(&o, &s, "Model pack folder").ends_with("folder    On (next start)"));
        assert_eq!(o.input(Input::Accept, &mut s, &mut b), Effect::Changed);
        assert!(!s.model_pack_folder);
        assert!(row(&o, &s, "Model pack folder").ends_with("folder    Off"));
        assert!(o.lines(&s, &b).iter().any(|l| l.starts_with("Model pack: .glb files")));
        assert!(!Settings::default().with_baseline_presentation().model_pack_folder);
    }

    #[test]
    fn hd_textures_row_toggles_and_shows_the_pack_state() {
        let (mut o, mut s, mut b) = open();
        o.cursor = MAIN.iter().position(|r| *r == Row::Graphics).unwrap();
        o.input(Input::Accept, &mut s, &mut b);
        o.cursor = GRAPHICS.iter().position(|r| *r == GRow::HdTextures).unwrap();
        let row = |o: &Overlay, s: &Settings| o.lines(s, &Bindings::default()).into_iter().find(|l| l.contains("HD textures  ")).unwrap();
        assert!(row(&o, &s).ends_with("Off"));
        assert_eq!(o.input(Input::Accept, &mut s, &mut b), Effect::Changed);
        assert!(s.hd_textures);
        o.hd = HdStatus::Generating { done: 0, total: 0 };
        assert!(row(&o, &s).contains("reading the disc"));
        o.hd = HdStatus::Generating { done: 50, total: 200 };
        assert!(row(&o, &s).contains("generating 25 %"));
        o.hd = HdStatus::Ready;
        assert!(row(&o, &s).ends_with("On"));
        // The note under the page while the row is selected.
        assert!(o.lines(&s, &Bindings::default()).iter().any(|l| l.contains("made once from your disc")));
    }

    #[test]
    fn anti_aliasing_cycles_through_smaa() {
        let (mut o, mut s, mut b) = open();
        s.msaa = 1;
        s.ambient_occlusion = false;
        while o.lines(&s, &b).iter().all(|l| !l.starts_with("> Anti-aliasing")) {
            o.input(Input::Down, &mut s, &mut b);
        }
        let mut seen = Vec::new();
        for _ in 0..4 {
            o.input(Input::Right, &mut s, &mut b);
            seen.push((s.msaa, s.smaa));
        }
        assert_eq!(seen, [(2, false), (4, false), (1, true), (1, false)]);
        o.input(Input::Left, &mut s, &mut b);
        assert_eq!((s.msaa, s.smaa), (1, true));
        assert!(o.lines(&s, &b).iter().any(|l| l == "> Anti-aliasing        SMAA"));
        // Ambient occlusion shows the SMAA that stands in for MSAA.
        s.smaa = false;
        s.msaa = 4;
        s.ambient_occlusion = true;
        assert!(o.lines(&s, &b).iter().any(|l| l.ends_with("SMAA (for 4x MSAA: ambient occlusion)")));
    }

    #[test]
    fn camera_rows_toggle() {
        let (mut o, mut s, mut b) = open();
        assert_eq!(s.camera, CameraMode::Free);
        assert_eq!(Settings::baseline().camera, CameraMode::Original);
        while o.lines(&s, &b).iter().all(|l| !l.starts_with("> Camera")) {
            o.input(Input::Down, &mut s, &mut b);
        }
        assert_eq!(o.input(Input::Accept, &mut s, &mut b), Effect::Changed);
        assert_eq!(s.camera, CameraMode::Original);
        o.input(Input::Accept, &mut s, &mut b);
        assert_eq!(s.camera, CameraMode::Free);
        assert!(o.lines(&s, &b).iter().any(|l| l == "> Camera               Free (right stick)"));
        assert!(o.lines(&s, &b).iter().any(|l| l.starts_with("Free camera: right stick")));
        o.input(Input::Down, &mut s, &mut b);
        o.input(Input::Accept, &mut s, &mut b);
        assert!(!s.camera_follow_rails);
        o.input(Input::Down, &mut s, &mut b);
        o.input(Input::Accept, &mut s, &mut b);
        assert!(s.camera_invert);
    }

    #[test]
    fn original_presentation_keeps_window_and_keys() {
        let (mut o, mut s, mut b) = open();
        s.widescreen = true;
        s.fullscreen = true;
        s.msaa = 4;
        s.button_prompts = Prompts::Xbox;
        while o.lines(&s, &b).iter().all(|l| !l.starts_with("> Original")) {
            o.input(Input::Down, &mut s, &mut b);
        }
        o.input(Input::Accept, &mut s, &mut b);
        assert!(!s.widescreen && s.fullscreen && s.msaa == 1 && !s.interpolate && s.button_prompts == Prompts::Xbox);
    }
}
