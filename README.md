# editor
A cross platform low-level code editor

![preview](https://imgur.com/download/EM5hMzA/)

## Frontend
This editor is a frontend for [xi-editor](https://github.com/xi-editor/xi-editor)

## Features
- Multi-platform windowing and input using [winit](https://github.com/rust-windowing/winit-rs).
- Vulkan rendering using [vulkano](https://github.com/vulkano-rs/vulkano).
- Widget system for rendering queue, currently implemented: 
  - primitive widgets (coloured quads)
  - text widget for glyph rendering (currently using [glyph-brush](https://github.com/alexheretic/glyph-brush) 
  - edit view is a widget (containing many internal widgets).
- Line numbers & gutter (Currently toggle with F5)
- Themes (Currently bound to F1, F2, and F3)
- Syntax highlighting
- Vim key bindings (more advanced keystrokes still a work in progress)

### Key Bindings (Vim)
Currently only basic bindings have been made, but the plan is to implement most of the main keybindings, will unlikely support
vimscript. Bindings are written using rust macros, below is an example:

See more at **src/events/bindings.rs**
```rust
...
bindings.extend(bindings!(
  Keybinding;
  
  F1; Action::SetTheme(String::from("Solarized (dark)"));
  F5; Action::ToggleLineNumbers;
  PageUp, ~Mode::Command; motion!(Motion Up by Page);
  Return, +Mode::Normal; motion!(Motion Down), motion!(Motion FirstOccupied);
  Delete, +Mode::Insert; motion!(Delete Right);
  ...
));
```
As you can see, the beginning of each key binding starts with the key trigger (ie. a function key, a letter etc.)
You can specify conditions, such as requiring a single mode with `+` to be active, or any other mode other then the one specified
with `~`. This is followed by an `Action`, and other macros exist such as motion to make life a little easier without chaining
a bunch of nested rust enumerables together.

### Preferences
As this is a frontend for Xi-Editor, preferences can be stored at `$HOME/.config/xi/preferences.xiconfig` in toml format.
Here is an example:

```toml
font_size = 14
tab_size = 2
translate_tabs_to_spaces = true
```

## Plans (Likely to change...)
- Implement multi-view handling
- Implement LSP plugin
- Mouse interaction
- In-built LISP-like language to define keymapping to actions. Useful for complex key-bindings like VIM, still in the ideation phase...
