use smithay_client_toolkit::keyboard::keysyms;

use winit::event::{
    ModifiersState,
    ScanCode,
};

fn kc_alpha(kc: ScanCode, shift: bool) -> Option<String> {
    let ret: char = match kc {
        keysyms::XKB_KEY_A | keysyms::XKB_KEY_a => 'A', 
        keysyms::XKB_KEY_B | keysyms::XKB_KEY_b => 'B',
        keysyms::XKB_KEY_C | keysyms::XKB_KEY_c => 'C',
        keysyms::XKB_KEY_D | keysyms::XKB_KEY_d => 'D',
        keysyms::XKB_KEY_E | keysyms::XKB_KEY_e => 'E',
        keysyms::XKB_KEY_F | keysyms::XKB_KEY_f => 'F',
        keysyms::XKB_KEY_G | keysyms::XKB_KEY_g => 'G',
        keysyms::XKB_KEY_H | keysyms::XKB_KEY_h => 'H',
        keysyms::XKB_KEY_I | keysyms::XKB_KEY_i => 'I',
        keysyms::XKB_KEY_J | keysyms::XKB_KEY_j => 'J',
        keysyms::XKB_KEY_K | keysyms::XKB_KEY_k => 'K',
        keysyms::XKB_KEY_L | keysyms::XKB_KEY_l => 'L',
        keysyms::XKB_KEY_M | keysyms::XKB_KEY_m => 'M',
        keysyms::XKB_KEY_N | keysyms::XKB_KEY_n => 'N',
        keysyms::XKB_KEY_O | keysyms::XKB_KEY_o => 'O',
        keysyms::XKB_KEY_P | keysyms::XKB_KEY_p => 'P',
        keysyms::XKB_KEY_Q | keysyms::XKB_KEY_q => 'Q',
        keysyms::XKB_KEY_R | keysyms::XKB_KEY_r => 'R',
        keysyms::XKB_KEY_S | keysyms::XKB_KEY_s => 'S',
        keysyms::XKB_KEY_T | keysyms::XKB_KEY_t => 'T',
        keysyms::XKB_KEY_U | keysyms::XKB_KEY_u => 'U',
        keysyms::XKB_KEY_V | keysyms::XKB_KEY_v => 'V',
        keysyms::XKB_KEY_W | keysyms::XKB_KEY_w => 'W',
        keysyms::XKB_KEY_X | keysyms::XKB_KEY_x => 'X',
        keysyms::XKB_KEY_Y | keysyms::XKB_KEY_y => 'Y',
        keysyms::XKB_KEY_Z | keysyms::XKB_KEY_z => 'Z',
        _ => return None,
    };

    if !shift {
        Some(ret.to_lowercase().to_string())
    } else {
        Some(ret.to_string())
    }
}

fn kc_numeric_symbols(modifiers: ModifiersState, kc: ScanCode) -> Option<String> {
    let alt = modifiers.alt();
    let ctrl = modifiers.ctrl();

    if alt || ctrl {
        return None;
    }

    let shift = modifiers.shift();
    
    let ret = if shift {
        match kc {
            keysyms::XKB_KEY_asciitilde => '~',
            keysyms::XKB_KEY_1 => '!',
            keysyms::XKB_KEY_2 => '@',
            keysyms::XKB_KEY_3 => '#',
            keysyms::XKB_KEY_4 => '$',
            keysyms::XKB_KEY_5 => '%',
            keysyms::XKB_KEY_6 => '^',
            keysyms::XKB_KEY_7 => '&',
            keysyms::XKB_KEY_8 => '*',
            keysyms::XKB_KEY_9 => '(',
            keysyms::XKB_KEY_0 => ')',
            keysyms::XKB_KEY_minus => '_',
            keysyms::XKB_KEY_underscore => '_',
            keysyms::XKB_KEY_equal => '+',
            keysyms::XKB_KEY_plus => '+',
            keysyms::XKB_KEY_braceleft => '{',
            keysyms::XKB_KEY_bracketleft=> '{',
            keysyms::XKB_KEY_braceright=> '}',
            keysyms::XKB_KEY_bracketright=> '}',
            keysyms::XKB_KEY_backslash => '|',
            keysyms::XKB_KEY_colon => ':',
            keysyms::XKB_KEY_semicolon => ':',
            keysyms::XKB_KEY_quotedbl => '"',
            keysyms::XKB_KEY_apostrophe => '"',
            keysyms::XKB_KEY_comma => '<',
            keysyms::XKB_KEY_less => '<',
            keysyms::XKB_KEY_period => '>',
            keysyms::XKB_KEY_greater => '>',
            keysyms::XKB_KEY_slash => '?',
            keysyms::XKB_KEY_question => '?',
            keysyms::XKB_KEY_space => ' ',
            _ => return None,
        }
    } else {
        match kc {
            keysyms::XKB_KEY_asciitilde => '`',
            keysyms::XKB_KEY_1 => '1',
            keysyms::XKB_KEY_2 => '2',
            keysyms::XKB_KEY_3 => '3',
            keysyms::XKB_KEY_4 => '4',
            keysyms::XKB_KEY_5 => '5',
            keysyms::XKB_KEY_6 => '6',
            keysyms::XKB_KEY_7 => '7',
            keysyms::XKB_KEY_8 => '8',
            keysyms::XKB_KEY_9 => '9',
            keysyms::XKB_KEY_0 => '0',
            keysyms::XKB_KEY_minus => '-',
            keysyms::XKB_KEY_underscore => '-',
            keysyms::XKB_KEY_equal => '=',
            keysyms::XKB_KEY_plus => '=',
            keysyms::XKB_KEY_braceleft => '[',
            keysyms::XKB_KEY_bracketleft=> '[',
            keysyms::XKB_KEY_braceright=> ']',
            keysyms::XKB_KEY_bracketright=> ']',
            keysyms::XKB_KEY_backslash => '\\',
            keysyms::XKB_KEY_colon => ';',
            keysyms::XKB_KEY_semicolon => ';',
            keysyms::XKB_KEY_quotedbl => '\'',
            keysyms::XKB_KEY_apostrophe => '\'',
            keysyms::XKB_KEY_comma => ',',
            keysyms::XKB_KEY_less => ',',
            keysyms::XKB_KEY_period => '.',
            keysyms::XKB_KEY_greater => '.',
            keysyms::XKB_KEY_slash => '/',
            keysyms::XKB_KEY_question => '/',
            keysyms::XKB_KEY_space => ' ',
            _ => return None,
        }
    };

    Some(ret.to_string())
}

pub fn map_input_into_string(modifiers: ModifiersState, scancode: Option<ScanCode>) -> Option<String> {
    if scancode.is_none() {
        return None;
    }
    let kc = scancode.unwrap();

    let alt = modifiers.alt();
    let ctrl = modifiers.ctrl();
    let shift = modifiers.shift();

    if alt || ctrl {
        return None
    }

    let alpha = kc_alpha(kc, shift);
    if alpha.is_some() {
        alpha
    } else {
        kc_numeric_symbols(modifiers, kc)
    } 
}
