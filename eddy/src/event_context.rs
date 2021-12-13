// Copyright 2018 The xi-editor Authors.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! A container for the state relevant to a single event.

use std::iter;
use std::cell::RefCell;
use std::sync::Arc;
use std::ops::Range;
use std::path::Path;
use std::time::Duration;

use rope::{Cursor, Rope, RopeDelta};

use crate::client::Client;
use crate::Action;
use crate::editor::Editor;
use crate::file::FileInfo;
use crate::line_offset::LineOffset;
use crate::selection::InsertDrift;
use crate::styles::ThemeStyleMap;
use crate::editor::{
    BufferId, ViewId, REWRAP_VIEW_IDLE_MASK,
};
use crate::view::View;
use crate::width_cache::WidthCache;
use crate::actions::Position;

// Maximum returned result from plugin get_data RPC.
pub const MAX_SIZE_LIMIT: usize = 1024 * 1024;

const LINE_ENDING: &str = "\n";

pub enum ActionTarget {
    View,
    Buffer,
    Special,
}

#[derive(Debug)]
pub enum EventError {}

/// Hover Item sent from Plugin to Core
#[derive(Debug, Clone)]
pub struct Hover {
    pub content: String,
    pub range: Option<Range<usize>>,
}

/// A collection of all the state relevant for handling a particular event.
///
/// This is created dynamically for each event that arrives to the core,
/// such as a user-initiated edit or style updates from a plugin.
pub struct EventContext<'a> {
    pub view_id: ViewId,
    pub buffer_id: BufferId,
    pub view: &'a RefCell<View>,
    pub editor: &'a RefCell<Editor>,
    pub info: Option<&'a FileInfo>,
    pub siblings: Vec<&'a RefCell<View>>,
    pub client: &'a Arc<Client>,
    pub style_map: &'a RefCell<ThemeStyleMap>,
    pub width_cache: &'a RefCell<WidthCache>,
    pub kill_ring: &'a RefCell<Rope>,
}

impl<'a> EventContext<'a> {

    /// Executes a closure with mutable references to the editor and the view,
    /// common in edit actions that modify the text.
    pub fn with_editor<R, F>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut Editor, &mut View, &mut Rope) -> R,
    {
        let mut kill_ring = self.kill_ring.borrow_mut();
        let mut editor = self.editor.borrow_mut();
        let mut view = self.view.borrow_mut();

        f(&mut editor, &mut view, &mut kill_ring)
    }

    /// Executes a closure with a mutable reference to the view and a reference
    /// to the current text. This is common to most edits that just modify
    /// selection or viewport state.
    fn with_view<R, F>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut View, &Rope) -> R,
    {
        f(&mut self.view.borrow_mut(), 
          self.editor.borrow().get_buffer())
    }

    pub fn do_edit(&mut self, action: Action) {
        self.dispatch_action(action);
        self.after_edit("core");
        self.render_if_needed();
    }

    fn determine_action_target(action: Action) -> ActionTarget {
        use self::Action::*;

        match action {
            NewView { .. }
            | Resize(..)
            | RequestLines(..)
            | RequestHover { .. }
            | Reindent => ActionTarget::Special,

            Delete(..)
            | InsertChars(..)
            | InsertNewline
            | InsertTab
            | Undo
            | Redo
            | Yank
            | Indent
            | Outdent
            | DuplicateLine
            | IncreaseNumber
            | DecreaseNumber 
            | Uppercase
            | Lowercase
            | Paste(..)
            | Replace(..) 
            | Repeat(..)
            | Duplicate(..) => ActionTarget::Buffer,

            SelectAll
            | SetMode( .. )
            | AddSelection(..)
            | CollapseSelections
            | GoToLine(..)
            | Move(..)
            | MoveSelection(..)
            | Gesture { .. }
            | Scroll { .. } => ActionTarget::View,
        }
    }

    fn dispatch_action(&mut self, action: Action) {
        let target = Self::determine_action_target(action.clone());

        match target {
            ActionTarget::View => {
                self.with_view(|view, text| view.do_edit(text, action));
                self.editor.borrow_mut().update_edit_type();
                if self.with_view(|v, t| v.needs_wrap_in_visible_region(t)) {
                    self.rewrap();
                }
            }
            ActionTarget::Buffer => {
                self.with_editor(|ed, view, k_ring| ed.do_edit(view, k_ring, action))
            }
            ActionTarget::Special => self.do_special(action),
        }
    }

    fn do_special(&mut self, cmd: Action) {
        match cmd {
            Action::Resize(size) => {
                self.with_view(|view, _| view.set_size(size));
                //if self.config.word_wrap {
                    self.update_wrap_settings(false);
                //}
            }
            Action::RequestLines(first, last) => {
                self.do_request_lines(first, last)
            }
            Action::RequestHover { request_id, position } => {
                self.do_request_hover(request_id, position)
            }
            Action::Reindent => self.do_reindent(),
            _ => unreachable!(),
        }
    }

    pub fn do_edit_sync(&mut self, cmd: Action) -> Result<Option<String>, EventError> {
        let result = match cmd {
            Action::Yank => Ok(self.with_editor(|ed, view, _| ed.do_cut(view))),
            _ => unreachable!(),
        };
        self.after_edit("core");
        self.render_if_needed();
        result
    }

    /// Commits any changes to the buffer, updating views and plugins as needed.
    /// This only updates internal state; it does not update the client.
    fn after_edit(&mut self, author: &str) {
        let edit_info = self.editor.borrow_mut().commit_delta();
        let (delta, last_text, drift) = match edit_info {
            Some(edit_info) => edit_info,
            None => return,
        };

        self.update_views(&self.editor.borrow(), &delta, &last_text, drift);
        self.update_plugins(&mut self.editor.borrow_mut(), delta, author);
    }

    fn update_views(&self, ed: &Editor, delta: &RopeDelta, last_text: &Rope, drift: InsertDrift) {
        let mut width_cache = self.width_cache.borrow_mut();
        let iter_views = iter::once(&self.view).chain(self.siblings.iter());

        iter_views.for_each(|view| {
            view.borrow_mut().after_edit(
                ed.get_buffer(),
                last_text,
                delta,
                self.client,
                &mut width_cache,
                drift,
            )
        });
    }

    fn update_plugins(&self, ed: &mut Editor, delta: RopeDelta, author: &str) {
        //TODO
        ed.update_edit_type();
    }

    /// Renders the view, if a render has not already been scheduled.
    pub fn render_if_needed(&mut self) {
        let needed = !self.view.borrow().has_pending_render();
        if needed {
            self.render()
        }
    }

    pub fn _finish_delayed_render(&mut self) {
        self.render();
        self.view.borrow_mut().set_has_pending_render(false);
    }

    /// Flushes any changes in the views out to the frontend.
    fn render(&mut self) {
        let ed = self.editor.borrow();
        //TODO: render other views
        self.view.borrow_mut().render_if_dirty(
            ed.get_buffer(),
            self.client,
            self.style_map,
            ed.get_layers().get_merged(),
            ed.is_pristine(),
        )
    }
}

