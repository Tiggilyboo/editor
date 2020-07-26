use winit::event::{
    VirtualKeyCode,
    ScanCode,
};

pub fn map_scancode(scancode: ScanCode) -> Option<VirtualKeyCode> {
    match scancode {
        0x12 => Some(VirtualKeyCode::Underline),
        0x20 => Some(VirtualKeyCode::Apostrophe),
        0x29 => Some(VirtualKeyCode::Grave),
        0x33 => Some(VirtualKeyCode::Comma),
        0x35 => Some(VirtualKeyCode::Slash),
        0x34 => Some(VirtualKeyCode::Period),
        0x40 => Some(VirtualKeyCode::Apostrophe),
        0x43 => Some(VirtualKeyCode::Backslash),
        0x7e => Some(VirtualKeyCode::Grave),
        0x1a => Some(VirtualKeyCode::LBracket),
        0x1b => Some(VirtualKeyCode::RBracket),
        0x0c => Some(VirtualKeyCode::Underline),
        _ => {
            println!("unable to handle scancode: {:x}", scancode);
            None
        }
    }
}
