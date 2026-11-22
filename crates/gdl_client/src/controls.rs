//! Player 1's keyboard bindings (NATIVE: the PS2 had a pad). Each action
//! presses one joyGetStatus entry of port 0 (research/player.md "Pad
//! buttons"); the defaults are the client's original keys, and
//! Settings::keys (action name → key names) replaces an action's keys.
//! The menus accept the bound keys of the directions, cross (accept),
//! triangle (back) and circle (centre) besides their fixed ones.
//!
//! NATIVE (this fork, decisions/0012): the controllers' buttons, one layout
//! for every controller (Settings::pad, action name → button names), used
//! while a level plays; menus, the select screen, the shop, the map and
//! scroll messages keep the standard buttons (the owner's choice). The
//! sticks are not rebound.
use bevy::input::gamepad::GamepadButton;
use bevy::input::keyboard::KeyCode;
use std::collections::BTreeMap;

/// One rebindable action.
pub struct Action {
    /// The settings file's name.
    pub name: &'static str,
    pub label: &'static str,
    /// The port-0 pad entry it presses.
    pub entry: usize,
    pub defaults: &'static [KeyCode],
    /// NATIVE (this fork): the controller buttons pressing it in the
    /// standard layout (gdl_play's original mapping).
    pub pad: &'static [GamepadButton],
}

/// Port 0's actions, in the overlay's order. Labels name the pad button
/// and what research has measured it doing; square and R2 are named only.
pub const ACTIONS: [Action; 18] = [
    Action { name: "up", label: "Up", entry: 2, defaults: &[KeyCode::ArrowUp, KeyCode::KeyW], pad: &[GamepadButton::DPadUp] },
    Action { name: "down", label: "Down", entry: 3, defaults: &[KeyCode::ArrowDown, KeyCode::KeyS], pad: &[GamepadButton::DPadDown] },
    Action { name: "left", label: "Left", entry: 0, defaults: &[KeyCode::ArrowLeft, KeyCode::KeyA], pad: &[GamepadButton::DPadLeft] },
    Action { name: "right", label: "Right", entry: 1, defaults: &[KeyCode::ArrowRight, KeyCode::KeyD], pad: &[GamepadButton::DPadRight] },
    Action { name: "cross", label: "Cross (attack)", entry: 0xd, defaults: &[KeyCode::Space, KeyCode::KeyK], pad: &[GamepadButton::South] },
    Action { name: "circle", label: "Circle (magic)", entry: 0xb, defaults: &[KeyCode::KeyE, KeyCode::KeyL], pad: &[GamepadButton::East] },
    Action { name: "triangle", label: "Triangle (turbo)", entry: 0xc, defaults: &[KeyCode::ShiftLeft, KeyCode::KeyI], pad: &[GamepadButton::North] },
    Action { name: "square", label: "Square", entry: 0xa, defaults: &[KeyCode::KeyQ, KeyCode::KeyJ], pad: &[GamepadButton::West] },
    Action { name: "l1", label: "L1 (dash)", entry: 8, defaults: &[KeyCode::KeyZ], pad: &[GamepadButton::LeftTrigger] },
    Action { name: "l2", label: "L2 (team move)", entry: 9, defaults: &[KeyCode::KeyX], pad: &[GamepadButton::LeftTrigger2] },
    Action { name: "r1", label: "R1 (strafe)", entry: 0x12, defaults: &[KeyCode::ControlLeft], pad: &[GamepadButton::RightTrigger] },
    Action { name: "r2", label: "R2", entry: 0x13, defaults: &[KeyCode::KeyV], pad: &[GamepadButton::RightTrigger2] },
    Action { name: "start", label: "Start", entry: 0x14, defaults: &[KeyCode::Enter], pad: &[GamepadButton::Start] },
    Action { name: "select", label: "Select", entry: 0x15, defaults: &[KeyCode::Tab], pad: &[GamepadButton::Select] },
    // The right stick (entries 0xe–0x11, full deflection): Control Style
    // One Handed moves and Robotron aims and fires with it.
    Action { name: "rstick_up", label: "Right stick up", entry: 0x10, defaults: &[KeyCode::Numpad8], pad: &[] },
    Action { name: "rstick_down", label: "Right stick down", entry: 0x11, defaults: &[KeyCode::Numpad2], pad: &[] },
    Action { name: "rstick_left", label: "Right stick left", entry: 0xe, defaults: &[KeyCode::Numpad4], pad: &[] },
    Action { name: "rstick_right", label: "Right stick right", entry: 0xf, defaults: &[KeyCode::Numpad6], pad: &[] },
];

