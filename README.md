# editor
A cross platform low-level code editor

## Frontend
This editor is a frontend for [xi-editor](https://github.com/xi-editor/xi-editor)

## Features
- Multi-platform windowing and input using [winit](https://github.com/rust-windowing/winit-rs).
- Vulkan rendering using [vulkano](https://github.com/vulkano-rs/vulkano).
- Widget system for rendering queue, currently implemented: primitives (coloured quads), text rendering (currently using [glyph-brush](https://github.com/alexheretic/glyph-brush) and the entire edit view is a widget (containing many internal widgets).
- Line numbers & gutter
- Themes
- Syntax highlighting
- Incomplete set of VIM-like modal key bindings (See below for better implementation)

## Plans (Likely to change...)
- Implement multi-view handling
- Implement LSP plugin
- Mouse interaction
- In-built LISP-like language to define keymapping to actions. Useful for complex key-bindings like VIM, still in the ideation phase...
