// Copyright 2016 The xi-editor Authors.
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

use std::borrow::Cow;
use std::collections::BTreeSet;

use rope::{
    diff::{Diff, LineHashDiff},
    engine::{Engine, RevId, RevToken},
    rope::count_newlines,
    spans::SpansBuilder,
    DeltaBuilder, Interval, LinesMetric, Rope, RopeDelta, Transformer,
};

use crate::annotations::{AnnotationType, Annotations};
use crate::edit_ops::{self, IndentDirection};
use super::line_offset::{LineOffset, LogicalLines};
use super::{Action, Motion, Quantity};
use super::selection::{InsertDrift, SelRegion};
use super::layers::Layers;
use super::view::View;
use super::styles::ThemeStyleMap;

// TODO This could go much higher without issue but while developing it is
// better to keep it low to expose bugs in the GC during casual testing.

const MAX_UNDOS: usize = 300;
pub(crate) const RENDER_VIEW_IDLE_MASK: usize = 1 << 25;
pub(crate) const REWRAP_VIEW_IDLE_MASK: usize = 1 << 26;
pub(crate) const FIND_VIEW_IDLE_MASK: usize = 1 << 27;

pub type PluginId = usize;

/// ViewIds are the primary means of routing messages between
/// xi-core and a client view.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ViewId(pub usize);

/// BufferIds uniquely identify open buffers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BufferId(pub usize);

impl From<usize> for ViewId {
    fn from(src: usize) -> ViewId {
        ViewId(src)
    }
}

impl From<ViewId> for usize {
    fn from(src: ViewId) -> usize {
        src.0
    }
}

impl Iterator for ViewId
{
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.0 + 1)
    }
}

pub struct ScopeSpan {
    scope_id: u32,
    start: usize,
    end: usize,
}

pub struct DataSpan {
    pub start: usize,
    pub end: usize,
    pub data: String,
}

static SURROUNDING_PAIRS: &[(&str, &str)] = &[
    ("\"", "\""),
    ("'", "'"),
    ("{", "}"),
    ("[", "]"),
    ("(", ")"),
    ("<", ">"),
];

pub struct Editor {
    /// The contents of the buffer.
    text: Rope,
    /// The CRDT engine, which tracks edit history and manages concurrent edits.
    engine: Engine,

    /// The most recent revision.
    last_rev_id: RevId,
    /// The revision of the last save.
    pristine_rev_id: RevId,
    undo_group_id: usize,
    /// Undo groups that may still be toggled
    live_undos: Vec<usize>,
    /// The index of the current undo; subsequent undos are currently 'undone'
    /// (but may be redone)
    cur_undo: usize,
    /// undo groups that are undone
    undos: BTreeSet<usize>,
    /// undo groups that are no longer live and should be gc'ed
    gc_undos: BTreeSet<usize>,
    force_undo_group: bool,

    this_edit_type: EditType,
    last_edit_type: EditType,

    revs_in_flight: usize,

    last_synced_rev: RevId,

    layers: Layers,
}

impl Editor {
    /// Creates a new `Editor` with a new empty buffer.
    pub fn new() -> Editor {
        Self::with_text("")
    }

    /// Creates a new `Editor`, loading text into a new buffer.
    pub fn with_text<T: Into<Rope>>(text: T) -> Editor {
        let engine = Engine::new(text.into());
        let buffer = engine.get_head().clone();
        let last_rev_id = engine.get_head_rev_id();

        Editor {
            text: buffer,
            engine,
            last_rev_id,
            pristine_rev_id: last_rev_id,
            undo_group_id: 1,
            // GC only works on undone edits or prefixes of the visible edits,
            // but initial file loading can create an edit with undo group 0,
            // so we want to collect that as part of the prefix.
            live_undos: vec![0],
            cur_undo: 1,
            undos: BTreeSet::new(),
            gc_undos: BTreeSet::new(),
            force_undo_group: false,
            last_edit_type: EditType::Other,
            this_edit_type: EditType::Other,
            layers: Layers::default(),
            revs_in_flight: 0,
            last_synced_rev: last_rev_id,
        }
    }

    pub(crate) fn get_buffer(&self) -> &Rope {
        &self.text
    }

    pub(crate) fn get_layers(&self) -> &Layers {
        &self.layers
    }

    pub(crate) fn get_layers_mut(&mut self) -> &mut Layers {
        &mut self.layers
    }

    pub(crate) fn get_head_rev_token(&self) -> u64 {
        self.engine.get_head_rev_id().token()
    }

