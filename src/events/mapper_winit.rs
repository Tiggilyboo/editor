use winit::event::{
    VirtualKeyCode,
    ScanCode,
};

pub fn map_scancode(scancode: ScanCode) -> Option<VirtualKeyCode> {
    match scancode {
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
            println!("unable to handle scancode: {:x}", scancode);
            None
        }
    }
}