/// NATIVE (this fork): the controller buttons that can be bound (the Mode
/// / guide button is often taken by the system; the sticks stay sticks).
pub const PAD_BINDABLE: &[GamepadButton] = &[
    GamepadButton::South,
    GamepadButton::East,
    GamepadButton::West,
    GamepadButton::North,
    GamepadButton::LeftTrigger,
    GamepadButton::LeftTrigger2,
    GamepadButton::RightTrigger,
    GamepadButton::RightTrigger2,
    GamepadButton::Select,
    GamepadButton::Start,
    GamepadButton::LeftThumb,
    GamepadButton::RightThumb,
    GamepadButton::DPadUp,
    GamepadButton::DPadDown,
    GamepadButton::DPadLeft,
    GamepadButton::DPadRight,
    GamepadButton::C,
    GamepadButton::Z,
];

/// A controller button's settings-file name (e.g. "South", "LeftTrigger2").
pub fn pad_name(b: GamepadButton) -> String {
    format!("{b:?}")
}

/// A settings-file button name, if it names a bindable button.
pub fn parse_pad(name: &str) -> Option<GamepadButton> {
    PAD_BINDABLE.iter().copied().find(|&b| pad_name(b) == name)
}

/// A controller button as the overlay shows it (Xbox / PlayStation names).
pub fn pad_label(b: GamepadButton) -> &'static str {
    match b {
        GamepadButton::South => "A / Cross",
        GamepadButton::East => "B / Circle",
        GamepadButton::West => "X / Square",
        GamepadButton::North => "Y / Triangle",
        GamepadButton::LeftTrigger => "LB / L1",
        GamepadButton::LeftTrigger2 => "LT / L2",
        GamepadButton::RightTrigger => "RB / R1",
        GamepadButton::RightTrigger2 => "RT / R2",
        GamepadButton::Select => "View / Select",
        GamepadButton::Start => "Menu / Start",
        GamepadButton::LeftThumb => "Left stick click / L3",
        GamepadButton::RightThumb => "Right stick click / R3",
        GamepadButton::DPadUp => "D-pad up",
        GamepadButton::DPadDown => "D-pad down",
        GamepadButton::DPadLeft => "D-pad left",
        GamepadButton::DPadRight => "D-pad right",
        GamepadButton::C => "C",
        GamepadButton::Z => "Z",
        _ => "other",
    }
}

/// Keys that can be bound (the client's own function keys and Escape are
/// not: F1–F3, F9–F11 and Escape keep their native uses).
pub const BINDABLE: &[KeyCode] = &[
    KeyCode::KeyA, KeyCode::KeyB, KeyCode::KeyC, KeyCode::KeyD, KeyCode::KeyE, KeyCode::KeyF, KeyCode::KeyG, KeyCode::KeyH, KeyCode::KeyI,
    KeyCode::KeyJ, KeyCode::KeyK, KeyCode::KeyL, KeyCode::KeyM, KeyCode::KeyN, KeyCode::KeyO, KeyCode::KeyP, KeyCode::KeyQ, KeyCode::KeyR,
    KeyCode::KeyS, KeyCode::KeyT, KeyCode::KeyU, KeyCode::KeyV, KeyCode::KeyW, KeyCode::KeyX, KeyCode::KeyY, KeyCode::KeyZ, KeyCode::Digit0,
    KeyCode::Digit1, KeyCode::Digit2, KeyCode::Digit3, KeyCode::Digit4, KeyCode::Digit5, KeyCode::Digit6, KeyCode::Digit7, KeyCode::Digit8,
    KeyCode::Digit9, KeyCode::ArrowUp, KeyCode::ArrowDown, KeyCode::ArrowLeft, KeyCode::ArrowRight, KeyCode::Space, KeyCode::Enter,
    KeyCode::Tab, KeyCode::Backspace, KeyCode::ShiftLeft, KeyCode::ShiftRight, KeyCode::ControlLeft, KeyCode::ControlRight, KeyCode::AltLeft,
    KeyCode::AltRight, KeyCode::Comma, KeyCode::Period, KeyCode::Slash, KeyCode::Semicolon, KeyCode::Quote, KeyCode::BracketLeft,
    KeyCode::BracketRight, KeyCode::Backslash, KeyCode::Minus, KeyCode::Equal, KeyCode::Backquote, KeyCode::Insert, KeyCode::Delete,
    KeyCode::Home, KeyCode::End, KeyCode::PageUp, KeyCode::PageDown, KeyCode::Numpad0, KeyCode::Numpad1, KeyCode::Numpad2, KeyCode::Numpad3,
    KeyCode::Numpad4, KeyCode::Numpad5, KeyCode::Numpad6, KeyCode::Numpad7, KeyCode::Numpad8, KeyCode::Numpad9, KeyCode::NumpadAdd,
    KeyCode::NumpadSubtract, KeyCode::NumpadMultiply, KeyCode::NumpadDivide, KeyCode::NumpadDecimal, KeyCode::NumpadEnter,
];

