use rpc::{
    Action,
    ActionTarget,
    Motion,
    Mode,
    Quantity,
};

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
    target: ActionTarget,
    actions: Vec<Action>,
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

    pub fn get_target(&self) -> ActionTarget {
        self.target
    }

    pub fn get_actions(&self) -> Vec<Action> {
        println!("actions: {:?}", self.actions.clone());
        self.actions.clone()
    }
}
macro_rules! motion {
    (
        $action:ident
        $motion:ident 
        $(by$quantity:ident)*
    ) => {{
        let mut _motion: Motion = Motion::$motion;
        let mut _quantity: Option<Quantity> = None;
        $(_quantity = Some(Quantity::$quantity(1));)*
        Action::$action((_motion, _quantity))
    }};
}
macro_rules! shift {
    () => {{ ModifiersState::SHIFT }};
}
macro_rules! ctrl {
    () => {{ ModifiersState::CTRL }};
}

macro_rules! bindings {
    (
        KeyBinding;
        $(
            $key:ident
            $(,$mods:expr)?
            $(,+$mode:expr)?
            $(,~$notmode:expr)?
            $(,@$target:expr)?
            ;$($action:expr),*
        );*
        $(;)*
    ) => {{
        bindings!(
            KeyBinding;
            $(
                Key::KeyCode($key)
                $(,$mods)?
                $(,+$mode)?
                $(,~$notmode)?
                $(,@$target)?
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
            $(,@$target:expr)?
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
            let mut _target = ActionTarget::FocusedView;
            $(_target = $target;)?
            let mut _actions: Vec<Action> = vec!(); 
            $(_actions.push($action);)*

            v.push($ty {
                trigger: $key,
                mods: _mods,
                mode: _mode,
                notmode: _notmode,
                target: _target,
                actions: _actions,
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
        A, shift!(), +mode, @target; Action::InsertChar('A');
        B, shift!(), +mode, @target; Action::InsertChar('B');
        C, shift!(), +mode, @target; Action::InsertChar('C');
        D, shift!(), +mode, @target; Action::InsertChar('D');
        E, shift!(), +mode, @target; Action::InsertChar('E');
        F, shift!(), +mode, @target; Action::InsertChar('F');
        G, shift!(), +mode, @target; Action::InsertChar('G');
        H, shift!(), +mode, @target; Action::InsertChar('H');
        I, shift!(), +mode, @target; Action::InsertChar('I');
        J, shift!(), +mode, @target; Action::InsertChar('J');
        K, shift!(), +mode, @target; Action::InsertChar('K');
        L, shift!(), +mode, @target; Action::InsertChar('L');
        M, shift!(), +mode, @target; Action::InsertChar('M');
        N, shift!(), +mode, @target; Action::InsertChar('N');
        O, shift!(), +mode, @target; Action::InsertChar('O');
        P, shift!(), +mode, @target; Action::InsertChar('P');
        Q, shift!(), +mode, @target; Action::InsertChar('Q');
        R, shift!(), +mode, @target; Action::InsertChar('R');
        S, shift!(), +mode, @target; Action::InsertChar('S');
        T, shift!(), +mode, @target; Action::InsertChar('T');
        U, shift!(), +mode, @target; Action::InsertChar('U');
        V, shift!(), +mode, @target; Action::InsertChar('V');
        W, shift!(), +mode, @target; Action::InsertChar('W');
        X, shift!(), +mode, @target; Action::InsertChar('X');
        Y, shift!(), +mode, @target; Action::InsertChar('Y');
        Z, shift!(), +mode, @target; Action::InsertChar('Z');
        
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

        Key1, shift!(), +mode, @target; Action::InsertChar('!');
        Key2, shift!(), +mode, @target; Action::InsertChar('@');
        Key3, shift!(), +mode, @target; Action::InsertChar('#');
        Key4, shift!(), +mode, @target; Action::InsertChar('$');
        Key5, shift!(), +mode, @target; Action::InsertChar('%');
        Key6, shift!(), +mode, @target; Action::InsertChar('^');
        Key7, shift!(), +mode, @target; Action::InsertChar('&');
        Key8, shift!(), +mode, @target; Action::InsertChar('*');
        Key9, shift!(), +mode, @target; Action::InsertChar('(');
        Key0, shift!(), +mode, @target; Action::InsertChar(')');
        
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

        Grave,      shift!(), +mode, @target; Action::InsertChar('~');
        Minus,      shift!(), +mode, @target; Action::InsertChar('_');
        Add,        shift!(), +mode, @target; Action::InsertChar('+');
        LBracket,   shift!(), +mode, @target; Action::InsertChar('{');
        RBracket,   shift!(), +mode, @target; Action::InsertChar('}');
        Backslash,  shift!(), +mode, @target; Action::InsertChar('|');
        Colon,      shift!(), +mode, @target; Action::InsertChar(':');
        Apostrophe, shift!(), +mode, @target; Action::InsertChar('"');
        Comma,      shift!(), +mode, @target; Action::InsertChar('<');
        Period,     shift!(), +mode, @target; Action::InsertChar('>');
        Slash,      shift!(), +mode, @target; Action::InsertChar('?');

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

pub fn bind_motion_selects(mode: Mode) -> Vec<KeyBinding> {
    bindings!(
        KeyBinding;

        Left,   +mode; motion!(Select Left);
        Right,  +mode; motion!(Select Right);
        Up,     +mode; motion!(Select Up);
        Down,   +mode; motion!(Select Down);
        H,      +mode; motion!(Select Left); 
        J,      +mode; motion!(Select Down); 
        K,      +mode; motion!(Select Up); 
        L,      +mode; motion!(Select Right); 

        W,      +mode; motion!(Select Right by Word);
        E,      +mode; motion!(Select RightEnd by Word); 
        Key4,   +mode; motion!(Select Last);

        W, shift!(), +mode; motion!(Select Right by Semantic); 
        Key5, shift!(), +mode; motion!(Select Bracket); 
        Key6, shift!(), +mode; motion!(Select FirstOccupied); 
    )
}

pub fn bind_motions(mode: Mode) -> Vec<KeyBinding> {
    bindings!(
        KeyBinding;

        Home,   +mode; motion!(Motion First); 
        End,    +mode; motion!(Motion Last); 
        Key0,   +mode; motion!(Motion First); 
        Key4,   shift!(), +mode; motion!(Motion Last); 
        Key5,   shift!(), +mode; motion!(Motion Bracket); 
        Key6,   shift!(), +mode; motion!(Motion FirstOccupied); 

        Up,     shift!(), +mode; motion!(Motion Up by Page); 
        Down,   shift!(), +mode; motion!(Motion Down by Page); 

        Left,   ctrl!(), +mode; motion!(Motion Left by Word); 
        Right,  ctrl!(), +mode; motion!(Motion Right by Word); 
        Left,   +mode; motion!(Motion Left); 
        Right,  +mode; motion!(Motion Right); 
        Up,     +mode; motion!(Motion Up); 
        Down,   +mode; motion!(Motion Down);
    )
}

pub fn default_key_bindings() -> Vec<KeyBinding> {
    let mut bindings = bindings!(
        KeyBinding;

        Escape, ~Mode::Normal; Action::SetMode(Mode::Normal);
        Escape, +Mode::SelectLine; Action::ClearSelection;
        Escape, +Mode::SelectBlock; Action::ClearSelection;
        Escape, +Mode::Select; Action::ClearSelection;
        I, +Mode::Normal; Action::SetMode(Mode::Insert);
        V, +Mode::Normal; Action::SetMode(Mode::Select);
        V, ctrl!(), +Mode::Normal; Action::SetMode(Mode::SelectBlock);
        V, shift!(), +Mode::Normal; Action::SetMode(Mode::SelectLine);
        R, shift!(), +Mode::Normal; Action::SetMode(Mode::Replace);
        Colon, shift!(), +Mode::Normal; Action::SetMode(Mode::Command);

        Back, +Mode::Command, @ActionTarget::StatusBar; motion!(Delete Left);
        Delete, +Mode::Command, @ActionTarget::StatusBar; motion!(Delete Right);
        Return, +Mode::Command; Action::ExecuteCommand;
        D,      shift!(), +Mode::Normal; motion!(Delete Last);
    );
    bindings.extend(bind_motions(Mode::Normal));
    bindings.extend(bind_motions(Mode::Insert));
    bindings.extend(bind_alpha_numeric(Mode::Command, ActionTarget::StatusBar));
    bindings.extend(bind_motion_selects(Mode::Select));
    bindings.extend(bind_motion_selects(Mode::SelectLine));
    bindings.extend(bind_motion_selects(Mode::SelectBlock));
    bindings.extend(bind_alpha_numeric(Mode::Insert, ActionTarget::FocusedView));

    bindings.extend(bindings!(
        KeyBinding;

        F1; Action::SetTheme(String::from("Solarized (dark)"));
        F2; Action::SetTheme(String::from("Solarized (light)"));
        F3; Action::SetTheme(String::from("InspiredGitHub"));
        F5; Action::ToggleLineNumbers;

        PageUp, ~Mode::Command; motion!(Motion Up by Page);
        PageDown, ~Mode::Command; motion!(Motion Down by Page);

        Return, +Mode::Normal; motion!(Motion Down), motion!(Motion FirstOccupied);
        Return, +Mode::Insert; Action::NewLine;
        Back, +Mode::Insert; motion!(Delete Left);
        Delete, +Mode::Insert; motion!(Delete Right);
        Back, ~Mode::Insert; motion!(Motion Left); 
        Delete, ~Mode::Insert; motion!(Motion Right); 
        Space, ~Mode::Insert; motion!(Motion Right); 
        
        Y, +Mode::Select; Action::Copy;
        Y, +Mode::SelectLine; Action::Copy;
        Y, +Mode::SelectBlock; Action::Copy;
        Copy, +Mode::Insert; Action::Copy;
        Cut, +Mode::Insert; Action::Cut;

        P, ~Mode::Insert; Action::Paste;
        Paste, +Mode::Insert; Action::Paste;
        
        U, ~Mode::Insert; Action::Undo;
        R, ctrl!(), ~Mode::Insert; Action::Redo;

        Tab, +Mode::Insert; Action::Indent;
        Tab, shift!(), +Mode::Insert; Action::Outdent;

        Minus, ctrl!(); Action::DecreaseFontSize;
        Subtract, ctrl!(); Action::DecreaseFontSize;
        Equals, ctrl!(); Action::IncreaseFontSize;
        Add, ctrl!(); Action::IncreaseFontSize;

        Up, shift!() | ctrl!(), +Mode::Insert; Action::AddCursor(Motion::Up);
        Down, shift!() | ctrl!(), +Mode::Insert; Action::AddCursor(Motion::Down);
    ));

    bindings
}