    pub(crate) fn get_edit_type(&self) -> EditType {
        self.this_edit_type
    }

    pub(crate) fn get_active_undo_group(&self) -> usize {
        *self.live_undos.last().unwrap_or(&0)
    }

    pub(crate) fn update_edit_type(&mut self) {
        self.last_edit_type = self.this_edit_type;
        self.this_edit_type = EditType::Other
    }

    pub(crate) fn set_pristine(&mut self) {
        self.pristine_rev_id = self.engine.get_head_rev_id();
    }

    pub(crate) fn is_pristine(&self) -> bool {
        self.engine.is_equivalent_revision(self.pristine_rev_id, self.engine.get_head_rev_id())
    }

    /// Set whether or not edits are forced into the same undo group rather than being split by
    /// their EditType.
    ///
    /// This is used for things such as recording playback, where you don't want the
    /// individual events to be undoable, but instead the entire playback should be.
    pub(crate) fn set_force_undo_group(&mut self, force_undo_group: bool) {
        self.force_undo_group = force_undo_group;
    }

    /// Sets this Editor's contents to `text`, preserving undo state and cursor
    /// position when possible.
    pub fn reload(&mut self, text: Rope) {
        let delta = LineHashDiff::compute_delta(self.get_buffer(), &text);
        self.add_delta(delta);
        self.set_pristine();
    }

    // each outstanding plugin edit represents a rev_in_flight.
    pub fn increment_revs_in_flight(&mut self) {
        self.revs_in_flight += 1;
    }

    // GC of CRDT engine is deferred until all plugins have acknowledged the new rev,
    // so when the ack comes back, potentially trigger GC.
    pub fn dec_revs_in_flight(&mut self) {
        self.revs_in_flight -= 1;
        self.gc_undos();
    }

    /// Applies a delta to the text, and updates undo state.
    ///
    /// Records the delta into the CRDT engine so that it can be undone. Also
    /// contains the logic for merging edits into the same undo group. At call
    /// time, self.this_edit_type should be set appropriately.
    ///
    /// This method can be called multiple times, accumulating deltas that will
    /// be committed at once with `commit_delta`. Note that it does not update
    /// the views. Thus, view-associated state such as the selection and line
    /// breaks are to be considered invalid after this method, until the
    /// `commit_delta` call.
    fn add_delta(&mut self, delta: RopeDelta) {
        let head_rev_id = self.engine.get_head_rev_id();
        let undo_group = self.calculate_undo_group();
        self.last_edit_type = self.this_edit_type;
        let priority = 0x10000;
        self.engine.edit_rev(priority, undo_group, head_rev_id.token(), delta);
        self.text = self.engine.get_head().clone();
    }

    pub(crate) fn calculate_undo_group(&mut self) -> usize {
        let has_undos = !self.live_undos.is_empty();
        let force_undo_group = self.force_undo_group;
        let is_unbroken_group = !self.this_edit_type.breaks_undo_group(self.last_edit_type);

        if has_undos && (force_undo_group || is_unbroken_group) {
            *self.live_undos.last().unwrap()
        } else {
            let undo_group = self.undo_group_id;
            self.gc_undos.extend(&self.live_undos[self.cur_undo..]);
            self.live_undos.truncate(self.cur_undo);
            self.live_undos.push(undo_group);
            if self.live_undos.len() <= MAX_UNDOS {
                self.cur_undo += 1;
            } else {
                self.gc_undos.insert(self.live_undos.remove(0));
            }
            self.undo_group_id += 1;
            undo_group
        }
    }

    /// Commits the current delta. If the buffer has changed, returns
    /// a 3-tuple containing the delta representing the changes, the previous
    /// buffer, and an `InsertDrift` enum describing the correct selection update
    /// behaviour.
    pub(crate) fn commit_delta(&mut self) -> Option<(RopeDelta, Rope, InsertDrift)> {
        if self.engine.get_head_rev_id() == self.last_rev_id {
            return None;
        }

        let last_token = self.last_rev_id.token();
        let delta = self.engine.try_delta_rev_head(last_token).expect("last_rev not found");
        // TODO (performance): it's probably quicker to stash last_text
        // rather than resynthesize it.
        let last_text = self.engine.get_rev(last_token).expect("last_rev not found");

        // Transpose can rotate characters inside of a selection; this is why it's an Inside edit.
        // Surround adds characters on either side of a selection, that's why it's an Outside edit.
        let drift = match self.this_edit_type {
            EditType::Transpose => InsertDrift::Inside,
            EditType::Surround => InsertDrift::Outside,
            _ => InsertDrift::Default,
        };
        self.layers.update_all(&delta);

        self.last_rev_id = self.engine.get_head_rev_id();
        Some((delta, last_text, drift))
    }