/// Helpers related to specific commands.
///
/// Certain events and actions don't generalize well; handling these
/// requires access to particular combinations of state. We isolate such
/// special cases here.
impl<'a> EventContext<'a> {
    pub fn view_init(&mut self) {
        let wrap_width = 0; //self.config.wrap_width;
        let word_wrap = true; // self.config.word_wrap;

        self.with_view(|view, text| view.update_wrap_settings(text, wrap_width, word_wrap));
    }

    pub fn finish_init(&mut self) {
        // Rewrap and request a render.
        // This is largely similar to update_wrap_settings(), the only difference
        // being that the view is expected to be already initialized.
        self.rewrap();

        if self.view.borrow().needs_more_wrap() {
            self.schedule_rewrap();
        }

        self.with_view(|view, text| view.set_dirty(text));
        self.render()
    }

    pub fn after_save(&mut self, path: &Path) {
        self.editor.borrow_mut().set_pristine();
        self.with_view(|view, text| view.set_dirty(text));
        self.render()
    }

    /// Returns `true` if this was the last view
    pub fn close_view(&self) -> bool {
        // we probably want to notify plugins _before_ we close the view
        // TODO: determine what plugins we're stopping
        self.siblings.is_empty()
    }

    pub fn config_changed(&mut self) {
        unimplemented!();
    }

    pub fn reload(&mut self, text: Rope) {
        self.with_editor(|ed, _, _| ed.reload(text));
        self.after_edit("core");
        self.render();
    }

    /// Returns the text to be saved, appending a newline if necessary.
    pub fn text_for_save(&mut self) -> Rope {
        let mut rope = self.editor.borrow().get_buffer().clone();
        let rope_len = rope.len();

        if rope_len < 1 { //|| !self.config.save_with_newline {
            return rope;
        }

        let cursor = Cursor::new(&rope, rope.len());
        let has_newline_at_eof = match cursor.get_leaf() {
            Some((last_chunk, _)) => last_chunk.ends_with(LINE_ENDING),
            // The rope can't be empty, since we would have returned earlier if it was
            None => unreachable!(),
        };

        if !has_newline_at_eof {
            let line_ending = LINE_ENDING;
            rope.edit(rope_len.., line_ending);
            rope
        } else {
            rope
        }
    }

