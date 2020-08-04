use std::fmt;

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
    target: ActionTarget,
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

    pub fn get_target(&self) -> ActionTarget {
        self.target.clone()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActionTarget {
    FocusedView,
    StatusBar,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Motion(Motion),
    MotionSelect(Motion),
    MotionDelete(Motion),
    SetMode(Mode),
    SetTheme(String),
    ShowLineNumbers(bool),
    InsertChar(char),
    DefineCommand((String, Box<Action>)),
    ExecuteCommand,
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
    Save,
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
    SingleSelection,
    Undo,
    Redo,
    UpperCase,
    LowerCase,
    AddCursorAbove,
    AddCursorBelow,
    SelectAll,

    None,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
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
impl fmt::Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
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
            $(,@$target:expr)*
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
                $(,@$target)*
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
            $(,@$target:expr)*
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
            let mut _target = ActionTarget::FocusedView;
            $(_target = $target;)*

            v.push($ty {
                trigger: $key,
                mods: _mods,
                mode: _mode,
                notmode: _notmode,
                target: _target,
                action: $action.into(),
            });
        )*

        v
    }};
}

pub fn default_mouse_bindings() -> Vec<MouseBinding> {
    vec!() 
}

pub fn bind_alpha_numeric(mode: Mode) -> Vec<KeyBinding> {
    bindings!(
        KeyBinding;
        A, ModifiersState::SHIFT, +mode; Action::InsertChar('A');
        B, ModifiersState::SHIFT, +mode; Action::InsertChar('B');
        C, ModifiersState::SHIFT, +mode; Action::InsertChar('C');
        D, ModifiersState::SHIFT, +mode; Action::InsertChar('D');
        E, ModifiersState::SHIFT, +mode; Action::InsertChar('E');
        F, ModifiersState::SHIFT, +mode; Action::InsertChar('F');
        G, ModifiersState::SHIFT, +mode; Action::InsertChar('G');
        H, ModifiersState::SHIFT, +mode; Action::InsertChar('H');
        I, ModifiersState::SHIFT, +mode; Action::InsertChar('I');
        J, ModifiersState::SHIFT, +mode; Action::InsertChar('J');
        K, ModifiersState::SHIFT, +mode; Action::InsertChar('K');
        L, ModifiersState::SHIFT, +mode; Action::InsertChar('L');
        M, ModifiersState::SHIFT, +mode; Action::InsertChar('M');
        N, ModifiersState::SHIFT, +mode; Action::InsertChar('N');
        O, ModifiersState::SHIFT, +mode; Action::InsertChar('O');
        P, ModifiersState::SHIFT, +mode; Action::InsertChar('P');
        Q, ModifiersState::SHIFT, +mode; Action::InsertChar('Q');
        R, ModifiersState::SHIFT, +mode; Action::InsertChar('R');
        S, ModifiersState::SHIFT, +mode; Action::InsertChar('S');
        T, ModifiersState::SHIFT, +mode; Action::InsertChar('T');
        U, ModifiersState::SHIFT, +mode; Action::InsertChar('U');
        V, ModifiersState::SHIFT, +mode; Action::InsertChar('V');
        W, ModifiersState::SHIFT, +mode; Action::InsertChar('W');
        X, ModifiersState::SHIFT, +mode; Action::InsertChar('X');
        Y, ModifiersState::SHIFT, +mode; Action::InsertChar('Y');
        Z, ModifiersState::SHIFT, +mode; Action::InsertChar('Z');
        
        A, +mode; Action::InsertChar('a');
        B, +mode; Action::InsertChar('b');
        C, +mode; Action::InsertChar('c');
        D, +mode; Action::InsertChar('d');
        E, +mode; Action::InsertChar('e');
        F, +mode; Action::InsertChar('f');
        G, +mode; Action::InsertChar('g');
        H, +mode; Action::InsertChar('h');
        I, +mode; Action::InsertChar('i');
        J, +mode; Action::InsertChar('j');
        K, +mode; Action::InsertChar('k');
        L, +mode; Action::InsertChar('l');
        M, +mode; Action::InsertChar('m');
        N, +mode; Action::InsertChar('n');
        O, +mode; Action::InsertChar('o');
        P, +mode; Action::InsertChar('p');
        Q, +mode; Action::InsertChar('q');
        R, +mode; Action::InsertChar('r');
        S, +mode; Action::InsertChar('s');
        T, +mode; Action::InsertChar('t');
        U, +mode; Action::InsertChar('u');
        V, +mode; Action::InsertChar('v');
        W, +mode; Action::InsertChar('w');
        X, +mode; Action::InsertChar('x');
        Y, +mode; Action::InsertChar('y');
        Z, +mode; Action::InsertChar('z');

        Key1, ModifiersState::SHIFT, +mode; Action::InsertChar('!');
        Key2, ModifiersState::SHIFT, +mode; Action::InsertChar('@');
        Key3, ModifiersState::SHIFT, +mode; Action::InsertChar('#');
        Key4, ModifiersState::SHIFT, +mode; Action::InsertChar('$');
        Key5, ModifiersState::SHIFT, +mode; Action::InsertChar('%');
        Key6, ModifiersState::SHIFT, +mode; Action::InsertChar('^');
        Key7, ModifiersState::SHIFT, +mode; Action::InsertChar('&');
        Key8, ModifiersState::SHIFT, +mode; Action::InsertChar('*');
        Key9, ModifiersState::SHIFT, +mode; Action::InsertChar('(');
        Key0, ModifiersState::SHIFT, +mode; Action::InsertChar(')');
        
        Key1, +mode; Action::InsertChar('1');
        Key2, +mode; Action::InsertChar('2');
        Key3, +mode; Action::InsertChar('3');
        Key4, +mode; Action::InsertChar('4');
        Key5, +mode; Action::InsertChar('5');
        Key6, +mode; Action::InsertChar('6');
        Key7, +mode; Action::InsertChar('7');
        Key8, +mode; Action::InsertChar('8');
        Key9, +mode; Action::InsertChar('9');
        Key0, +mode; Action::InsertChar('0');

        Grave,      ModifiersState::SHIFT, +mode; Action::InsertChar('~');
        Minus,      ModifiersState::SHIFT, +mode; Action::InsertChar('_');
        Add,        ModifiersState::SHIFT, +mode; Action::InsertChar('+');
        LBracket,   ModifiersState::SHIFT, +mode; Action::InsertChar('{');
        RBracket,   ModifiersState::SHIFT, +mode; Action::InsertChar('}');
        Backslash,  ModifiersState::SHIFT, +mode; Action::InsertChar('|');
        Colon,      ModifiersState::SHIFT, +mode; Action::InsertChar(':');
        Apostrophe, ModifiersState::SHIFT, +mode; Action::InsertChar('"');
        Comma,      ModifiersState::SHIFT, +mode; Action::InsertChar('<');
        Period,     ModifiersState::SHIFT, +mode; Action::InsertChar('>');
        Slash,      ModifiersState::SHIFT, +mode; Action::InsertChar('?');

        Grave,      +mode; Action::InsertChar('`');
        Minus,      +mode; Action::InsertChar('-');
        Equals,     +mode; Action::InsertChar('=');
        LBracket,   +mode; Action::InsertChar('[');
        RBracket,   +mode; Action::InsertChar(']');
        Backslash,  +mode; Action::InsertChar('\\');
        Semicolon,  +mode; Action::InsertChar(';');
        Apostrophe, +mode; Action::InsertChar('\'');
        Comma,      +mode; Action::InsertChar(',');
        Period,     +mode; Action::InsertChar('.');
        Slash,      +mode; Action::InsertChar('/');
        Space,      +mode; Action::InsertChar(' ');
    )
}