    /// Attempts to find the delta from head for the given `RevToken`. Returns
    /// `None` if the revision is not found, so this result should be checked if
    /// the revision is coming from a plugin.
    pub(crate) fn delta_rev_head(&self, target_rev_id: RevToken) -> Option<RopeDelta> {
        self.engine.try_delta_rev_head(target_rev_id).ok()
    }

    fn gc_undos(&mut self) {
        if self.revs_in_flight == 0 && !self.gc_undos.is_empty() {
            self.engine.gc(&self.gc_undos);
            self.undos = &self.undos - &self.gc_undos;
            self.gc_undos.clear();
        }
    }

    pub fn merge_new_state(&mut self, new_engine: Engine) {
        self.engine.merge(&new_engine);
        self.text = self.engine.get_head().clone();
        // TODO: better undo semantics. This only implements separate undo
        // histories for low concurrency.
        self.undo_group_id = self.engine.max_undo_group_id() + 1;
        self.last_synced_rev = self.engine.get_head_rev_id();
        self.commit_delta();
    }

    /// See `Engine::set_session_id`. Only useful for Fuchsia sync.
    pub fn set_session_id(&mut self, session: (u64, u32)) {
        self.engine.set_session_id(session);
    }

    fn do_insert(&mut self, view: &View, chars: &str) {
        let pair_search = SURROUNDING_PAIRS.iter().find(|pair| pair.0 == chars);
        let caret_exists = view.sel_regions().iter().any(|region| region.is_caret());
        if let (Some(pair), false) = (pair_search, caret_exists) {
            self.this_edit_type = EditType::Surround;
            self.add_delta(edit_ops::surround(
                &self.text,
                view.sel_regions(),
                pair.0.to_string(),
                pair.1.to_string(),
            ));
        } else {
            self.this_edit_type = EditType::InsertChars;
            self.add_delta(edit_ops::insert(&self.text, view.sel_regions(), chars));
        }
    }

    fn do_paste(&mut self, view: &View, chars: &str) {
        if view.sel_regions().len() == 1 || view.sel_regions().len() != count_lines(chars) {
            self.add_delta(edit_ops::insert(&self.text, view.sel_regions(), chars));
        } else {
            let mut builder = DeltaBuilder::new(self.text.len());
            for (sel, line) in view.sel_regions().iter().zip(chars.lines()) {
                let iv = Interval::new(sel.min(), sel.max());
                builder.replace(iv, line.into());
            }
            self.add_delta(builder.build());
        }
    }

    pub(crate) fn do_cut(&mut self, view: &mut View) -> Option<String> {
        let result = self.do_copy(view);
        let delta = edit_ops::delete_sel_regions(&self.text, &view.sel_regions());
        if !delta.is_identity() {
            self.this_edit_type = EditType::Delete;
            self.add_delta(delta);
        }
        result
    }

    pub(crate) fn do_copy(&self, view: &View) -> Option<String> {
        if let Some(val) = edit_ops::extract_sel_regions(&self.text, view.sel_regions()) {
            Some(val.into_owned())
        } else {
            None
        }
    }

    fn do_undo(&mut self) {
        if self.cur_undo > 1 {
            self.cur_undo -= 1;
            assert!(self.undos.insert(self.live_undos[self.cur_undo]));
            self.this_edit_type = EditType::Undo;
            self.update_undos();
        }
    }

    fn do_redo(&mut self) {
        if self.cur_undo < self.live_undos.len() {
            assert!(self.undos.remove(&self.live_undos[self.cur_undo]));
            self.cur_undo += 1;
            self.this_edit_type = EditType::Redo;
            self.update_undos();
        }
    }

    fn update_undos(&mut self) {
        self.engine.undo(self.undos.clone());
        self.text = self.engine.get_head().clone();
    }

    fn do_delete_by_movement(
        &mut self,
        view: &View,
        movement: Motion,
        quantity: Quantity,
        save: bool,
        kill_ring: &mut Rope,
    ) {
        let (delta, rope) = edit_ops::delete_by_movement(
            &self.text,
            view.sel_regions(),
            view.get_lines(),
            movement,
            quantity,
            view.scroll_height(),
            save,
        );
        if let Some(rope) = rope {
            *kill_ring = rope;
        }
        if !delta.is_identity() {
            self.this_edit_type = EditType::Delete;
            self.add_delta(delta);
        }
    }

