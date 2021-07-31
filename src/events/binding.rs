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
        //println!("actions: {:?}", self.actions.clone());
        self.actions.clone()
    }

    pub fn get_trigger(&self) -> &T {
        &self.trigger
    }
}
macro_rules! motion {
    (
        $action:ident
        $motion:ident 
        $(by$quantity:ident)?
    ) => {{
        let mut _motion: Motion = Motion::$motion;
        let mut _quantity: Option<Quantity> = None;
        $(_quantity = Some(Quantity::$quantity(1));)*
        Action::$action((_motion, _quantity))
    }};
}
macro_rules! shift { () => {{ ModifiersState::SHIFT }}; }
macro_rules! ctrl { () => {{ ModifiersState::CTRL }}; }
macro_rules! mods_empty { () => {{ ModifiersState::empty() }}; }

macro_rules! key_binding {
    ($key:ident
     ,$mods:expr
     ,$mode:ident
     ,$target:expr
     ;$($action:expr),*
    ) => {{
        let mut _mods: ModifiersState = $mods;
        let _notmode = Mode::None;
        let mut _mode: Mode = Mode::$mode;
        let mut _target: ActionTarget = $target;
        let mut _actions: Vec<Action> = Vec::new();
        $(_actions.push($action);)*
        KeyBinding {
            trigger: Key::KeyCode($key),
            mods: _mods,
            mode: _mode,
            notmode: _notmode,
            target: _target,
            actions: _actions,
        }
    }};
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

macro_rules! bind_extended_motions {
    ($mode:ident, $action:ident, $target:expr) => {{
        vec![
            key_binding!(H,     mods_empty!(),  $mode, $target; motion!($action Left)),
            key_binding!(J,     mods_empty!(),  $mode, $target; motion!($action Down)),
            key_binding!(K,     mods_empty!(),  $mode, $target; motion!($action Up)),
            key_binding!(L,     mods_empty!(),  $mode, $target; motion!($action Right)),
            key_binding!(W,     mods_empty!(),  $mode, $target; motion!($action Right by Word)),
            key_binding!(E,     mods_empty!(),  $mode, $target; motion!($action RightEnd by Word)),
            key_binding!(W,     shift!(),       $mode, $target; motion!($action Right by Semantic)),
            key_binding!(Key0,  mods_empty!(),  $mode, $target; motion!($action First)),
            key_binding!(Key4,  shift!(),       $mode, $target; motion!($action Last)),
            key_binding!(Key5,  shift!(),       $mode, $target; motion!($action Bracket)),
            key_binding!(Key6,  shift!(),       $mode, $target; motion!($action FirstOccupied)),
            key_binding!(B,     mods_empty!(),  $mode, $target; motion!($action Left by Word)),
            key_binding!(B,     shift!(),       $mode, $target; motion!($action Left by Semantic)),
            key_binding!(H,     shift!(),       $mode, $target; motion!($action High)),
            key_binding!(M,     shift!(),       $mode, $target; motion!($action Middle)),
            key_binding!(L,     shift!(),       $mode, $target; motion!($action Low)),
        ]
    }};
}
macro_rules! bind_motions {
    ($mode:ident, $action:ident, $target:expr) => {{
       vec![
           key_binding!(Home,   mods_empty!(), $mode, $target; motion!($action First)), 
           key_binding!(End,    mods_empty!(), $mode, $target; motion!($action Last)), 

           key_binding!(Up,     shift!(), $mode, $target; motion!($action Up by Page)), 
           key_binding!(Down,   shift!(), $mode, $target; motion!($action Down by Page)), 

           key_binding!(Left,   ctrl!(), $mode, $target; motion!($action Left by Word)), 
           key_binding!(Right,  ctrl!(), $mode, $target; motion!($action Right by Word)), 
           key_binding!(Left,   mods_empty!(), $mode, $target; motion!($action Left)), 
           key_binding!(Right,  mods_empty!(), $mode, $target; motion!($action Right)), 
           key_binding!(Up,     mods_empty!(), $mode, $target; motion!($action Up)), 
           key_binding!(Down,   mods_empty!(), $mode, $target; motion!($action Down)),
        ]
    }};
}

pub fn default_mouse_bindings() -> Vec<MouseBinding> {
    bindings!(
        MouseBinding;
    )
}
pub fn bind_numeric(mode: Mode, target: ActionTarget) -> Vec<KeyBinding> {
    bindings!(KeyBinding;
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
    )
}
fn bind_symbols(mode: Mode, target: ActionTarget) -> Vec<KeyBinding> {
    bindings!(KeyBinding; 
        Grave,      shift!(), +mode, @target; Action::InsertChar('~');
        Minus,      shift!(), +mode, @target; Action::InsertChar('_');
        Equals,     shift!(), +mode, @target; Action::InsertChar('+');
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
        Space,      shift!(), +mode, @target; Action::InsertChar(' ');
    )
}
pub fn bind_alpha_numeric(mode: Mode, target: ActionTarget) -> Vec<KeyBinding> {
    let mut bindings = bindings!(KeyBinding;
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
    );
    bindings.extend(bind_numeric(mode, target));
    bindings.extend(bind_symbols(mode, target));

    bindings
}

#[inline]
fn command_mode_bindings() -> Vec<KeyBinding> {
    bindings!(KeyBinding;
        Back,   +Mode::Command, @ActionTarget::StatusBar; motion!(Delete Left);
        Delete, +Mode::Command, @ActionTarget::StatusBar; motion!(Delete Right);
        Return, +Mode::Command; Action::Execute;
    )
}

#[inline]
fn motion_mode_bindings() -> Vec<KeyBinding> {
    let mut bindings = bind_numeric(Mode::Normal, ActionTarget::StatusBar);
    for b in bindings.iter_mut() {
        b.actions.insert(0, Action::SetMode(Mode::Motion));
    }
    bindings.extend(bind_numeric(Mode::Motion, ActionTarget::StatusBar));

    bindings.extend(bindings!(KeyBinding;
        G, +Mode::Motion; Action::Motion((Motion::First, Some(Quantity::Line(0)))), motion!(Motion First), Action::SetMode(Mode::Normal);
        G, shift!(), +Mode::Motion, @ActionTarget::FocusedView; Action::InsertChar('G'), Action::Execute;
        Return, +Mode::Normal; Action::Execute;
    ));

    bindings
}
#[inline]
fn replace_mode_bindings() -> Vec<KeyBinding> {
    let mut replace_once_bindings = bind_alpha_numeric(Mode::ReplaceOnce, ActionTarget::FocusedView);
    for b in replace_once_bindings.iter_mut() {
        b.actions.push(Action::Delete((Motion::Right, None)));
        b.actions.push(Action::Motion((Motion::Left, None)));
        b.actions.push(Action::SetMode(Mode::Normal));
    }

    let mut replace_bindings = bind_alpha_numeric(Mode::Replace, ActionTarget::FocusedView);
    for b in replace_bindings.iter_mut() {
        b.actions.push(Action::Delete((Motion::Right, Some(Quantity::default()))));
    }

    replace_once_bindings.extend(replace_bindings);
    replace_once_bindings
}

pub fn default_key_bindings() -> Vec<KeyBinding> {
    let mut bindings = bindings!(KeyBinding;
        Escape, ~Mode::Normal; Action::ClearSelection, Action::SetMode(Mode::Normal);

        I, +Mode::Normal; Action::SetMode(Mode::Insert);
        V, +Mode::Normal; Action::SetMode(Mode::Select);
        G, +Mode::Normal; Action::SetMode(Mode::Motion);
        G, shift!(), +Mode::Normal; Action::Motion((Motion::Last, Some(Quantity::Line(0)))), motion!(Motion First), Action::SetMode(Mode::Normal);
        V, ctrl!(), +Mode::Normal; Action::SetMode(Mode::SelectBlock);
        V, shift!(), +Mode::Normal; Action::SetMode(Mode::SelectLine);
        R, +Mode::Normal; Action::SetMode(Mode::ReplaceOnce);
        R, shift!(), +Mode::Normal; Action::SetMode(Mode::Replace);
        Colon, shift!(), +Mode::Normal; Action::SetMode(Mode::Command);

        A,      +Mode::Normal; motion!(Motion Right), Action::SetMode(Mode::Insert);
        A,      shift!(), +Mode::Normal; motion!(Motion Last), Action::SetMode(Mode::Insert);
        D,      +Mode::Normal; Action::SetMode(Mode::Delete);
        D,      +Mode::Delete; motion!(Motion First), motion!(Select Last), Action::Cut, motion!(Delete Left), Action::SetMode(Mode::Normal);
        D,      shift!(), +Mode::Normal; motion!(Select Last), motion!(Delete Left);
        X,      +Mode::Normal; motion!(Select Right), Action::Cut;
        X,      shift!(), +Mode::Normal; motion!(Select Left), Action::Cut;
    );
    bindings.extend(bind_motions!(Normal, Motion, ActionTarget::FocusedView));
    bindings.extend(bind_motions!(Insert, Motion, ActionTarget::FocusedView));
    bindings.extend(bind_motions!(Select, Select, ActionTarget::FocusedView));
    bindings.extend(bind_motions!(SelectLine, Select, ActionTarget::FocusedView));
    bindings.extend(bind_motions!(SelectBlock, Select, ActionTarget::FocusedView));
    bindings.extend(bind_motions!(Delete, Delete, ActionTarget::FocusedView));
    bindings.extend(bind_motions!(Command, Motion, ActionTarget::StatusBar));
    bindings.extend(bind_extended_motions!(Normal, Motion, ActionTarget::FocusedView));
    bindings.extend(bind_extended_motions!(Select, Select, ActionTarget::FocusedView));
    bindings.extend(bind_extended_motions!(SelectLine, Select, ActionTarget::FocusedView));
    bindings.extend(bind_extended_motions!(SelectBlock, Select, ActionTarget::FocusedView));
    bindings.extend(bind_extended_motions!(Delete, Delete, ActionTarget::FocusedView));
    bindings.extend(bind_alpha_numeric(Mode::Command, ActionTarget::StatusBar));
    bindings.extend(bind_alpha_numeric(Mode::Insert, ActionTarget::FocusedView));
    bindings.extend(command_mode_bindings());
    bindings.extend(motion_mode_bindings());
    bindings.extend(replace_mode_bindings());

    bindings.extend(bindings!(KeyBinding;
        F1; Action::SetTheme(String::from("Solarized (dark)"));
        F2; Action::SetTheme(String::from("Solarized (light)"));
        F3; Action::SetTheme(String::from("InspiredGitHub"));
        F5; Action::ToggleLineNumbers;

        PageUp, +Mode::Normal; motion!(Motion Up by Page);
        PageDown, +Mode::Normal; motion!(Motion Down by Page);
        PageUp, +Mode::Select; motion!(Select Up by Page);
        PageDown, +Mode::Select; motion!(Select Down by Page);
        PageUp, +Mode::SelectLine; motion!(Select Up by Page);
        PageDown, +Mode::SelectLine; motion!(Select Down by Page);
        PageUp, +Mode::SelectBlock; motion!(Select Up by Page);
        PageDown, +Mode::SelectBlock; motion!(Select Down by Page);

        Return, +Mode::Normal; motion!(Motion Down), motion!(Motion FirstOccupied);
        Return, +Mode::Insert; Action::NewLine;
        Back, +Mode::Insert; motion!(Delete Left);
        Delete, +Mode::Insert; motion!(Delete Right);
        Back, ~Mode::Insert; motion!(Motion Left); 
        Delete, ~Mode::Insert; motion!(Motion Right); 
        Space, ~Mode::Insert; motion!(Motion Right); 
        Space, shift!(), ~Mode::Insert; motion!(Motion Right); 
        
        Y, +Mode::Select; Action::Cut;
        Y, +Mode::SelectLine; Action::Cut;
        Y, +Mode::SelectBlock; Action::Cut;
        Copy, +Mode::Insert; Action::Copy;
        Cut, +Mode::Insert; Action::Cut;

        P, +Mode::Normal; Action::Paste;
        Paste, +Mode::Insert; Action::Paste;
        
        U, +Mode::Normal; Action::Undo;
        R, ctrl!(), +Mode::Normal; Action::Redo;

        Tab, +Mode::Insert; Action::Indent;
        Tab, shift!(), +Mode::Insert; Action::Outdent;

        Minus, ctrl!(); Action::DecreaseFontSize;
        NumpadSubtract, ctrl!(); Action::DecreaseFontSize;
        Equals, ctrl!(); Action::IncreaseFontSize;

        Up, shift!() | ctrl!(), +Mode::Insert; Action::AddCursor(Motion::Up);
        Down, shift!() | ctrl!(), +Mode::Insert; Action::AddCursor(Motion::Down);
        
        O,  +Mode::Normal; motion!(Motion Last), Action::NewLine, Action::SetMode(Mode::Insert);
        O,  shift!(), +Mode::Normal; motion!(Motion Up), motion!(Motion Last), Action::NewLine, Action::SetMode(Mode::Insert);
    ));

    bindings
}
