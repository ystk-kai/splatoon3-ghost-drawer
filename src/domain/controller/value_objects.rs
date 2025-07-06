use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Button {
    value: u16,
}

impl Button {
    pub const Y: Button = Button { value: 0x0001 };
    pub const B: Button = Button { value: 0x0002 };
    pub const A: Button = Button { value: 0x0004 };
    pub const X: Button = Button { value: 0x0008 };
    pub const L: Button = Button { value: 0x0010 };
    pub const R: Button = Button { value: 0x0020 };
    pub const ZL: Button = Button { value: 0x0040 };
    pub const ZR: Button = Button { value: 0x0080 };
    pub const MINUS: Button = Button { value: 0x0100 };
    pub const PLUS: Button = Button { value: 0x0200 };
    pub const L_STICK: Button = Button { value: 0x0400 };
    pub const R_STICK: Button = Button { value: 0x0800 };
    pub const HOME: Button = Button { value: 0x1000 };
    pub const CAPTURE: Button = Button { value: 0x2000 };

    pub fn new(value: u16) -> Self {
        Self { value }
    }

    pub fn value(&self) -> u16 {
        self.value
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DPad {
    value: u8,
}

impl DPad {
    pub const NEUTRAL: DPad = DPad { value: 0x08 };
    pub const UP: DPad = DPad { value: 0x00 };
    pub const UP_RIGHT: DPad = DPad { value: 0x01 };
    pub const RIGHT: DPad = DPad { value: 0x02 };
    pub const DOWN_RIGHT: DPad = DPad { value: 0x03 };
    pub const DOWN: DPad = DPad { value: 0x04 };
    pub const DOWN_LEFT: DPad = DPad { value: 0x05 };
    pub const LEFT: DPad = DPad { value: 0x06 };
    pub const UP_LEFT: DPad = DPad { value: 0x07 };

    pub fn new(value: u8) -> Self {
        Self { value }
    }

    pub fn value(&self) -> u8 {
        self.value
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StickPosition {
    pub x: u8,
    pub y: u8,
}

impl StickPosition {
    pub const CENTER: StickPosition = StickPosition { x: 128, y: 128 };
    pub const MIN: u8 = 0;
    pub const MAX: u8 = 255;

    pub fn new(x: u8, y: u8) -> Self {
        Self { x, y }
    }

    pub fn from_normalized(x: f32, y: f32) -> Self {
        let x = ((x.clamp(-1.0, 1.0) + 1.0) * 127.5) as u8;
        let y = ((y.clamp(-1.0, 1.0) + 1.0) * 127.5) as u8;
        Self { x, y }
    }

    pub fn to_normalized(&self) -> (f32, f32) {
        let x = (self.x as f32 / 127.5) - 1.0;
        let y = (self.y as f32 / 127.5) - 1.0;
        (x, y)
    }

    pub fn is_centered(&self) -> bool {
        *self == Self::CENTER
    }
}

impl Default for StickPosition {
    fn default() -> Self {
        Self::CENTER
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ButtonState {
    pressed: u16,
}

impl ButtonState {
    pub fn new() -> Self {
        Self { pressed: 0 }
    }

    pub fn press(&mut self, button: Button) {
        self.pressed |= button.value();
    }

    pub fn release(&mut self, button: Button) {
        self.pressed &= !button.value();
    }

    pub fn is_pressed(&self, button: Button) -> bool {
        (self.pressed & button.value()) != 0
    }

    pub fn pressed_buttons(&self) -> Vec<Button> {
        let mut buttons = Vec::new();
        if self.is_pressed(Button::Y) { buttons.push(Button::Y); }
        if self.is_pressed(Button::B) { buttons.push(Button::B); }
        if self.is_pressed(Button::A) { buttons.push(Button::A); }
        if self.is_pressed(Button::X) { buttons.push(Button::X); }
        if self.is_pressed(Button::L) { buttons.push(Button::L); }
        if self.is_pressed(Button::R) { buttons.push(Button::R); }
        if self.is_pressed(Button::ZL) { buttons.push(Button::ZL); }
        if self.is_pressed(Button::ZR) { buttons.push(Button::ZR); }
        if self.is_pressed(Button::MINUS) { buttons.push(Button::MINUS); }
        if self.is_pressed(Button::PLUS) { buttons.push(Button::PLUS); }
        if self.is_pressed(Button::L_STICK) { buttons.push(Button::L_STICK); }
        if self.is_pressed(Button::R_STICK) { buttons.push(Button::R_STICK); }
        if self.is_pressed(Button::HOME) { buttons.push(Button::HOME); }
        if self.is_pressed(Button::CAPTURE) { buttons.push(Button::CAPTURE); }
        buttons
    }

    pub fn raw_value(&self) -> u16 {
        self.pressed
    }
}

impl Default for ButtonState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct HidReport {
    pub buttons: ButtonState,
    pub dpad: DPad,
    pub left_stick: StickPosition,
    pub right_stick: StickPosition,
}

impl HidReport {
    pub fn new() -> Self {
        Self {
            buttons: ButtonState::new(),
            dpad: DPad::NEUTRAL,
            left_stick: StickPosition::CENTER,
            right_stick: StickPosition::CENTER,
        }
    }

    pub fn to_bytes(&self) -> [u8; 8] {
        let button_bytes = self.buttons.raw_value().to_le_bytes();
        [
            button_bytes[0],
            button_bytes[1],
            self.dpad.value(),
            self.left_stick.x,
            self.left_stick.y,
            self.right_stick.x,
            self.right_stick.y,
            0x00, // Padding
        ]
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() < 8 {
            return Err("Invalid HID report size".to_string());
        }

        let buttons = ButtonState {
            pressed: u16::from_le_bytes([bytes[0], bytes[1]]),
        };

        Ok(Self {
            buttons,
            dpad: DPad::new(bytes[2]),
            left_stick: StickPosition::new(bytes[3], bytes[4]),
            right_stick: StickPosition::new(bytes[5], bytes[6]),
        })
    }
}

impl Default for HidReport {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ControllerType {
    ProController,
    JoyConLeft,
    JoyConRight,
    JoyConPair,
}

impl ControllerType {
    pub fn product_id(&self) -> u16 {
        match self {
            ControllerType::ProController => 0x2009,
            ControllerType::JoyConLeft => 0x2006,
            ControllerType::JoyConRight => 0x2007,
            ControllerType::JoyConPair => 0x2008,
        }
    }

    pub fn product_name(&self) -> &'static str {
        match self {
            ControllerType::ProController => "Pro Controller",
            ControllerType::JoyConLeft => "Joy-Con (L)",
            ControllerType::JoyConRight => "Joy-Con (R)",
            ControllerType::JoyConPair => "Joy-Con L/R",
        }
    }
}

impl fmt::Display for ControllerType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.product_name())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ControllerCommand {
    pub sequence: Vec<ControllerAction>,
    pub name: String,
    pub description: Option<String>,
}

impl ControllerCommand {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            sequence: Vec::new(),
            name: name.into(),
            description: None,
        }
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn add_action(mut self, action: ControllerAction) -> Self {
        self.sequence.push(action);
        self
    }

    pub fn total_duration_ms(&self) -> u32 {
        self.sequence.iter().map(|a| a.duration_ms).sum()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ControllerAction {
    pub action_type: ActionType,
    pub duration_ms: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ActionType {
    PressButton(Button),
    ReleaseButton(Button),
    SetDPad(DPad),
    MoveLeftStick(StickPosition),
    MoveRightStick(StickPosition),
    SetReport(HidReport),
    Wait,
}

impl ControllerAction {
    pub fn press_button(button: Button, duration_ms: u32) -> Self {
        Self {
            action_type: ActionType::PressButton(button),
            duration_ms,
        }
    }

    pub fn release_button(button: Button, duration_ms: u32) -> Self {
        Self {
            action_type: ActionType::ReleaseButton(button),
            duration_ms,
        }
    }

    pub fn set_dpad(dpad: DPad, duration_ms: u32) -> Self {
        Self {
            action_type: ActionType::SetDPad(dpad),
            duration_ms,
        }
    }

    pub fn move_left_stick(position: StickPosition, duration_ms: u32) -> Self {
        Self {
            action_type: ActionType::MoveLeftStick(position),
            duration_ms,
        }
    }

    pub fn move_right_stick(position: StickPosition, duration_ms: u32) -> Self {
        Self {
            action_type: ActionType::MoveRightStick(position),
            duration_ms,
        }
    }

    pub fn wait(duration_ms: u32) -> Self {
        Self {
            action_type: ActionType::Wait,
            duration_ms,
        }
    }
}