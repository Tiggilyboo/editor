use eddy::{
    Mode,
    Action,
    Motion,
    Quantity,
};
use crate::events::mapper_winit::map_char;

use winit::event::{
    ModifiersState,
    ScanCode,
    VirtualKeyCode,
    VirtualKeyCode::*,
    MouseButton,
};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum Key {
    ScanCode(ScanCode),
    KeyCode(VirtualKeyCode),
}


#[derive(Debug, Clone, PartialEq)]
pub struct Binding<T> {
    mods: ModifiersState,
    mode: Mode,
    notmode: Mode,
    trigger: T,
    pub actions: Vec<Action>,
}

pub type KeyBinding = Binding<Key>;
pub type MouseBinding = Binding<MouseButton>;

impl<T: Eq> Binding<T> {
    #[inline]
    pub fn is_triggered_by(&self, mode: Mode, mods: ModifiersState, input: &T) -> bool {
        self.trigger == *input
            && self.mods == mods
            && (self.mode == Mode::None || self.mode == mode)
            && (self.notmode == Mode::None || self.notmode != mode)
    }
}


macro_rules! shift { () => {{ ModifiersState::SHIFT }}; }
macro_rules! ctrl { () => {{ ModifiersState::CTRL }}; }
macro_rules! alt { () => {{ ModifiersState::ALT }}; }

macro_rules! bindings {
    (KeyBinding;
        $(
            $key:ident
            $(,$mods:expr)?
            $(,+$mode:expr)?
            $(,~$notmode:expr)?
            ;$($action:expr),*
        );*
        $(;)*
    ) => {{
        bindings!(
            KeyBinding;
            $(Key::KeyCode($key)
              $(,$mods)?
              $(,+$mode)?
              $(,~$notmode)?
              ;$($action),*
            );*
        )
    }};
    (
        $ty:ident;
        $(
            $key:expr
            $(,$mods:expr)?
            $(,+$mode:expr)?
            $(,~$notmode:expr)?
            ;$($action:expr),*
        );*
        $(;)*
    ) => {{
        let mut v = Vec::new();
        $(
            let mut _mods = ModifiersState::empty();
            $(_mods = $mods;)?
            let mut _mode = Mode::None;
            $(_mode = $mode;)?
            let mut _notmode = Mode::None;
            $(_notmode = $notmode;)?
            let mut _actions: Vec<Action> = vec!();
            $(_actions.push($action);)*

            v.push($ty {
                trigger: $key,
                mods: _mods,
                mode: _mode,
                notmode: _notmode,
                actions: _actions,
            });
        )*

        v
    }};
}

macro_rules! motion {
    ($motion: ident) => {{
        Action::Move(Motion::$motion, Quantity::Character)
    }};
}

const ALPHA: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const ALPHA_LOWER: &str = "abcdefghijklmnopqrstuvwxyz";
const NUMERIC: &str = "0123456789";

macro_rules! bindings_key_range {
    (

        $ty:ident;
        $(
            [$key_start:literal-$key_end:literal]
            $(,$mods:expr)?
            $(,+$mode:expr)?
            $(,~$notmode:expr)?
            ;$($action:ident),*
        );*
        $(;)*
    ) => {{
        let mut v = Vec::new();
        $(
            let mut _mods = ModifiersState::empty();
            $(_mods = $mods;)?
            let key_range = if $key_start >= 'A' && $key_end <= 'Z' {
                if _mods.contains(ModifiersState::SHIFT) {
                    ALPHA
                } else {
                    ALPHA_LOWER
                }
            } else if ($key_start >= '0' && $key_end <= '9') {
                NUMERIC
            } else {
                ""
            };
            let mut _mode = Mode::None;
            $(_mode = $mode;)?
            let mut _notmode = Mode::None;
            $(_notmode = $notmode;)?

            key_range.chars().for_each(|k| {
                let mut _actions: Vec<Action> = vec!();
                $(
                    if _mods.contains(ModifiersState::SHIFT) {
                        _actions.push(Action::$action(k.to_string()));
                    } else {
                        _actions.push(Action::$action(k.to_string()));
                    }
                )*
                if let Some(keycode) = map_char(k) {
                    v.push(KeyBinding {
                        trigger: Key::KeyCode(keycode),
                        mods: _mods,
                        mode: _mode,
                        notmode: _notmode,
                        actions: _actions.clone(),
                    });
                }
            });
        )*
        
        v
    }};
}

