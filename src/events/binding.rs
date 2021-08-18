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
    fn eq(&self, other: &Self) -> bool {
       self.is_triggered_by(other.mode, other.mods, &other.trigger) 
    }
}


macro_rules! shift { () => {{ ModifiersState::SHIFT }}; }
macro_rules! ctrl { () => {{ ModifiersState::CTRL }}; }
macro_rules! alt { () => {{ ModifiersState::ALT }}; }
macro_rules! no_mods { () => {{ ModifiersState::empty() }}; }

macro_rules! key_binding {
    ($key:ident
     ,$mods:expr
     ,$mode:ident
     ;$($action:expr),*
    ) => {{
        let mut _mods: ModifiersState = $mods;
        let _notmode = Mode::None;
        let mut _mode: Mode = Mode::$mode;
        let mut _actions: Vec<Action> = Vec::new();
        $(_actions.push($action);)*
        KeyBinding {
            trigger: Key::KeyCode($key),
            mods: _mods,
            mode: _mode,
            notmode: _notmode,
            actions: _actions,
        }
    }};
}

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
            let key_range = if $key_start >= 'A' && $key_end <= 'Z' {
                ALPHA
            } else if ($key_start >= 'a' && $key_end <= '9') {
                ALPHA_LOWER
            } else if ($key_start >= '0' && $key_end <= '9') {
                NUMERIC
            } else {
                ""
            };
            let mut _mods = ModifiersState::empty();
            $(_mods = $mods;)?
            let mut _mode = Mode::None;
            $(_mode = $mode;)?
            let mut _notmode = Mode::None;
            $(_notmode = $notmode;)?

            key_range.chars().for_each(|k| {
                let mut _actions: Vec<Action> = vec!();
                $(
                    _actions.push(Action::$action(k.to_string()));
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

pub fn default_key_bindings() -> Vec<KeyBinding> {
    let mut bindings = bindings!(
        KeyBinding;
        
        // Modes
        Escape, ~Mode::Normal; Action::SetMode(Mode::Normal);
        I, +Mode::Normal; Action::SetMode(Mode::Insert);
        V, +Mode::Normal; Action::SetMode(Mode::Visual);
        V, shift!(), +Mode::Normal; Action::Move(Motion::First, Quantity::Character), Action::MoveSelection(Motion::Last, Quantity::Character), Action::SetMode(Mode::Visual);
        V, ctrl!(), +Mode::Normal; Action::SetMode(Mode::Visual);
        A, +Mode::Normal; motion!(Forward), Action::SetMode(Mode::Insert);
        S, +Mode::Normal; Action::Delete(Motion::Forward, Quantity::Character), Action::SetMode(Mode::Insert);
        Colon, +Mode::Normal; Action::SetMode(Mode::Command);       
    );

    let mut insert_bindings = bindings_key_range!(
        KeyBinding; 
        ['A'-'Z'], +Mode::Insert; InsertChars;
    );

    bindings.append(&mut insert_bindings);

    bindings
}

pub fn default_mouse_bindings() -> Vec<MouseBinding> {
    vec![]
}