pub fn default_key_bindings() -> Vec<KeyBinding> {
    let mut bindings = bindings!(
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

        Left, +Mode::Select; Action::MotionSelect(Motion::Left);
        Right, +Mode::Select; Action::MotionSelect(Motion::Right);
        Up, +Mode::Select; Action::MotionSelect(Motion::Up);
        Down, +Mode::Select; Action::MotionSelect(Motion::Down);
        H, +Mode::Select; Action::MotionSelect(Motion::Left);
        J, +Mode::Select; Action::MotionSelect(Motion::Down);
        K, +Mode::Select; Action::MotionSelect(Motion::Up);
        L, +Mode::Select; Action::MotionSelect(Motion::Right);

        Left, ModifiersState::CTRL, ~Mode::Insert; Action::Motion(Motion::WordLeft);
        Right, ModifiersState::CTRL, ~Mode::Insert; Action::Motion(Motion::WordRight);
        W, ~Mode::Insert; Action::Motion(Motion::WordRight);
        E, ~Mode::Insert; Action::Motion(Motion::WordRightEnd);
        D, ModifiersState::SHIFT, +Mode::Normal; Action::MotionDelete(Motion::Last);

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
        Back, ~Mode::Insert; Action::Motion(Motion::Left);
        Delete, ~Mode::Insert; Action::Motion(Motion::Right);
        Space, ~Mode::Insert; Action::Motion(Motion::Right);
        
        Y, +Mode::Select; Action::Copy;
        Y, +Mode::LineSelect; Action::Copy;
        Y, +Mode::BlockSelect; Action::Copy;
        Copy, +Mode::Insert; Action::Copy;
        Cut, +Mode::Insert; Action::Cut;

        P, ~Mode::Insert; Action::Paste;
        Paste, +Mode::Insert; Action::Paste;
        
        U, ~Mode::Insert; Action::Undo;
        R, ModifiersState::CTRL, ~Mode::Insert; Action::Redo;

        Tab, +Mode::Insert; Action::Indent;
        Tab, ModifiersState::SHIFT, +Mode::Insert; Action::Outdent;

        Minus, ModifiersState::CTRL; Action::DecreaseFontSize;
        Subtract, ModifiersState::CTRL; Action::DecreaseFontSize;
        Equals, ModifiersState::CTRL; Action::IncreaseFontSize;
        Add, ModifiersState::CTRL; Action::IncreaseFontSize;

        W, +Mode::Command, @ActionTarget::StatusBar; Action::InsertChar('w');
        W, ModifiersState::SHIFT, +Mode::Command, @ActionTarget::StatusBar; Action::InsertChar('w');
        Return, +Mode::Command, @ActionTarget::StatusBar; Action::ExecuteCommand;
    );

    bindings.extend(bind_alpha_numeric(Mode::Insert));

    bindings
}