fn default_symbol_bindings() -> Vec<KeyBinding> {
    let bindings = bindings!(
        KeyBinding; 

        Period, +Mode::Insert; Action::InsertChars(".".into());
        Comma, +Mode::Insert; Action::InsertChars(",".into());
        Apostrophe, +Mode::Insert; Action::InsertChars("'".into());
        Semicolon, +Mode::Insert; Action::InsertChars(";".into());
        Slash, +Mode::Insert; Action::InsertChars("/".into());

        Period, shift!(), +Mode::Insert; Action::InsertChars(">".into());
        Comma, shift!(), +Mode::Insert; Action::InsertChars("<".into());
        Apostrophe, shift!(), +Mode::Insert; Action::InsertChars("\"".into());
        Colon, +Mode::Insert; Action::InsertChars(":".into());
        Semicolon, shift!(), +Mode::Insert; Action::InsertChars(":".into());
        Slash, shift!(), +Mode::Insert; Action::InsertChars("?".into());
        Backslash, shift!(), +Mode::Insert; Action::InsertChars("\\".into());

        Key0, shift!(), +Mode::Insert; Action::InsertChars(")".into());
        Key1, shift!(), +Mode::Insert; Action::InsertChars("!".into());
        Key2, shift!(), +Mode::Insert; Action::InsertChars("@".into());
        Key3, shift!(), +Mode::Insert; Action::InsertChars("#".into());
        Key4, shift!(), +Mode::Insert; Action::InsertChars("$".into());
        Key5, shift!(), +Mode::Insert; Action::InsertChars("%".into());
        Key6, shift!(), +Mode::Insert; Action::InsertChars("^".into());
        Key7, shift!(), +Mode::Insert; Action::InsertChars("&".into());
        Key8, shift!(), +Mode::Insert; Action::InsertChars("*".into());
        Key9, shift!(), +Mode::Insert; Action::InsertChars("(".into());

        Plus, +Mode::Insert; Action::InsertChars("+".into());
        Minus, +Mode::Insert; Action::InsertChars("-".into());
        Equals, +Mode::Insert; Action::InsertChars("=".into());
        LBracket, shift!(), +Mode::Insert; Action::InsertChars("{".into());
        RBracket, shift!(), +Mode::Insert; Action::InsertChars("}".into());
        LBracket, +Mode::Insert; Action::InsertChars("[".into());
        RBracket, +Mode::Insert; Action::InsertChars("]".into());
    );

    bindings
}

