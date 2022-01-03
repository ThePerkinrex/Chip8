use crossbeam_channel::{bounded, Receiver, Sender};
use winit::event::VirtualKeyCode;

const fn keymap(key: VirtualKeyCode) -> Option<u8> {
    use VirtualKeyCode::*;
    match key {
        Key1 => Some(0x1), // 1
        Key2 => Some(0x2), // 2
        Key3 => Some(0x3), // 3
        Key4 => Some(0xc), // 4
        Q => Some(0x4),    // Q
        W => Some(0x5),    // W
        E => Some(0x6),    // E
        R => Some(0xD),    // R
        A => Some(0x7),    // A
        S => Some(0x8),    // S
        D => Some(0x9),    // D
        F => Some(0xE),    // F
        Z => Some(0xA),    // Z
        X => Some(0x0),    // X
        C => Some(0xB),    // C
        V => Some(0xF),    // V
        _ => None,
    }
}

#[derive(Default)]
pub struct Keyboard {
    keys_pressed: u16,
    on_next_keypress: Option<Sender<u8>>,
}

impl Keyboard {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_pressed(&self, key: u8) -> bool {
        self.keys_pressed & (1 << key) != 0
    }

    pub fn key_down(&mut self, key_code: VirtualKeyCode) -> bool {
        keymap(key_code)
            .map(|key| {
                self.keys_pressed |= 1 << key;
                if let Some(sender) = self.on_next_keypress.take() {
                    sender.send(key).unwrap()
                }
            })
            .is_some()
    }

    pub fn key_up(&mut self, key_code: VirtualKeyCode) -> bool {
        keymap(key_code)
            .map(|key| {
                self.keys_pressed &= 0xffff ^ (1 << key);
            })
            .is_some()
    }

    pub fn set_callback(&mut self) -> Receiver<u8> {
        let (tx, rx) = bounded(1);
        self.on_next_keypress = Some(tx);
        rx
    }
}
