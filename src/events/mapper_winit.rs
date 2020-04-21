use winit::event::{
    ModifiersState,
    KeyboardInput,
    VirtualKeyCode,
};

fn kc_alpha(kc: VirtualKeyCode, shift: bool) -> Option<String> {
    let mut ret: char = match kc {
        VirtualKeyCode::A => 'A', 
        VirtualKeyCode::B => 'B',
        VirtualKeyCode::C => 'C',
        VirtualKeyCode::D => 'D',
        VirtualKeyCode::E => 'E',
        VirtualKeyCode::F => 'F',
        VirtualKeyCode::G => 'G',
        VirtualKeyCode::H => 'H',
        VirtualKeyCode::I => 'I',
        VirtualKeyCode::J => 'J',
        VirtualKeyCode::K => 'K',
        VirtualKeyCode::L => 'L',
        VirtualKeyCode::M => 'M',
        VirtualKeyCode::N => 'N',
        VirtualKeyCode::O => 'O',
        VirtualKeyCode::P => 'P',
        VirtualKeyCode::Q => 'Q',
        VirtualKeyCode::R => 'R',
        VirtualKeyCode::S => 'S',
        VirtualKeyCode::T => 'T',
        VirtualKeyCode::U => 'U',
        VirtualKeyCode::V => 'V',
        VirtualKeyCode::W => 'W',
        VirtualKeyCode::X => 'X',
        VirtualKeyCode::Y => 'Y',
        VirtualKeyCode::Z => 'Z',
        _ => return None,
    };

    if !shift {
        Some(ret.to_lowercase().to_string())
    } else {
        Some(ret.to_string())
    }
}

fn kc_numeric_symbols(modifiers: ModifiersState, kc: VirtualKeyCode) -> Option<String> {
    let alt = modifiers.alt();
    let ctrl = modifiers.ctrl();

    if alt || ctrl {
        return None;
    }

    let shift = modifiers.shift();
    
    let ret = if shift {
        match kc {
            VirtualKeyCode::Grave => '~',
            VirtualKeyCode::Key1 => '!',
            VirtualKeyCode::Key2 => '@',
            VirtualKeyCode::Key3 => '#',
            VirtualKeyCode::Key4 => '$',
            VirtualKeyCode::Key5 => '%',
            VirtualKeyCode::Key6 => '^',
            VirtualKeyCode::Key7 => '&',
            VirtualKeyCode::Key8 => '*',
            VirtualKeyCode::Key9 => '(',
            VirtualKeyCode::Key0 => ')',
            VirtualKeyCode::Minus => '_',
            VirtualKeyCode::Equals => '+',
            VirtualKeyCode::LBracket => '{',
            VirtualKeyCode::RBracket => '}',
            VirtualKeyCode::Backslash => '|',
            VirtualKeyCode::Colon => ':',
            VirtualKeyCode::Apostrophe => '"',
            VirtualKeyCode::Comma => '<',
            VirtualKeyCode::Period => '>',
            VirtualKeyCode::Slash => '?',
            _ => return None,
        }
    } else {
        match kc {
            VirtualKeyCode::Grave => '`',
            VirtualKeyCode::Key1 => '1',
            VirtualKeyCode::Key2 => '2',
            VirtualKeyCode::Key3 => '3',
            VirtualKeyCode::Key4 => '4',
            VirtualKeyCode::Key5 => '5',
            VirtualKeyCode::Key6 => '6',
            VirtualKeyCode::Key7 => '7',
            VirtualKeyCode::Key8 => '8',
            VirtualKeyCode::Key9 => '9',
            VirtualKeyCode::Key0 => '0',
            VirtualKeyCode::Minus => '-',
            VirtualKeyCode::Equals => '=',
            VirtualKeyCode::LBracket => '[',
            VirtualKeyCode::RBracket => ']',
            VirtualKeyCode::Backslash => '\\',
            VirtualKeyCode::Colon => ';',
            VirtualKeyCode::Apostrophe => '\'',
            VirtualKeyCode::Comma => ',',
            VirtualKeyCode::Period => '.',
            VirtualKeyCode::Slash => '/',
            _ => return None,
        }
    };

    Some(ret.to_string())
}

pub fn key_into_string(modifiers: ModifiersState, input: KeyboardInput) -> Option<String> {
    if input.virtual_keycode.is_none() {
        return None;
    }

    let alt = modifiers.alt();
    let ctrl = modifiers.ctrl();
    let shift = modifiers.shift();

    if alt || ctrl {
        return None
    }

    kc_alpha(input.virtual_keycode.unwrap(), shift)
}