pub fn default_key_bindings() -> Vec<KeyBinding> {
    let mut bindings = bindings!(
        KeyBinding;
        
        // Modes
        Escape, +Mode::Visual; Action::CollapseSelections;
        Escape, ~Mode::Normal; Action::SetMode(Mode::Normal);
        I, +Mode::Normal; Action::SetMode(Mode::Insert);
        V, +Mode::Normal; Action::SetMode(Mode::Visual);
        V, shift!(), +Mode::Normal; Action::Move(Motion::Begin, Quantity::Line), Action::MoveSelection(Motion::End, Quantity::Line), Action::SetMode(Mode::Visual);
        V, ctrl!(), +Mode::Normal; Action::SetMode(Mode::Visual);
        A, +Mode::Normal; motion!(Forward), Action::SetMode(Mode::Insert);
        A, shift!(), +Mode::Normal; Action::Move(Motion::End, Quantity::Line), Action::SetMode(Mode::Insert);
        S, +Mode::Normal; Action::Delete(Motion::Forward, Quantity::Character), Action::SetMode(Mode::Insert);
        S, shift!(), +Mode::Normal; Action::Move(Motion::Begin, Quantity::Line), Action::MoveSelection(Motion::Last, Quantity::Line), 
            Action::Delete(Motion::Forward, Quantity::Selection), Action::SetMode(Mode::Insert);
        O, +Mode::Normal; Action::Move(Motion::End, Quantity::Line), Action::InsertNewline, Action::SetMode(Mode::Insert);
        O, shift!(), +Mode::Normal; motion!(Above), Action::Move(Motion::End, Quantity::Line), Action::InsertNewline, Action::SetMode(Mode::Insert);
        D, +Mode::Normal; Action::SetMode(Mode::Delete);
        Colon, +Mode::Normal; Action::SetMode(Mode::Command);
        
        // Insert
        Back, +Mode::Insert; Action::Delete(Motion::Backward, Quantity::Character);
        Delete, +Mode::Insert; Action::Delete(Motion::Forward, Quantity::Character);
        Delete, +Mode::Normal; Action::Delete(Motion::Forward, Quantity::Character);
        Space, +Mode::Insert; Action::InsertChars(" ".into());
        Space, +Mode::Normal; Action::Move(Motion::Forward, Quantity::Character);
        Return, +Mode::Insert; Action::InsertNewline;
        Tab, +Mode::Insert; Action::InsertTab;

        // Character 
        Up, +Mode::Normal; Action::Move(Motion::Above, Quantity::Character);
        Down, +Mode::Normal; Action::Move(Motion::Below, Quantity::Character);
        Left, +Mode::Normal; Action::Move(Motion::Backward, Quantity::Character);
        Right, +Mode::Normal; Action::Move(Motion::Forward, Quantity::Character);
        Up, +Mode::Insert; Action::Move(Motion::Above, Quantity::Character);
        Down, +Mode::Insert; Action::Move(Motion::Below, Quantity::Character);
        Left, +Mode::Insert; Action::Move(Motion::Backward, Quantity::Character);
        Right, +Mode::Insert; Action::Move(Motion::Forward, Quantity::Character);

        // Word 
        Up, ctrl!(), +Mode::Normal; Action::Move(Motion::Above, Quantity::Word);
        Down, ctrl!(), +Mode::Normal; Action::Move(Motion::Below, Quantity::Word);
        Left, ctrl!(), +Mode::Normal; Action::Move(Motion::Backward, Quantity::Word);
        Right, ctrl!(), +Mode::Normal; Action::Move(Motion::Forward, Quantity::Word);
        W, +Mode::Normal; Action::Move(Motion::Forward, Quantity::Word);
        B, +Mode::Normal; Action::Move(Motion::Backward, Quantity::Word);
        Up, ctrl!(), +Mode::Insert; Action::Move(Motion::Above, Quantity::Word);
        Down, ctrl!(), +Mode::Insert; Action::Move(Motion::Below, Quantity::Word);
        Left, ctrl!(), +Mode::Insert; Action::Move(Motion::Backward, Quantity::Word);
        Right, ctrl!(), +Mode::Insert; Action::Move(Motion::Forward, Quantity::Word);

        // Line
        PageUp, +Mode::Normal; Action::Move(Motion::Above, Quantity::Page);
        PageDown, +Mode::Normal; Action::Move(Motion::Below, Quantity::Page);
        Home, +Mode::Normal; Action::Move(Motion::First, Quantity::Line);
        End, +Mode::Normal; Action::Move(Motion::Last, Quantity::Line); 
        PageUp, +Mode::Insert; Action::Move(Motion::Above, Quantity::Page);
        PageDown, +Mode::Insert; Action::Move(Motion::Below, Quantity::Page);
        Home, +Mode::Insert; Action::Move(Motion::First, Quantity::Line);
        End, +Mode::Insert; Action::Move(Motion::Last, Quantity::Line); 
        G, shift!(), +Mode::Normal; Action::Move(Motion::Last, Quantity::Line);

        // Delete
        Left, +Mode::Delete; Action::Delete(Motion::Backward, Quantity::Character);
        Right, +Mode::Delete; Action::Delete(Motion::Forward, Quantity::Character);
        Up, +Mode::Delete; Action::Delete(Motion::Above, Quantity::Line);
        Down, +Mode::Delete; Action::Delete(Motion::Below, Quantity::Line);
        D, +Mode::Delete; Action::Move(Motion::First, Quantity::Line), Action::Delete(Motion::Forward, Quantity::Line);

        // Visual
        Left, +Mode::Visual; Action::MoveSelection(Motion::Backward, Quantity::Character);
        Right, +Mode::Visual; Action::MoveSelection(Motion::Forward, Quantity::Character);
        Up, +Mode::Visual; Action::MoveSelection(Motion::Above, Quantity::Character);
        Down, +Mode::Visual; Action::MoveSelection(Motion::Below, Quantity::Character);

        Left, ctrl!(), +Mode::Visual; Action::MoveSelection(Motion::Backward, Quantity::Word);
        Right, ctrl!(), +Mode::Visual; Action::MoveSelection(Motion::Forward, Quantity::Word);
        Up, ctrl!(), +Mode::Visual; Action::MoveSelection(Motion::Above, Quantity::Word);
        Down, ctrl!(), +Mode::Visual; Action::MoveSelection(Motion::Below, Quantity::Word);

        W, +Mode::Visual; Action::MoveSelection(Motion::Forward, Quantity::Word);
        D, +Mode::Visual; Action::Delete(Motion::Forward, Quantity::Selection);
        D, shift!(), +Mode::Visual; Action::CollapseSelections, Action::Move(Motion::First, Quantity::Character), Action::MoveSelection(Motion::Last, Quantity::Line), Action::Delete(Motion::Forward, Quantity::Selection);
    );
    let mut insert_symbol_bindings = default_symbol_bindings();
    bindings.append(&mut insert_symbol_bindings);

    let mut insert_bindings_ranges = bindings_key_range!(
        KeyBinding; 
        ['0'-'9'], +Mode::Insert; InsertChars;
        ['A'-'Z'], +Mode::Insert; InsertChars;
        ['A'-'Z'], shift!(), +Mode::Insert; InsertChars;
    );
    bindings.append(&mut insert_bindings_ranges);

    bindings
}

pub fn default_mouse_bindings() -> Vec<MouseBinding> {
    vec![]
}
