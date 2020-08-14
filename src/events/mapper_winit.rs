use winit::event::{
    VirtualKeyCode,
    ScanCode,
};

// Due to some missing scancodes in winit, we map them here
#[inline]
pub fn map_scancode(scancode: ScanCode) -> Option<VirtualKeyCode> {
    match scancode {
        0x02 => Some(VirtualKeyCode::Key1),
        0x03 => Some(VirtualKeyCode::Key2),
        0x04 => Some(VirtualKeyCode::Key3),
        0x05 => Some(VirtualKeyCode::Key4),
        0x06 => Some(VirtualKeyCode::Key5),
        0x07 => Some(VirtualKeyCode::Key6),
        0x08 => Some(VirtualKeyCode::Key7),
        0x09 => Some(VirtualKeyCode::Key8),
        0x0a => Some(VirtualKeyCode::Key9),
        0x0b => Some(VirtualKeyCode::Key0),
        0x0c => Some(VirtualKeyCode::Minus),
        0x28 => Some(VirtualKeyCode::Apostrophe),
        0x29 => Some(VirtualKeyCode::Grave),
        0x33 => Some(VirtualKeyCode::Comma),
        0x34 => Some(VirtualKeyCode::Period),
        0x35 => Some(VirtualKeyCode::Slash),
        0x2b => Some(VirtualKeyCode::Backslash),
        0x7e => Some(VirtualKeyCode::Grave),
        0x1a => Some(VirtualKeyCode::LBracket),
        0x1b => Some(VirtualKeyCode::RBracket),
        _ => {
            None
        }
    }
}
