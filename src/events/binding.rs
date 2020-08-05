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

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
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
        match self {
            Mode::Normal => write!(f, "NORMAL"),
            Mode::Insert => write!(f, "INSERT"),
            Mode::Replace => write!(f, "REPLACE"),
            Mode::Command => write!(f, "COMMAND"),
            Mode::Select => write!(f, "VISUAL"),
            Mode::BlockSelect => write!(f, "V-BLOCK"),
            Mode::LineSelect => write!(f, "V-LINE"),
            _ => write!(f, "{:?}", self),
        }
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
    bindings!(
        MouseBinding;
    )
}

pub fn bind_alpha_numeric(mode: Mode, target: ActionTarget) -> Vec<KeyBinding> {
    bindings!(
        KeyBinding;
        A, ModifiersState::SHIFT, +mode, @target; Action::InsertChar('A');
        B, ModifiersState::SHIFT, +mode, @target; Action::InsertChar('B');
        C, ModifiersState::SHIFT, +mode, @target; Action::InsertChar('C');
        D, ModifiersState::SHIFT, +mode, @target; Action::InsertChar('D');
        E, ModifiersState::SHIFT, +mode, @target; Action::InsertChar('E');
        F, ModifiersState::SHIFT, +mode, @target; Action::InsertChar('F');
        G, ModifiersState::SHIFT, +mode, @target; Action::InsertChar('G');
        H, ModifiersState::SHIFT, +mode, @target; Action::InsertChar('H');
        I, ModifiersState::SHIFT, +mode, @target; Action::InsertChar('I');
        J, ModifiersState::SHIFT, +mode, @target; Action::InsertChar('J');
        K, ModifiersState::SHIFT, +mode, @target; Action::InsertChar('K');
        L, ModifiersState::SHIFT, +mode, @target; Action::InsertChar('L');
        M, ModifiersState::SHIFT, +mode, @target; Action::InsertChar('M');
        N, ModifiersState::SHIFT, +mode, @target; Action::InsertChar('N');
        O, ModifiersState::SHIFT, +mode, @target; Action::InsertChar('O');
        P, ModifiersState::SHIFT, +mode, @target; Action::InsertChar('P');
        Q, ModifiersState::SHIFT, +mode, @target; Action::InsertChar('Q');
        R, ModifiersState::SHIFT, +mode, @target; Action::InsertChar('R');
        S, ModifiersState::SHIFT, +mode, @target; Action::InsertChar('S');
        T, ModifiersState::SHIFT, +mode, @target; Action::InsertChar('T');
        U, ModifiersState::SHIFT, +mode, @target; Action::InsertChar('U');
        V, ModifiersState::SHIFT, +mode, @target; Action::InsertChar('V');
        W, ModifiersState::SHIFT, +mode, @target; Action::InsertChar('W');
        X, ModifiersState::SHIFT, +mode, @target; Action::InsertChar('X');
        Y, ModifiersState::SHIFT, +mode, @target; Action::InsertChar('Y');
        Z, ModifiersState::SHIFT, +mode, @target; Action::InsertChar('Z');
        
        A, +mode, @target; Action::InsertChar('a');
        B, +mode, @target; Action::InsertChar('b');
        C, +mode, @target; Action::InsertChar('c');
        D, +mode, @target; Action::InsertChar('d');
        E, +mode, @target; Action::InsertChar('e');
        F, +mode, @target; Action::InsertChar('f');
        G, +mode, @target; Action::InsertChar('g');
        H, +mode, @target; Action::InsertChar('h');
        I, +mode, @target; Action::InsertChar('i');
        J, +mode, @target; Action::InsertChar('j');
        K, +mode, @target; Action::InsertChar('k');
        L, +mode, @target; Action::InsertChar('l');
        M, +mode, @target; Action::InsertChar('m');
        N, +mode, @target; Action::InsertChar('n');
        O, +mode, @target; Action::InsertChar('o');
        P, +mode, @target; Action::InsertChar('p');
        Q, +mode, @target; Action::InsertChar('q');
        R, +mode, @target; Action::InsertChar('r');
        S, +mode, @target; Action::InsertChar('s');
        T, +mode, @target; Action::InsertChar('t');
        U, +mode, @target; Action::InsertChar('u');
        V, +mode, @target; Action::InsertChar('v');
        W, +mode, @target; Action::InsertChar('w');
        X, +mode, @target; Action::InsertChar('x');
        Y, +mode, @target; Action::InsertChar('y');
        Z, +mode, @target; Action::InsertChar('z');

        Key1, ModifiersState::SHIFT, +mode, @target; Action::InsertChar('!');
        Key2, ModifiersState::SHIFT, +mode, @target; Action::InsertChar('@');
        Key3, ModifiersState::SHIFT, +mode, @target; Action::InsertChar('#');
        Key4, ModifiersState::SHIFT, +mode, @target; Action::InsertChar('$');
        Key5, ModifiersState::SHIFT, +mode, @target; Action::InsertChar('%');
        Key6, ModifiersState::SHIFT, +mode, @target; Action::InsertChar('^');
        Key7, ModifiersState::SHIFT, +mode, @target; Action::InsertChar('&');
        Key8, ModifiersState::SHIFT, +mode, @target; Action::InsertChar('*');
        Key9, ModifiersState::SHIFT, +mode, @target; Action::InsertChar('(');
        Key0, ModifiersState::SHIFT, +mode, @target; Action::InsertChar(')');
        
        Key1, +mode, @target; Action::InsertChar('1');
        Key2, +mode, @target; Action::InsertChar('2');
        Key3, +mode, @target; Action::InsertChar('3');
        Key4, +mode, @target; Action::InsertChar('4');
        Key5, +mode, @target; Action::InsertChar('5');
        Key6, +mode, @target; Action::InsertChar('6');
        Key7, +mode, @target; Action::InsertChar('7');
        Key8, +mode, @target; Action::InsertChar('8');
        Key9, +mode, @target; Action::InsertChar('9');
        Key0, +mode, @target; Action::InsertChar('0');

        Grave,      ModifiersState::SHIFT, +mode, @target; Action::InsertChar('~');
        Minus,      ModifiersState::SHIFT, +mode, @target; Action::InsertChar('_');
        Add,        ModifiersState::SHIFT, +mode, @target; Action::InsertChar('+');
        LBracket,   ModifiersState::SHIFT, +mode, @target; Action::InsertChar('{');
        RBracket,   ModifiersState::SHIFT, +mode, @target; Action::InsertChar('}');
        Backslash,  ModifiersState::SHIFT, +mode, @target; Action::InsertChar('|');
        Colon,      ModifiersState::SHIFT, +mode, @target; Action::InsertChar(':');
        Apostrophe, ModifiersState::SHIFT, +mode, @target; Action::InsertChar('"');
        Comma,      ModifiersState::SHIFT, +mode, @target; Action::InsertChar('<');
        Period,     ModifiersState::SHIFT, +mode, @target; Action::InsertChar('>');
        Slash,      ModifiersState::SHIFT, +mode, @target; Action::InsertChar('?');

        Grave,      +mode, @target; Action::InsertChar('`');
        Minus,      +mode, @target; Action::InsertChar('-');
        Equals,     +mode, @target; Action::InsertChar('=');
        LBracket,   +mode, @target; Action::InsertChar('[');
        RBracket,   +mode, @target; Action::InsertChar(']');
        Backslash,  +mode, @target; Action::InsertChar('\\');
        Semicolon,  +mode, @target; Action::InsertChar(';');
        Apostrophe, +mode, @target; Action::InsertChar('\'');
        Comma,      +mode, @target; Action::InsertChar(',');
        Period,     +mode, @target; Action::InsertChar('.');
        Slash,      +mode, @target; Action::InsertChar('/');
        Space,      +mode, @target; Action::InsertChar(' ');
    )
}

pub fn default_key_bindings() -> Vec<KeyBinding> {
    let mut bindings = bindings!(
        KeyBinding;

        F1; Action::SetTheme(String::from("Solarized (dark)"));
        F2; Action::SetTheme(String::from("Solarized (light)"));
        F3; Action::SetTheme(String::from("InspiredGitHub"));
        F5; Action::ShowLineNumbers(true);

        Back, +Mode::Command, @ActionTarget::StatusBar; Action::Back;
        Delete, +Mode::Command, @ActionTarget::StatusBar; Action::Delete;
        Return, +Mode::Command, @ActionTarget::StatusBar; Action::ExecuteCommand;

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
    );

    bindings.extend(bind_alpha_numeric(Mode::Command, ActionTarget::StatusBar));
    bindings.extend(bind_alpha_numeric(Mode::Insert, ActionTarget::FocusedView));

    bindings
}
