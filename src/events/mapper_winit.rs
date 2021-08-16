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

pub fn map_char(ch: char) -> Option<VirtualKeyCode> {
    // This is crap
    let ch: &str = &String::from(ch).to_uppercase();

    match ch {
        "A" => Some(VirtualKeyCode::A),
        "B" => Some(VirtualKeyCode::B),
        "C" => Some(VirtualKeyCode::C),
        "D" => Some(VirtualKeyCode::D),
        "E" => Some(VirtualKeyCode::E),
        "F" => Some(VirtualKeyCode::F),
        "G" => Some(VirtualKeyCode::G),
        "H" => Some(VirtualKeyCode::H),
        "I" => Some(VirtualKeyCode::I),
        "J" => Some(VirtualKeyCode::J),
        "K" => Some(VirtualKeyCode::K),
        "L" => Some(VirtualKeyCode::L),
        "M" => Some(VirtualKeyCode::M),
        "N" => Some(VirtualKeyCode::N),
        "O" => Some(VirtualKeyCode::O),
        "P" => Some(VirtualKeyCode::P),
        "Q" => Some(VirtualKeyCode::Q),
        "R" => Some(VirtualKeyCode::R),
        "S" => Some(VirtualKeyCode::S),
        "T" => Some(VirtualKeyCode::T),
        "U" => Some(VirtualKeyCode::U),
        "V" => Some(VirtualKeyCode::V),
        "W" => Some(VirtualKeyCode::W),
        "X" => Some(VirtualKeyCode::X),
        "Y" => Some(VirtualKeyCode::Y),
        "Z" => Some(VirtualKeyCode::Z),
        "0" => Some(VirtualKeyCode::Key0),
        "1" => Some(VirtualKeyCode::Key1),
        "2" => Some(VirtualKeyCode::Key2),
        "3" => Some(VirtualKeyCode::Key3),
        "4" => Some(VirtualKeyCode::Key4),
        "5" => Some(VirtualKeyCode::Key5),
        "6" => Some(VirtualKeyCode::Key6),
        "7" => Some(VirtualKeyCode::Key7),
        "8" => Some(VirtualKeyCode::Key8),
        "9" => Some(VirtualKeyCode::Key9),
        _ => None,
    }
}