    fn do_delete_backward(&mut self, view: &View) {
        let delta = edit_ops::delete_backward(&self.text, view.sel_regions());
        if !delta.is_identity() {
            self.this_edit_type = EditType::Delete;
            self.add_delta(delta);
        }
    }

    fn do_transform_text<F: Fn(&str) -> String>(&mut self, view: &View, transform_function: F) {
        let delta = edit_ops::transform_text(&self.text, view.sel_regions(), transform_function);
        if !delta.is_identity() {
            self.this_edit_type = EditType::Other;
            self.add_delta(delta);
        }
    }

    fn do_capitalize_text(&mut self, view: &mut View) {
        let (delta, final_selection) = edit_ops::capitalize_text(&self.text, view.sel_regions());
        if !delta.is_identity() {
            self.this_edit_type = EditType::Other;
            self.add_delta(delta);
        }

        // at the end of the transformation carets are located at the end of the words that were
        // transformed last in the selections
        view.collapse_selections(&self.text);
        view.set_selection(&self.text, final_selection);
    }

    fn do_modify_indent(&mut self, view: &View, direction: IndentDirection) {
        let delta = edit_ops::modify_indent(&self.text, view.sel_regions(), direction);
        self.add_delta(delta);
        self.this_edit_type = match direction {
            IndentDirection::In => EditType::InsertChars,
            IndentDirection::Out => EditType::Delete,
        }
    }

    fn do_insert_newline(&mut self, view: &View) {
        let delta = edit_ops::insert_newline(&self.text, view.sel_regions());
        self.add_delta(delta);
        self.this_edit_type = EditType::InsertNewline;
    }

    fn do_insert_tab(&mut self, view: &View) {
        let regions = view.sel_regions();
        let delta = edit_ops::insert_tab(&self.text, regions);

        // if we indent multiple regions or multiple lines,
        // we treat this as an indentation adjustment; otherwise it is
        // just inserting text.
        let condition = regions
            .first()
            .map(|x| LogicalLines.get_line_range(&self.text, x).len() > 1)
            .unwrap_or(false);

        self.add_delta(delta);
        self.this_edit_type =
            if regions.len() > 1 || condition { EditType::Indent } else { EditType::InsertChars };
    }

    fn do_yank(&mut self, view: &View, kill_ring: &Rope) {
        // TODO: if there are multiple cursors and the number of newlines
        // is one less than the number of cursors, split and distribute one
        // line per cursor.
        let delta = edit_ops::insert(&self.text, view.sel_regions(), kill_ring.clone());
        self.add_delta(delta);
    }

    fn do_duplicate_line(&mut self, view: &View) {
        let delta = edit_ops::duplicate_line(&self.text, view.sel_regions());
        self.add_delta(delta);
        self.this_edit_type = EditType::Other;
    }

    fn do_change_number<F: Fn(i128) -> Option<i128>>(
        &mut self,
        view: &View,
        transform_function: F,
    ) {
        let delta = edit_ops::change_number(&self.text, view.sel_regions(), transform_function);
        if !delta.is_identity() {
            self.this_edit_type = EditType::Other;
            self.add_delta(delta);
        }
    }

    pub(crate) fn do_edit(
        &mut self,
        view: &mut View,
        kill_ring: &mut Rope,
        cmd: Action,
    ) {
        match cmd {
            Action::Delete(motion, quantity) => {
                match motion {
                    Motion::Backward => self.do_delete_backward(view),
                    _ => self.do_delete_by_movement(view, motion, quantity, false, kill_ring),
                }
            },
            Action::Undo => self.do_undo(),
            Action::Redo => self.do_redo(),
            Action::Uppercase => self.do_transform_text(view, |s| s.to_uppercase()),
            Action::Lowercase => self.do_transform_text(view, |s| s.to_lowercase()),
            Action::Indent => self.do_modify_indent(view, IndentDirection::In),
            Action::Outdent => self.do_modify_indent(view, IndentDirection::Out),
            Action::InsertNewline => self.do_insert_newline(view),
            Action::InsertTab => self.do_insert_tab(view),
            Action::InsertChars(chars) => self.do_insert(view, &chars),
            Action::Paste(chars) => self.do_paste(view, &chars),
            Action::Yank => self.do_yank(view, kill_ring),
            Action::DuplicateLine => self.do_duplicate_line(view),
            Action::Duplicate(quantity) => match quantity {
                Quantity::Line => self.do_duplicate_line(view),
                _ => unimplemented!(),
            },
            Action::IncreaseNumber => self.do_change_number(view, |s| s.checked_add(1)),
            Action::DecreaseNumber => self.do_change_number(view, |s| s.checked_sub(1)),
            _ => unimplemented!(),
        }
    }