    /// Called after anything changes that effects word wrap, such as the size of
    /// the window or the user's wrap settings. `rewrap_immediately` should be `true`
    /// except in the resize case; during live resize we want to delay recalculation
    /// to avoid unnecessary work.
    fn update_wrap_settings(&mut self, rewrap_immediately: bool) {
        // TODO
        let wrap_width = 0; //self.config.wrap_width;
        let word_wrap = false; //self.config.word_wrap;
        self.with_view(|view, text| view.update_wrap_settings(text, wrap_width, word_wrap));
        if rewrap_immediately {
            self.rewrap();
            self.with_view(|view, text| view.set_dirty(text));
        }
        if self.view.borrow().needs_more_wrap() {
            self.schedule_rewrap();
        }
    }

    /// Tells the view to rewrap a batch of lines, if needed. This guarantees that
    /// the currently visible region will be correctly wrapped; the caller should
    /// check if additional wrapping is necessary and schedule that if so.
    fn rewrap(&mut self) {
        let ed = self.editor.borrow();
        let mut width_cache = self.width_cache.borrow_mut();
        self.view.borrow_mut().rewrap(
            ed.get_buffer(), 
            &mut width_cache, 
            self.client, 
            ed.get_layers().get_merged());
    }

    /// Does a rewrap batch, and schedules follow-up work if needed.
    pub fn do_rewrap_batch(&mut self) {
        self.rewrap();
        if self.view.borrow().needs_more_wrap() {
            self.schedule_rewrap();
        }
        self.render_if_needed();
    }

    fn schedule_rewrap(&self) {
        let view_id: usize = self.view_id.into();
        let token = REWRAP_VIEW_IDLE_MASK | view_id;
        self.client.schedule_idle(token);
    }

    fn do_request_lines(&mut self, first: usize, last: usize) {
        let ed = self.editor.borrow();
        self.view.borrow_mut().request_lines(
            ed.get_buffer(),
            self.client,
            self.style_map,
            ed.get_layers().get_merged(),
            first,
            last,
            ed.is_pristine(),
        )
    }

    fn selected_line_ranges(&mut self) -> Vec<(usize, usize)> {
        let mut prev_range: Option<Range<usize>> = None;
        let mut line_ranges = Vec::new();
        let ed = self.editor.borrow();
        // we send selection state to syntect in the form of a vec of line ranges,
        // so we combine overlapping selections to get the minimum set of ranges.
        for region in self.view.borrow().sel_regions().iter() {
            let start = ed.get_buffer().line_of_offset(region.min());
            let end = ed.get_buffer().line_of_offset(region.max()) + 1;
            let line_range = start..end;
            let prev = prev_range.take();
            match (prev, line_range) {
                (None, range) => prev_range = Some(range),
                (Some(ref prev), ref range) if range.start <= prev.end => {
                    let combined =
                        Range { start: prev.start.min(range.start), end: prev.end.max(range.end) };
                    prev_range = Some(combined);
                }
                (Some(prev), range) => {
                    line_ranges.push((prev.start, prev.end));
                    prev_range = Some(range);
                }
            }
        }

        if let Some(prev) = prev_range {
            line_ranges.push((prev.start, prev.end));
        }

        line_ranges
    }

    fn do_reindent(&mut self) {
        println!("TODO: Syntect reindentation handling");
    }

    fn do_request_hover(&mut self, request_id: usize, position: Option<Position>) {
        if let Some(position) = self.get_resolved_position(position) {
            //self.with_each_plugin(|p| p.get_hover(self.view_id, request_id, position))
        }
    }

    fn do_show_hover(&mut self, request_id: usize, hover: Result<Hover, EventError>) {
        match hover {
            Ok(hover) => {
                // TODO: Get Range from hover here and use it to highlight text
                self.client.show_hover(self.view_id, request_id, hover.content)
            }
            Err(err) => println!("Hover Response from Client Error {:?}", err),
        }
    }

    /// Gives the requested position in UTF-8 offset format to be sent to plugin
    /// If position is `None`, it tries to get the current Caret Position and use
    /// that instead
    fn get_resolved_position(&mut self, position: Option<Position>) -> Option<usize> {
        position
            .map(|p| self.with_view(|view, text| view.line_col_to_offset(text, p.line, p.column)))
            .or_else(|| self.view.borrow().get_caret_offset())
    }
}