/// A key's settings-file name (its KeyCode name, e.g. "KeyA", "ArrowLeft").
pub fn key_name(k: KeyCode) -> String {
    format!("{k:?}")
}

/// A settings-file key name, if it names a bindable key.
pub fn parse_key(name: &str) -> Option<KeyCode> {
    BINDABLE.iter().copied().find(|&k| key_name(k) == name)
}

/// Each action's keys (and, NATIVE in this fork, controller buttons).
#[derive(Clone, Debug, PartialEq, bevy::prelude::Resource)]
pub struct Bindings {
    pub keys: [Vec<KeyCode>; ACTIONS.len()],
    pub pad: [Vec<GamepadButton>; ACTIONS.len()],
}

impl Default for Bindings {
    fn default() -> Self {
        Self { keys: std::array::from_fn(|i| ACTIONS[i].defaults.to_vec()), pad: std::array::from_fn(|i| ACTIONS[i].pad.to_vec()) }
    }
}

impl Bindings {
    /// The defaults with the stored actions replaced (unknown actions and
    /// key names are ignored; an action whose names all fail keeps its
    /// defaults). `pad`: Settings::pad the same way, except that an empty
    /// list is kept (an action with no button).
    pub fn from_settings(stored: &BTreeMap<String, Vec<String>>, pad: &BTreeMap<String, Vec<String>>) -> Self {
        let mut b = Self::default();
        for (i, a) in ACTIONS.iter().enumerate() {
            if let Some(names) = stored.get(a.name) {
                let keys: Vec<KeyCode> = names.iter().filter_map(|n| parse_key(n)).collect();
                if !keys.is_empty() {
                    b.keys[i] = keys;
                }
            }
            if let Some(names) = pad.get(a.name) {
                let buttons: Vec<GamepadButton> = names.iter().filter_map(|n| parse_pad(n)).collect();
                if buttons.len() == names.len() {
                    b.pad[i] = buttons;
                }
            }
        }
        // A button stored on two actions stays on the first.
        for i in 0..ACTIONS.len() {
            let (before, rest) = b.pad.split_at_mut(i);
            rest[0].retain(|x| !before.iter().flatten().any(|y| y == x));
        }
        b
    }

    /// NATIVE (this fork): the actions whose buttons differ from the
    /// standard layout, for Settings::pad.
    pub fn pad_to_settings(&self) -> BTreeMap<String, Vec<String>> {
        ACTIONS
            .iter()
            .zip(&self.pad)
            .filter(|(a, p)| p.as_slice() != a.pad)
            .map(|(a, p)| (a.name.to_string(), p.iter().map(|&b| pad_name(b)).collect()))
            .collect()
    }

    /// NATIVE (this fork): put `button` on action `i` alone. An action that
    /// had it takes action `i`'s previous buttons (a swap), so no action is
    /// left without one that had one.
    pub fn bind_pad(&mut self, i: usize, button: GamepadButton) {
        let before = std::mem::take(&mut self.pad[i]);
        for (j, p) in self.pad.iter_mut().enumerate() {
            if j != i && p.contains(&button) {
                p.retain(|&x| x != button);
                if p.is_empty() {
                    *p = before.iter().copied().filter(|&x| x != button).collect();
                }
            }
        }
        self.pad[i] = vec![button];
    }

    /// NATIVE (this fork): the standard controller layout.
    pub fn reset_pad(&mut self) {
        self.pad = Self::default().pad;
    }