    pub fn theme_changed(&mut self, style_map: &ThemeStyleMap) {
        self.layers.theme_changed(style_map);
    }

    pub fn plugin_n_lines(&self) -> usize {
        self.text.measure::<LinesMetric>() + 1
    }

    pub fn update_spans(
        &mut self,
        view: &mut View,
        plugin: PluginId,
        start: usize,
        len: usize,
        spans: Vec<ScopeSpan>,
        rev: RevToken,
    ) {
        // TODO: more protection against invalid input
        let mut start = start;
        let mut end_offset = start + len;
        let mut sb = SpansBuilder::new(len);
        for span in spans {
            sb.add_span(Interval::new(span.start, span.end), span.scope_id);
        }
        let mut spans = sb.build();
        if rev != self.engine.get_head_rev_id().token() {
            if let Ok(delta) = self.engine.try_delta_rev_head(rev) {
                let mut transformer = Transformer::new(&delta);
                let new_start = transformer.transform(start, false);
                if !transformer.interval_untouched(Interval::new(start, end_offset)) {
                    spans = spans.transform(start, end_offset, &mut transformer);
                }
                start = new_start;
                end_offset = transformer.transform(end_offset, true);
            } else {
                panic!("Revision {} not found", rev);
            }
        }
        let iv = Interval::new(start, end_offset);
        self.layers.update_layer(plugin, iv, spans);
        view.invalidate_styles(&self.text, start, end_offset);
    }

    pub fn update_annotations(
        &mut self,
        view: &mut View,
        plugin: PluginId,
        start: usize,
        len: usize,
        annotation_spans: Vec<DataSpan>,
        annotation_type: AnnotationType,
        rev: RevToken,
    ) {
        let mut start = start;
        let mut end_offset = start + len;
        let mut sb = SpansBuilder::new(len);
        for span in annotation_spans {
            sb.add_span(Interval::new(span.start, span.end), span.data);
        }
        let mut spans = sb.build();
        if rev != self.engine.get_head_rev_id().token() {
            if let Ok(delta) = self.engine.try_delta_rev_head(rev) {
                let mut transformer = Transformer::new(&delta);
                let new_start = transformer.transform(start, false);
                if !transformer.interval_untouched(Interval::new(start, end_offset)) {
                    spans = spans.transform(start, end_offset, &mut transformer);
                }
                start = new_start;
                end_offset = transformer.transform(end_offset, true);
            } else {
                panic!("Revision {} not found", rev);
            }
        }
        let iv = Interval::new(start, end_offset);
        view.update_annotations(plugin, iv, Annotations { items: spans, annotation_type });
    }

    pub(crate) fn get_rev(&self, rev: RevToken) -> Option<Cow<Rope>> {
        let text_cow = if rev == self.engine.get_head_rev_id().token() {
            Cow::Borrowed(&self.text)
        } else {
            match self.engine.get_rev(rev) {
                None => return None,
                Some(text) => Cow::Owned(text),
            }
        };

        Some(text_cow)
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum EditType {
    /// A catchall for edits that don't fit elsewhere, and which should
    /// always have their own undo groups; used for things like cut/copy/paste.
    Other,
    /// An insert from the keyboard/IME (not a paste or a yank).
    InsertChars,
    InsertNewline,
    /// An indentation adjustment.
    Indent,
    Delete,
    Undo,
    Redo,
    Transpose,
    Surround,
}

impl EditType {
    /// Checks whether a new undo group should be created between two edits.
    fn breaks_undo_group(self, previous: EditType) -> bool {
        self == EditType::Other || self == EditType::Transpose || self != previous
    }
}

fn last_selection_region(regions: &[SelRegion]) -> Option<&SelRegion> {
    for region in regions.iter().rev() {
        if !region.is_caret() {
            return Some(region);
        }
    }
    None
}

/// Counts the number of lines in the string, not including any trailing newline.
fn count_lines(s: &str) -> usize {
    let mut newlines = count_newlines(s);
    if s.as_bytes().last() == Some(&0xa) {
        newlines -= 1;
    }
    1 + newlines
}

