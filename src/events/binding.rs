
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Binding<T> {
    mods: ModifiersState,
    mode: Mode,
    notmode: Mode,
    trigger: T,
    action: Action,
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

    pub fn get_action(&self) -> Action {
        self.action.clone()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Motion(Motion),
    MotionSelect(Motion),
    MotionDelete(Motion),
    SetMode(Mode),
    SetTheme(String),
    ShowLineNumbers(bool),
    Back,
    Delete,
    Indent,
    Outdent,
    NewLine,
    SearchNext,
    SearchPrev,
    SearchStart,
    SearchEnd,
    Open,
    Quit,
    Copy,
    Cut,
    Paste,
    IncreaseFontSize,
    DecreaseFontSize,
    ScrollPageUp,
    ScrollPageDown,
    ScrollHalfPageUp,
    ScrollHalfPageDown,
    ScrollLineUp,
    ScrollLineDown,
    ScrollToTop,
    ScrollToBottom,
    ClearSelection,
    ReceiveChar(char),
    Undo,
    Redo,
    UpperCase,
    LowerCase,
    AddCursorAbove,
    AddCursorBelow,
    SingleSelection,
    SelectAll,

    None,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Mode {
    Normal,
    Insert,
    Replace,
    Select,
    LineSelect,
    BlockSelect,
    Command,

    None,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Motion {
    Up,
    Down,
    Left,
    Right,
    First,
    Last,
    FirstOccupied,
    High,
    Middle,
    Low,
    SemanticLeft,
    SemanticRight,
    SemanticRightEnd,
    WordLeft,
    WordRight,
    WordRightEnd,
    Bracket,
}

macro_rules! bindings {
    (
        KeyBinding;
        $(
            $key:ident
            $(,$mods:expr)*
            $(,+$mode:expr)*
            $(,~$notmode:expr)*
            ;$action:expr
        );*
        $(;)*
    ) => {{
        bindings!(
            KeyBinding;
            $(
                Key::KeyCode($key)
                $(,$mods)*
                $(,+$mode)*
                $(,~$notmode)*
                ;$action
            );*
        )
    }};
    (
        $ty:ident;
        $(
            $key:expr
            $(,$mods:expr)*
            $(,+$mode:expr)*
            $(,~$notmode:expr)*
            ;$action:expr
        );*
        $(;)*
    ) => {{
        let mut v = Vec::new();

        $(
            let mut _mods = ModifiersState::empty();
            $(_mods = $mods;)*
            let mut _mode = Mode::None;
            $(_mode = $mode;)*
            let mut _notmode = Mode::None;
            $(_notmode = $notmode;)*

            v.push($ty {
                trigger: $key,
                mods: _mods,
                mode: _mode,
                notmode: _notmode,
                action: $action.into(),
            });
        )*

        v
    }};
}

pub fn default_mouse_bindings() -> Vec<MouseBinding> {
    vec!() 
}

pub fn default_key_bindings() -> Vec<KeyBinding> {
    let bindings = bindings!(
        KeyBinding;

        F1; Action::SetTheme(String::from("Solarized (dark)"));
        F2; Action::SetTheme(String::from("Solarized (light)"));
        F3; Action::SetTheme(String::from("InspiredGitHub"));
        F5; Action::ShowLineNumbers(true);

        Escape, ~Mode::Normal; Action::SetMode(Mode::Normal);
        Escape, +Mode::LineSelect; Action::ClearSelection;
        Escape, +Mode::BlockSelect; Action::ClearSelection;
        Escape, +Mode::Select; Action::ClearSelection;
        I, +Mode::Normal; Action::SetMode(Mode::Insert);
        V, +Mode::Normal; Action::SetMode(Mode::Select);
        V, ModifiersState::SHIFT, +Mode::Normal; Action::SetMode(Mode::BlockSelect);
        V, ModifiersState::CTRL, +Mode::Normal; Action::SetMode(Mode::LineSelect);
        R, ModifiersState::SHIFT, +Mode::Normal; Action::SetMode(Mode::Replace);
        Colon, ModifiersState::SHIFT, +Mode::Normal; Action::SetMode(Mode::Command);

        PageUp; Action::ScrollPageUp;
        PageDown; Action::ScrollPageDown;

        Home; Action::Motion(Motion::First);
        End; Action::Motion(Motion::Last);
        Key0, ~Mode::Insert; Action::Motion(Motion::First);
        Key4, ModifiersState::SHIFT, ~Mode::Insert; Action::Motion(Motion::Last);
        Key5, ModifiersState::SHIFT, ~Mode::Insert; Action::Motion(Motion::Bracket);
        Key6, ModifiersState::SHIFT, ~Mode::Insert; Action::Motion(Motion::FirstOccupied);

        Up, ModifiersState::SHIFT | ModifiersState::CTRL; Action::AddCursorAbove;
        Down, ModifiersState::SHIFT | ModifiersState::CTRL; Action::AddCursorBelow;
        Up, ModifiersState::SHIFT; Action::ScrollPageUp;
        Down, ModifiersState::SHIFT; Action::ScrollPageDown;

        Left; Action::Motion(Motion::Left);
        Right; Action::Motion(Motion::Right);
        Up; Action::Motion(Motion::Up);
        Down; Action::Motion(Motion::Down);
        
        H, ~Mode::Insert; Action::Motion(Motion::Left);
        J, ~Mode::Insert; Action::Motion(Motion::Down);
        K, ~Mode::Insert; Action::Motion(Motion::Up);
        L, ~Mode::Insert; Action::Motion(Motion::Right);

        Return, +Mode::Insert; Action::NewLine;
        Return, ~Mode::Insert; Action::Motion(Motion::Down);
        Back, +Mode::Insert; Action::Back;
        Delete, +Mode::Insert; Action::Delete;
        
        Y, +Mode::Select; Action::Copy;
        Y, +Mode::LineSelect; Action::Copy;
        Y, +Mode::BlockSelect; Action::Copy;
        Copy, +Mode::Insert; Action::Copy;

        P, +Mode::Insert; Action::Paste;
        Paste, +Mode::Insert; Action::Paste;
        
        U, +Mode::Insert; Action::Undo;
        R, ModifiersState::CTRL, +Mode::Insert; Action::Redo;

        Tab, +Mode::Insert; Action::Indent;
        Tab, ModifiersState::SHIFT, +Mode::Insert; Action::Outdent;

        Minus, ModifiersState::CTRL; Action::DecreaseFontSize;
        Subtract, ModifiersState::CTRL; Action::DecreaseFontSize;
        Equals, ModifiersState::CTRL; Action::IncreaseFontSize;
        Add, ModifiersState::CTRL; Action::IncreaseFontSize;
        
        A, ModifiersState::SHIFT, +Mode::Insert; Action::ReceiveChar('A');
        B, ModifiersState::SHIFT, +Mode::Insert; Action::ReceiveChar('B');
        C, ModifiersState::SHIFT, +Mode::Insert; Action::ReceiveChar('C');
        D, ModifiersState::SHIFT, +Mode::Insert; Action::ReceiveChar('D');
        E, ModifiersState::SHIFT, +Mode::Insert; Action::ReceiveChar('E');
        F, ModifiersState::SHIFT, +Mode::Insert; Action::ReceiveChar('F');
        G, ModifiersState::SHIFT, +Mode::Insert; Action::ReceiveChar('G');
        H, ModifiersState::SHIFT, +Mode::Insert; Action::ReceiveChar('H');
        I, ModifiersState::SHIFT, +Mode::Insert; Action::ReceiveChar('I');
        J, ModifiersState::SHIFT, +Mode::Insert; Action::ReceiveChar('J');
        K, ModifiersState::SHIFT, +Mode::Insert; Action::ReceiveChar('K');
        L, ModifiersState::SHIFT, +Mode::Insert; Action::ReceiveChar('L');
        M, ModifiersState::SHIFT, +Mode::Insert; Action::ReceiveChar('M');
        N, ModifiersState::SHIFT, +Mode::Insert; Action::ReceiveChar('N');
        O, ModifiersState::SHIFT, +Mode::Insert; Action::ReceiveChar('O');
        P, ModifiersState::SHIFT, +Mode::Insert; Action::ReceiveChar('P');
        Q, ModifiersState::SHIFT, +Mode::Insert; Action::ReceiveChar('Q');
        R, ModifiersState::SHIFT, +Mode::Insert; Action::ReceiveChar('R');
        S, ModifiersState::SHIFT, +Mode::Insert; Action::ReceiveChar('S');
        T, ModifiersState::SHIFT, +Mode::Insert; Action::ReceiveChar('T');
        U, ModifiersState::SHIFT, +Mode::Insert; Action::ReceiveChar('U');
        V, ModifiersState::SHIFT, +Mode::Insert; Action::ReceiveChar('V');
        W, ModifiersState::SHIFT, +Mode::Insert; Action::ReceiveChar('W');
        X, ModifiersState::SHIFT, +Mode::Insert; Action::ReceiveChar('X');
        Y, ModifiersState::SHIFT, +Mode::Insert; Action::ReceiveChar('Y');
        Z, ModifiersState::SHIFT, +Mode::Insert; Action::ReceiveChar('Z');
        
        A, +Mode::Insert; Action::ReceiveChar('a');
        B, +Mode::Insert; Action::ReceiveChar('b');
        C, +Mode::Insert; Action::ReceiveChar('c');
        D, +Mode::Insert; Action::ReceiveChar('d');
        E, +Mode::Insert; Action::ReceiveChar('e');
        F, +Mode::Insert; Action::ReceiveChar('f');
        G, +Mode::Insert; Action::ReceiveChar('g');
        H, +Mode::Insert; Action::ReceiveChar('h');
        I, +Mode::Insert; Action::ReceiveChar('i');
        J, +Mode::Insert; Action::ReceiveChar('j');
        K, +Mode::Insert; Action::ReceiveChar('k');
        L, +Mode::Insert; Action::ReceiveChar('l');
        M, +Mode::Insert; Action::ReceiveChar('m');
        N, +Mode::Insert; Action::ReceiveChar('n');
        O, +Mode::Insert; Action::ReceiveChar('o');
        P, +Mode::Insert; Action::ReceiveChar('p');
        Q, +Mode::Insert; Action::ReceiveChar('q');
        R, +Mode::Insert; Action::ReceiveChar('r');
        S, +Mode::Insert; Action::ReceiveChar('s');
        T, +Mode::Insert; Action::ReceiveChar('t');
        U, +Mode::Insert; Action::ReceiveChar('u');
        V, +Mode::Insert; Action::ReceiveChar('v');
        W, +Mode::Insert; Action::ReceiveChar('w');
        X, +Mode::Insert; Action::ReceiveChar('x');
        Y, +Mode::Insert; Action::ReceiveChar('y');
        Z, +Mode::Insert; Action::ReceiveChar('z');

        Key1, ModifiersState::SHIFT, +Mode::Insert; Action::ReceiveChar('!');
        Key2, ModifiersState::SHIFT, +Mode::Insert; Action::ReceiveChar('@');
        Key3, ModifiersState::SHIFT, +Mode::Insert; Action::ReceiveChar('#');
        Key4, ModifiersState::SHIFT, +Mode::Insert; Action::ReceiveChar('$');
        Key5, ModifiersState::SHIFT, +Mode::Insert; Action::ReceiveChar('%');
        Key6, ModifiersState::SHIFT, +Mode::Insert; Action::ReceiveChar('^');
        Key7, ModifiersState::SHIFT, +Mode::Insert; Action::ReceiveChar('&');
        Key8, ModifiersState::SHIFT, +Mode::Insert; Action::ReceiveChar('*');
        Key9, ModifiersState::SHIFT, +Mode::Insert; Action::ReceiveChar('(');
        Key0, ModifiersState::SHIFT, +Mode::Insert; Action::ReceiveChar(')');
        
        Key1, +Mode::Insert; Action::ReceiveChar('1');
        Key2, +Mode::Insert; Action::ReceiveChar('2');
        Key3, +Mode::Insert; Action::ReceiveChar('3');
        Key4, +Mode::Insert; Action::ReceiveChar('4');
        Key5, +Mode::Insert; Action::ReceiveChar('5');
        Key6, +Mode::Insert; Action::ReceiveChar('6');
        Key7, +Mode::Insert; Action::ReceiveChar('7');
        Key8, +Mode::Insert; Action::ReceiveChar('8');
        Key9, +Mode::Insert; Action::ReceiveChar('9');
        Key0, +Mode::Insert; Action::ReceiveChar('0');

        Grave,      ModifiersState::SHIFT, +Mode::Insert; Action::ReceiveChar('~');
        Minus,      ModifiersState::SHIFT, +Mode::Insert; Action::ReceiveChar('_');
        Equals,     ModifiersState::SHIFT, +Mode::Insert; Action::ReceiveChar('+');
        LBracket,   ModifiersState::SHIFT, +Mode::Insert; Action::ReceiveChar('{');
        RBracket,   ModifiersState::SHIFT, +Mode::Insert; Action::ReceiveChar('}');
        Backslash,  ModifiersState::SHIFT, +Mode::Insert; Action::ReceiveChar('|');
        Colon,      ModifiersState::SHIFT, +Mode::Insert; Action::ReceiveChar(':');
        Apostrophe, ModifiersState::SHIFT, +Mode::Insert; Action::ReceiveChar('"');
        Comma,      ModifiersState::SHIFT, +Mode::Insert; Action::ReceiveChar('<');
        Period,     ModifiersState::SHIFT, +Mode::Insert; Action::ReceiveChar('>');
        Slash,      ModifiersState::SHIFT, +Mode::Insert; Action::ReceiveChar('?');

        Grave,      +Mode::Insert; Action::ReceiveChar('`');
        Minus,      +Mode::Insert; Action::ReceiveChar('-');
        Equals,     +Mode::Insert; Action::ReceiveChar('=');
        LBracket,   +Mode::Insert; Action::ReceiveChar('[');
        RBracket,   +Mode::Insert; Action::ReceiveChar(']');
        Backslash,  +Mode::Insert; Action::ReceiveChar('\\');
        Semicolon,  +Mode::Insert; Action::ReceiveChar(';');
        Apostrophe, +Mode::Insert; Action::ReceiveChar('\'');
        Comma,      +Mode::Insert; Action::ReceiveChar(',');
        Period,     +Mode::Insert; Action::ReceiveChar('.');
        Slash,      +Mode::Insert; Action::ReceiveChar('/');
    );

    bindings
}