    /// The actions that differ from their defaults, for the settings file.
    pub fn to_settings(&self) -> BTreeMap<String, Vec<String>> {
        ACTIONS
            .iter()
            .zip(&self.keys)
            .filter(|(a, k)| k.as_slice() != a.defaults)
            .map(|(a, k)| (a.name.to_string(), k.iter().map(|&k| key_name(k)).collect()))
            .collect()
    }

    /// The keys of the action named `name`.
    pub fn of(&self, name: &str) -> &[KeyCode] {
        ACTIONS.iter().position(|a| a.name == name).map_or(&[], |i| &self.keys[i])
    }

    /// Bind `key` alone to action `i`, taking it off any other action.
    pub fn bind(&mut self, i: usize, key: KeyCode) {
        for k in self.keys.iter_mut() {
            k.retain(|&x| x != key);
        }
        self.keys[i] = vec![key];
        // An action left without keys gets its defaults that are free.
        for (j, a) in ACTIONS.iter().enumerate() {
            if self.keys[j].is_empty() {
                let free: Vec<KeyCode> = a.defaults.iter().copied().filter(|d| !self.keys.iter().flatten().any(|x| x == d)).collect();
                self.keys[j] = free;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_round_trip_to_an_empty_map() {
        let b = Bindings::default();
        assert!(b.to_settings().is_empty());
        assert_eq!(Bindings::from_settings(&BTreeMap::new(), &BTreeMap::new()), b);
    }

    #[test]
    fn stored_names_replace_an_actions_keys() {
        let mut m = BTreeMap::new();
        m.insert("cross".to_string(), vec!["KeyN".to_string(), "Nonsense".to_string()]);
        m.insert("nothing".to_string(), vec!["KeyM".to_string()]);
        let b = Bindings::from_settings(&m, &BTreeMap::new());
        assert_eq!(b.of("cross"), &[KeyCode::KeyN]);
        assert_eq!(b.to_settings(), BTreeMap::from([("cross".to_string(), vec!["KeyN".to_string()])]));
    }

    #[test]
    fn binding_a_key_moves_it() {
        let mut b = Bindings::default();
        let cross = ACTIONS.iter().position(|a| a.name == "cross").unwrap();
        // W was Up's second key.
        b.bind(cross, KeyCode::KeyW);
        assert_eq!(b.of("cross"), &[KeyCode::KeyW]);
        assert_eq!(b.of("up"), &[KeyCode::ArrowUp]);
    }

    /// NATIVE (this fork, decisions/0012): controller buttons.
    #[test]
    fn controller_buttons_swap_and_round_trip() {
        let at = |n: &str| ACTIONS.iter().position(|a| a.name == n).unwrap();
        let mut b = Bindings::default();
        assert!(b.pad_to_settings().is_empty());
        // Attack onto RB: strafe takes attack's A.
        b.bind_pad(at("cross"), GamepadButton::RightTrigger);
        assert_eq!(b.pad[at("cross")], [GamepadButton::RightTrigger]);
        assert_eq!(b.pad[at("r1")], [GamepadButton::South]);
        // A free button: nothing else changes.
        b.bind_pad(at("rstick_up"), GamepadButton::RightThumb);
        assert_eq!(b.pad[at("circle")], [GamepadButton::East]);
        let stored = b.pad_to_settings();
        assert_eq!(stored.len(), 3);
        assert_eq!(stored["cross"], ["RightTrigger"]);
        assert_eq!(Bindings::from_settings(&BTreeMap::new(), &stored), b);
        // Unknown names leave the action as standard; a button stored twice
        // stays on the first action.
        let odd = BTreeMap::from([("circle".to_string(), vec!["Nonsense".to_string()]), ("up".to_string(), vec!["South".to_string()])]);
        let o = Bindings::from_settings(&BTreeMap::new(), &odd);
        assert_eq!(o.pad[at("circle")], [GamepadButton::East]);
        assert_eq!((o.pad[at("up")].as_slice(), o.pad[at("cross")].as_slice()), (&[GamepadButton::South][..], &[][..]));
        b.reset_pad();
        assert_eq!(b, Bindings::default());
        for a in &ACTIONS {
            for &p in a.pad {
                assert_eq!(parse_pad(&pad_name(p)), Some(p), "{}", a.name);
            }
        }
    }

    #[test]
    fn every_default_key_is_bindable_and_named() {
        for a in &ACTIONS {
            for &k in a.defaults {
                assert_eq!(parse_key(&key_name(k)), Some(k), "{}", a.name);
            }
        }
    }
}
