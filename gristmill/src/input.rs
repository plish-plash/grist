use crate::{
    asset::{self, AssetError},
    math::Vec2,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub use miniquad::KeyCode;

#[derive(Default, Clone, Copy, PartialEq, Debug)]
pub enum InputState {
    #[default]
    None,
    Button(bool),
    Axis1(f32),
    Axis2(Vec2),
}

impl InputState {
    fn as_button(self) -> bool {
        match self {
            InputState::None => false,
            InputState::Button(b) => b,
            InputState::Axis1(v) => v.abs() >= 0.5,
            InputState::Axis2(_) => {
                eprintln!("Axis2 input can't be used for Button action");
                false
            }
        }
    }
    fn as_axis1(self) -> f32 {
        match self {
            InputState::None => 0.0,
            InputState::Button(b) => {
                if b {
                    1.0
                } else {
                    0.0
                }
            }
            InputState::Axis1(v) => v,
            InputState::Axis2(_) => {
                eprintln!("Axis2 input can't be used for Axis1 action");
                0.0
            }
        }
    }
    fn as_axis2(self) -> Vec2 {
        match self {
            InputState::None => Vec2::ZERO,
            InputState::Button(_) => {
                eprintln!("Button input can't be used for Axis2 action");
                Vec2::ZERO
            }
            InputState::Axis1(v) => Vec2 { x: v, y: 0.0 },
            InputState::Axis2(v) => v,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
}

impl TryFrom<miniquad::MouseButton> for MouseButton {
    type Error = ();
    fn try_from(value: miniquad::MouseButton) -> Result<Self, Self::Error> {
        match value {
            miniquad::MouseButton::Left => Ok(MouseButton::Left),
            miniquad::MouseButton::Middle => Ok(MouseButton::Middle),
            miniquad::MouseButton::Right => Ok(MouseButton::Right),
            miniquad::MouseButton::Unknown => Err(()),
        }
    }
}

pub enum InputEvent {
    Key { key: KeyCode, pressed: bool },
    MouseMotion { position: Vec2 },
    RawMouseMotion { delta: Vec2 },
    MouseButton { button: MouseButton, pressed: bool },
}

#[derive(Serialize, Deserialize)]
#[serde(remote = "KeyCode")]
enum KeyCodeRemote {
    Space,
    Apostrophe,
    Comma,
    Minus,
    Period,
    Slash,
    Key0,
    Key1,
    Key2,
    Key3,
    Key4,
    Key5,
    Key6,
    Key7,
    Key8,
    Key9,
    Semicolon,
    Equal,
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    LeftBracket,
    Backslash,
    RightBracket,
    GraveAccent,
    World1,
    World2,
    Escape,
    Enter,
    Tab,
    Backspace,
    Insert,
    Delete,
    Right,
    Left,
    Down,
    Up,
    PageUp,
    PageDown,
    Home,
    End,
    CapsLock,
    ScrollLock,
    NumLock,
    PrintScreen,
    Pause,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    F13,
    F14,
    F15,
    F16,
    F17,
    F18,
    F19,
    F20,
    F21,
    F22,
    F23,
    F24,
    F25,
    Kp0,
    Kp1,
    Kp2,
    Kp3,
    Kp4,
    Kp5,
    Kp6,
    Kp7,
    Kp8,
    Kp9,
    KpDecimal,
    KpDivide,
    KpMultiply,
    KpSubtract,
    KpAdd,
    KpEnter,
    KpEqual,
    LeftShift,
    LeftControl,
    LeftAlt,
    LeftSuper,
    RightShift,
    RightControl,
    RightAlt,
    RightSuper,
    Menu,
    Unknown,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct KeyBinding {
    #[serde(with = "KeyCodeRemote")]
    key: KeyCode,
    #[serde(skip)]
    pressed: bool,
}

impl KeyBinding {
    pub fn new(key: KeyCode) -> Self {
        KeyBinding {
            key,
            pressed: false,
        }
    }

    fn event(&mut self, event: &InputEvent) -> Option<InputState> {
        if let InputEvent::Key { key, pressed } = *event {
            if key == self.key {
                self.pressed = pressed;
                return Some(InputState::Button(self.pressed));
            }
        }
        None
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct KeyAxis1Binding {
    up: KeyBinding,
    down: KeyBinding,
}

impl KeyAxis1Binding {
    pub fn new(up: KeyCode, down: KeyCode) -> Self {
        KeyAxis1Binding {
            up: KeyBinding::new(up),
            down: KeyBinding::new(down),
        }
    }

    fn event(&mut self, event: &InputEvent) -> Option<InputState> {
        let mut changed = false;
        changed |= self.up.event(event).is_some();
        changed |= self.down.event(event).is_some();
        if changed {
            Some(self.state())
        } else {
            None
        }
    }
    fn state(&self) -> InputState {
        let mut x = 0.;
        if self.up.pressed {
            x += 1.0;
        }
        if self.down.pressed {
            x -= 1.0;
        }
        InputState::Axis1(x)
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct KeyAxis2Binding {
    up: KeyBinding,
    down: KeyBinding,
    left: KeyBinding,
    right: KeyBinding,
}

impl KeyAxis2Binding {
    pub fn new(up: KeyCode, down: KeyCode, left: KeyCode, right: KeyCode) -> Self {
        KeyAxis2Binding {
            up: KeyBinding::new(up),
            down: KeyBinding::new(down),
            left: KeyBinding::new(left),
            right: KeyBinding::new(right),
        }
    }

    fn event(&mut self, event: &InputEvent) -> Option<InputState> {
        let mut changed = false;
        changed |= self.up.event(event).is_some();
        changed |= self.down.event(event).is_some();
        changed |= self.left.event(event).is_some();
        changed |= self.right.event(event).is_some();
        if changed {
            Some(self.state())
        } else {
            None
        }
    }
    fn state(&self) -> InputState {
        let mut x = 0.;
        let mut y = 0.;
        if self.up.pressed {
            y += 1.0;
        }
        if self.down.pressed {
            y -= 1.0;
        }
        if self.left.pressed {
            x -= 1.0;
        }
        if self.right.pressed {
            x += 1.0;
        }
        InputState::Axis2(Vec2 { x, y })
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct MouseButtonBinding {
    button: MouseButton,
    #[serde(skip)]
    pressed: bool,
}

impl MouseButtonBinding {
    pub fn new(button: MouseButton) -> Self {
        MouseButtonBinding {
            button,
            pressed: false,
        }
    }

    fn event(&mut self, event: &InputEvent) -> Option<InputState> {
        if let InputEvent::MouseButton { button, pressed } = *event {
            if button == self.button {
                self.pressed = pressed;
                return Some(InputState::Button(self.pressed));
            }
        }
        None
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct MouseMotionBinding {
    sensitivity: f32,
    #[serde(skip)]
    motion: Vec2,
}

impl MouseMotionBinding {
    pub fn new(sensitivity: f32) -> Self {
        MouseMotionBinding {
            sensitivity,
            motion: Vec2::ZERO,
        }
    }

    fn event(&mut self, event: &InputEvent) -> Option<InputState> {
        if let InputEvent::RawMouseMotion { delta } = *event {
            self.motion += delta * self.sensitivity;
            Some(InputState::Axis2(self.motion))
        } else {
            None
        }
    }
    fn end_frame(&mut self) -> InputState {
        self.motion = Vec2::ZERO;
        InputState::Axis2(self.motion)
    }
}

#[derive(Clone, Serialize, Deserialize)]
enum Binding {
    Key(KeyBinding),
    KeyAxis1(KeyAxis1Binding),
    KeyAxis2(KeyAxis2Binding),
    MouseButton(MouseButtonBinding),
    MouseMotion(MouseMotionBinding),
}

impl Binding {
    fn event(&mut self, event: &InputEvent) -> Option<InputState> {
        match self {
            Binding::Key(binding) => binding.event(event),
            Binding::KeyAxis1(binding) => binding.event(event),
            Binding::KeyAxis2(binding) => binding.event(event),
            Binding::MouseButton(binding) => binding.event(event),
            Binding::MouseMotion(binding) => binding.event(event),
        }
    }
}

#[derive(Default, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct InputBindings(HashMap<String, Binding>);

impl InputBindings {
    pub fn new() -> Self {
        InputBindings(HashMap::new())
    }

    const FILENAME: &'static str = "controls.yaml";
    pub fn load_config() -> Result<InputBindings, AssetError> {
        asset::load_yaml_file("config", Self::FILENAME)
    }
    pub fn save_config(&self) -> Result<(), AssetError> {
        asset::save_yaml_file("config", Self::FILENAME, self)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn add_key(&mut self, key: &str, binding: KeyBinding) {
        self.0.insert(key.to_owned(), Binding::Key(binding));
    }
    pub fn add_key_axis1(&mut self, key: &str, binding: KeyAxis1Binding) {
        self.0.insert(key.to_owned(), Binding::KeyAxis1(binding));
    }
    pub fn add_key_axis2(&mut self, key: &str, binding: KeyAxis2Binding) {
        self.0.insert(key.to_owned(), Binding::KeyAxis2(binding));
    }
    pub fn add_mouse_button(&mut self, key: &str, binding: MouseButtonBinding) {
        self.0.insert(key.to_owned(), Binding::MouseButton(binding));
    }
    pub fn add_mouse_motion(&mut self, key: &str, binding: MouseMotionBinding) {
        self.0.insert(key.to_owned(), Binding::MouseMotion(binding));
    }
}

#[derive(Default, Clone)]
pub struct ActionState {
    changed: bool,
    state: InputState,
}

impl ActionState {
    pub fn changed(&self) -> bool {
        self.changed
    }
    pub fn button_state(&self) -> bool {
        self.state.as_button()
    }
    pub fn axis1_state(&self) -> f32 {
        self.state.as_axis1()
    }
    pub fn axis2_state(&self) -> Vec2 {
        self.state.as_axis2()
    }

    pub fn pressed(&self) -> bool {
        self.button_state()
    }
    pub fn released(&self) -> bool {
        !self.button_state()
    }
    pub fn just_pressed(&self) -> bool {
        self.pressed() && self.changed
    }
    pub fn just_released(&self) -> bool {
        self.released() && self.changed
    }
}

#[derive(Default)]
pub struct PointerState {
    pub position: Vec2,
    pub primary: bool,
    pub secondary: bool,
}

pub struct InputSystem {
    bindings: HashMap<String, (Binding, ActionState)>,
    pointer: PointerState,
}

impl InputSystem {
    pub fn new(bindings: InputBindings) -> Self {
        let bindings = bindings
            .0
            .into_iter()
            .map(|(key, binding)| (key, (binding, ActionState::default())))
            .collect();
        InputSystem {
            bindings,
            pointer: Default::default(),
        }
    }
    pub fn create_default_config_if_missing() -> asset::Result<()> {
        let path = asset::get_path("config", InputBindings::FILENAME);
        if path.exists() {
            return Ok(());
        }
        println!("Creating default file {}", path.to_string_lossy());
        let mut bindings = InputBindings::default();
        bindings.add_mouse_button("primary", MouseButtonBinding::new(MouseButton::Left));
        bindings.add_mouse_button("secondary", MouseButtonBinding::new(MouseButton::Right));
        bindings.add_mouse_motion("look", MouseMotionBinding::new(0.01));
        bindings.add_key("exit", KeyBinding::new(KeyCode::Escape));
        bindings.add_key_axis2(
            "move",
            KeyAxis2Binding::new(KeyCode::W, KeyCode::S, KeyCode::A, KeyCode::D),
        );
        bindings.add_key("jump", KeyBinding::new(KeyCode::Space));
        bindings.add_key_axis1(
            "fly",
            KeyAxis1Binding::new(KeyCode::Space, KeyCode::LeftShift),
        );
        bindings.save_config()
    }
    pub fn load_config() -> asset::Result<Self> {
        Ok(Self::new(InputBindings::load_config()?))
    }

    pub fn try_get(&self, key: &str) -> Option<&ActionState> {
        self.bindings.get(key).map(|(_, action)| action)
    }
    pub fn get(&self, key: &str) -> ActionState {
        if let Some(state) = self.try_get(key) {
            state.clone()
        } else {
            eprintln!("Input action \"{}\" not bound", key);
            ActionState::default()
        }
    }
    pub fn pointer(&self) -> &PointerState {
        &self.pointer
    }

    pub fn end_frame(&mut self) {
        // MouseMotionBindings work differently than others. The values are accumulated over each frame, then reset.
        for (binding, action) in self.bindings.values_mut() {
            action.changed = false;
            if let Binding::MouseMotion(binding) = binding {
                action.state = binding.end_frame();
            }
        }
    }

    pub fn handle_event(&mut self, event: InputEvent) {
        if let InputEvent::MouseMotion { position } = event {
            self.pointer.position = position;
            return;
        }

        for (key, (binding, action)) in self.bindings.iter_mut() {
            if let Some(state) = binding.event(&event) {
                if action.state != state {
                    action.state = state;
                    action.changed = true;
                }
                if key == "primary" {
                    self.pointer.primary = state.as_button();
                } else if key == "secondary" {
                    self.pointer.secondary = state.as_button();
                }
            }
        }
    }
}
