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

use std::cmp::max;
use std::sync::{
    Mutex,
    Arc,
};

use super::{
    annotations::{AnnotationStore, Annotations, ToAnnotation},
    actions::Action,
    actions::GestureType,
    Mode,
    Motion, 
    Quantity,
    ViewId, 
    BufferId,
    line_cache_shadow::{self, LineCacheShadow, RenderPlan, RenderTactic},
    line_offset::LineOffset,
    linewrap::{InvalLines, Lines, VisualLine, WrapWidth},
    movement::{region_movement, selection_movement},
    selection::{Affinity, InsertDrift, SelRegion, Selection},
    styles::{Style, ThemeStyleMap},
    width_cache::WidthCache,
    words::WordCursor,
    client::{
        Client,
        Update,
        UpdateOp,
    },
    line_cache::Line as LineUpdate,
};
use rope::{
    spans::Spans,
    Interval, Rope, RopeDelta,
};

type StyleMap = Arc<Mutex<ThemeStyleMap>>;
type PluginId = usize;

/// A view to a buffer. It is the buffer plus additional information
/// like line breaks and selection state.
pub struct View {
    view_id: ViewId,
    buffer_id: BufferId,

    /// Tracks whether this view has been scheduled to render.
    /// We attempt to reduce duplicate renders by setting a small timeout
    /// after an edit is applied, to allow batching with any plugin updates.
    pending_render: bool,
    size: Size,
    /// The selection state for this view. Invariant: non-empty.
    selection: Selection,

    drag_state: Option<DragState>,

    /// vertical scroll position
    first_line: usize,
    /// height of visible portion
    height: usize,
    lines: Lines,

    /// Front end's line cache state for this view. See the `LineCacheShadow`
    /// description for the invariant.
    lc_shadow: LineCacheShadow,

    /// New offset to be scrolled into position after an edit.
    scroll_to: Option<usize>,

    /// Annotations provided by plugins.
    annotations: AnnotationStore,

    mode: Mode,
}

/// Contains replacement string and replace options.
#[derive(Debug, Default, PartialEq, Clone)]
pub struct Replace {
    /// Replacement string.
    pub chars: String,
    pub preserve_case: bool,
}

/// A size, in pixel units (not display pixels).
#[derive(Debug, Default, PartialEq, Clone)]
pub struct Size {
    pub width: f64,
    pub height: f64,
}

/// State required to resolve a drag gesture into a selection.
struct DragState {
    /// All the selection regions other than the one being dragged.
    base_sel: Selection,

    /// Start of the region selected when drag was started (region is
    /// assumed to be forward).
    min: usize,

    /// End of the region selected when drag was started.
    max: usize,

    quantity: Quantity,
}


impl View {
    pub fn new(view_id: ViewId, buffer_id: BufferId) -> View {
        View {
            view_id,
            buffer_id,
            pending_render: false,
            selection: SelRegion::caret(0).into(),
            scroll_to: Some(0),
            size: Size::default(),
            drag_state: None,
            first_line: 0,
            height: 10,
            lines: Lines::default(),
            lc_shadow: LineCacheShadow::default(),
            annotations: AnnotationStore::new(),
            mode: Mode::Normal,
        }
    }

    pub fn get_buffer_id(&self) -> BufferId {
        self.buffer_id
    }

    pub fn get_view_id(&self) -> ViewId {
        self.view_id
    }

    pub fn get_mode(&self) -> Mode {
        self.mode
    }

    pub fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
    }

    pub fn get_lines(&self) -> &Lines {
        &self.lines
    }

    pub fn set_has_pending_render(&mut self, pending: bool) {
        self.pending_render = pending
    }

    pub fn has_pending_render(&self) -> bool {
        self.pending_render
    }

    pub fn update_wrap_settings(&mut self, text: &Rope, wrap_cols: usize, word_wrap: bool) {
        let wrap_width = match (word_wrap, wrap_cols) {
            (true, _) => WrapWidth::Width(self.size.width),
            (false, 0) => WrapWidth::None,
            (false, cols) => WrapWidth::Bytes(cols),
        };
        self.lines.set_wrap_width(text, wrap_width);
    }

    pub fn needs_more_wrap(&self) -> bool {
        !self.lines.is_converged()
    }

    pub fn needs_wrap_in_visible_region(&self, text: &Rope) -> bool {
        if self.lines.is_converged() {
            false
        } else {
            let visible_region = self.interval_of_visible_region(text);
            self.lines.interval_needs_wrap(visible_region)
        }
    }

    pub fn do_edit(&mut self, text: &Rope, cmd: Action) {
        use self::Action::*;
        match cmd {
            Action::SetMode(mode) => self.set_mode(mode),
            Move(motion, quantity) => self.do_move(text, motion, quantity, false),
            MoveSelection(motion, quantity) => self.do_move(text, motion, quantity, true),
            SelectAll => self.select_all(text),
            Scroll(range) => self.set_scroll(range.start, range.end),
            AddSelection(motion) => match motion {
                Motion::Above => self.add_selection_by_movement(text, Motion::Above, Quantity::Character),
                Motion::Below => self.add_selection_by_movement(text, Motion::Below, Quantity::Character),
                _ => unimplemented!(),
            },
            Gesture { line, col, ty } => self.do_gesture(text, line, col, ty),
            GoToLine(line) => self.goto_line(text, line),
            CollapseSelections => self.collapse_selections(text),
            _ => unimplemented!(),
        }
    }

    fn do_gesture(&mut self, text: &Rope, line: u64, col: u64, ty: GestureType) {
        let line = line as usize;
        let col = col as usize;
        let offset = self.line_col_to_offset(text, line, col);
        match ty {
            GestureType::Select { quantity, multi } => {
                self.select(text, offset, quantity, multi)
            }
            GestureType::SelectExtend { quantity } => {
                self.extend_selection(text, offset, quantity)
            }
            GestureType::Drag => self.do_drag(text, offset, Affinity::default()),

            _ => {
                panic!("Deprecated gesture type sent to do_gesture method");
            }
        }
    }

    fn goto_line(&mut self, text: &Rope, line: u64) {
        let offset = self.line_col_to_offset(text, line as usize, 0);
        self.set_selection(text, SelRegion::caret(offset));
    }

    pub fn set_size(&mut self, size: Size) {
        self.size = size;
    }

    pub fn set_scroll(&mut self, first: i64, last: i64) {
        let first = max(first, 0) as usize;
        let last = max(last, 0) as usize;
        self.first_line = first;
        self.height = last - first;
    }

    pub fn scroll_height(&self) -> usize {
        self.height
    }

    fn scroll_to_cursor(&mut self, text: &Rope) {
        let end = self.sel_regions().last().unwrap().end;
        let line = self.line_of_offset(text, end);
        if line < self.first_line {
            self.first_line = line;
        } else if self.first_line + self.height <= line {
            self.first_line = line - (self.height - 1);
        }
        // We somewhat arbitrarily choose the last region for setting the old-style
        // selection state, and for scrolling it into view if needed. This choice can
        // likely be improved.
        self.scroll_to = Some(end);
    }

    /// Removes any selection present at the given offset.
    /// Returns true if a selection was removed, false otherwise.
    pub fn deselect_at_offset(&mut self, text: &Rope, offset: usize) -> bool {
        if !self.selection.regions_in_range(offset, offset).is_empty() {
            let mut sel = self.selection.clone();
            sel.delete_range(offset, offset, true);
            if !sel.is_empty() {
                self.drag_state = None;
                self.set_selection_raw(text, sel);
                return true;
            }
        }
        false
    }

    /// Move the selection by the given movement. Return value is the offset of
    /// a point that should be scrolled into view.
    ///
    /// If `modify` is `true`, the selections are modified, otherwise the results
    /// of individual region movements become carets.
    pub fn do_move(&mut self, text: &Rope, movement: Motion, quantity: Quantity, modify: bool) {
        self.drag_state = None;
        let new_sel =
            selection_movement(movement, quantity, &self.selection, self, self.scroll_height(), text, modify);
        self.set_selection(text, new_sel);
    }

    /// Set the selection to a new value.
    pub fn set_selection<S: Into<Selection>>(&mut self, text: &Rope, sel: S) {
        self.set_selection_raw(text, sel.into());
        self.scroll_to_cursor(text);
    }

    /// Sets the selection to a new value, without invalidating.
    fn set_selection_for_edit(&mut self, text: &Rope, sel: Selection) {
        self.selection = sel;
        self.scroll_to_cursor(text);
    }

    /// Sets the selection to a new value, invalidating the line cache as needed.
    /// This function does not perform any scrolling.
    fn set_selection_raw(&mut self, text: &Rope, sel: Selection) {
        self.invalidate_selection(text);
        self.selection = sel;
        self.invalidate_selection(text);
    }

    /// Invalidate the current selection. Note that we could be even more
    /// fine-grained in the case of multiple cursors, but we also want this
    /// method to be fast even when the selection is large.
    fn invalidate_selection(&mut self, text: &Rope) {
        // TODO: refine for upstream (caret appears on prev line)
        let first_line = self.line_of_offset(text, self.selection.first().unwrap().min());
        let last_line = self.line_of_offset(text, self.selection.last().unwrap().max()) + 1;
        let all_caret = self.selection.iter().all(|region| region.is_caret());
        let invalid = if all_caret {
            line_cache_shadow::CURSOR_VALID
        } else {
            line_cache_shadow::CURSOR_VALID | line_cache_shadow::STYLES_VALID
        };
        self.lc_shadow.partial_invalidate(first_line, last_line, invalid);
    }

    fn add_selection_by_movement(&mut self, text: &Rope, motion: Motion, quantity: Quantity) {
        let mut sel = Selection::new();
        for &region in self.sel_regions() {
            sel.add_region(region);
            let new_region =
                region_movement(motion, quantity, region, self, self.scroll_height(), &text, false);
            sel.add_region(new_region);
        }
        self.set_selection(text, sel);
    }

    // TODO: insert from keyboard or input method shouldn't break undo group,
    /// Invalidates the styles of the given range (start and end are offsets within
    /// the text).
    pub fn invalidate_styles(&mut self, text: &Rope, start: usize, end: usize) {
        let first_line = self.line_of_offset(text, start);
        let (mut last_line, last_col) = self.offset_to_line_col(text, end);
        last_line += if last_col > 0 { 1 } else { 0 };
        self.lc_shadow.partial_invalidate(first_line, last_line, line_cache_shadow::STYLES_VALID);
    }

    pub fn update_annotations(
        &mut self,
        plugin: PluginId,
        interval: Interval,
        annotations: Annotations,
    ) {
        self.annotations.update(plugin, interval, annotations)
    }

    /// Select entire buffer.
    ///
    /// Note: unlike movement based selection, this does not scroll.
    pub fn select_all(&mut self, text: &Rope) {
        let selection = SelRegion::new(0, text.len()).into();
        self.set_selection_raw(text, selection);
    }

    /// Finds the unit of text containing the given offset.
    fn unit(&self, text: &Rope, offset: usize, quantity: Quantity) -> Interval {
        match quantity {
            Quantity::Character => Interval::new(offset, offset),
            Quantity::Word => {
                let mut word_cursor = WordCursor::new(text, offset);
                let (start, end) = word_cursor.select_word();
                Interval::new(start, end)
            }
            Quantity::Line => {
                let (line, _) = self.offset_to_line_col(text, offset);
                let (start, end) = self.lines.logical_line_range(text, line);
                Interval::new(start, end)
            }
            _ => unimplemented!(),
        }
    }

    /// Selects text with a certain granularity and supports multi_selection
    fn select(
        &mut self,
        text: &Rope,
        offset: usize,
        quantity: Quantity,
        multi: bool,
    ) {
        // If multi-select is enabled, toggle existing regions
        if multi
            && quantity == Quantity::Character
            && self.deselect_at_offset(text, offset)
        {
            return;
        }

        let region = self.unit(text, offset, quantity).into();

        let base_sel = match multi {
            true => self.selection.clone(),
            false => Selection::new(),
        };
        let mut selection = base_sel.clone();
        selection.add_region(region);
        self.set_selection(text, selection);

        self.drag_state =
            Some(DragState { base_sel, min: region.start, max: region.end, quantity });
    }

    /// Extends an existing selection (eg. when the user performs SHIFT + click).
    pub fn extend_selection(
        &mut self,
        text: &Rope,
        offset: usize,
        quantity: Quantity,
    ) {
        if self.sel_regions().is_empty() {
            return;
        }

        let (base_sel, last) = {
            let mut base = Selection::new();
            let (last, rest) = self.sel_regions().split_last().unwrap();
            for &region in rest {
                base.add_region(region);
            }
            (base, *last)
        };

        let mut sel = base_sel.clone();
        self.drag_state =
            Some(DragState { base_sel, min: last.start, max: last.start, quantity });

        let start = (last.start, last.start);
        let new_region = self.range_region(text, start, offset, quantity);

        // TODO: small nit, merged region should be backward if end < start.
        // This could be done by explicitly overriding, or by tweaking the
        // merge logic.
        sel.add_region(new_region);
        self.set_selection(text, sel);
    }

    /// Does a drag gesture, setting the selection from a combination of the drag
    /// state and new offset.
    fn do_drag(&mut self, text: &Rope, offset: usize, affinity: Affinity) {
        let new_sel = self.drag_state.as_ref().map(|drag_state| {
            let mut sel = drag_state.base_sel.clone();
            let start = (drag_state.min, drag_state.max);
            let new_region = self.range_region(text, start, offset, drag_state.quantity);
            sel.add_region(new_region.with_horiz(None).with_affinity(affinity));
            sel
        });

        if let Some(sel) = new_sel {
            self.set_selection(text, sel);
        }
    }

    /// Creates a `SelRegion` for range select or drag operations.
    pub fn range_region(
        &self,
        text: &Rope,
        start: (usize, usize),
        offset: usize,
        quantity: Quantity,
    ) -> SelRegion {
        let (min_start, max_start) = start;
        let end = self.unit(text, offset, quantity);
        let (min_end, max_end) = (end.start, end.end);
        if offset >= min_start {
            SelRegion::new(min_start, max_end)
        } else {
            SelRegion::new(max_start, min_end)
        }
    }

    /// Returns the regions of the current selection.
    pub fn sel_regions(&self) -> &[SelRegion] {
        &self.selection
    }

    /// Collapse all selections in this view into a single caret
    pub fn collapse_selections(&mut self, text: &Rope) {
        let mut sel = self.selection.clone();
        sel.collapse();
        self.set_selection(text, sel);
    }

    /// Determines whether the offset is in any selection (counting carets and
    /// selection edges).
    pub fn is_point_in_selection(&self, offset: usize) -> bool {
        !self.selection.regions_in_range(offset, offset).is_empty()
    }

    pub fn encode_styles(
        &self,
        client: &Client,
        styles: &StyleMap,
        start: usize,
        end: usize,
        sel: &[(usize, usize)],
        hls: &Vec<Vec<(usize, usize)>>,
        style_spans: &Spans<Style>,
    ) -> Vec<usize> {
        let mut encoded_styles = Vec::new();
        assert!(start <= end, "{} {}", start, end);
        let style_spans = style_spans.subseq(Interval::new(start, end));

        let mut ix = 0;
        // we add the special find highlights (1 to N) and selection (0) styles first.
        // We add selection after find because we want it to be preferred if the
        // same span exists in both sets (as when there is an active selection)
        for (index, cur_find_hls) in hls.iter().enumerate() {
            for &(sel_start, sel_end) in cur_find_hls {
                encoded_styles.push((sel_start) - ix);
                encoded_styles.push(sel_end - sel_start);
                encoded_styles.push(index + 1);
                ix = sel_end;
            }
        }
        for &(sel_start, sel_end) in sel {
            encoded_styles.push((sel_start) - ix);
            encoded_styles.push(sel_end - sel_start);
            encoded_styles.push(0);
            ix = sel_end;
        }
        for (iv, style) in style_spans.iter() {
            let style_id = self.get_or_def_style_id(client, styles, &style);
            encoded_styles.push(iv.start() - ix);
            encoded_styles.push(iv.end() - iv.start());
            encoded_styles.push(style_id);
            ix = iv.end();
        }
        encoded_styles
    }

    fn get_or_def_style_id(&self, client: &Client, style_map: &StyleMap, style: &Style) -> usize {
        
        let mut style_map = style_map.lock().unwrap();
        if let Some(ix) = style_map.lookup(style) {
            return ix;
        }
        let ix = style_map.add(style);
        let style = style_map.merge_with_default(style);

        client.define_style(ix, style);

        ix
    }

    // Encode a single line with its styles and cursors.
    // If "text" is not specified, don't add "text" to the output.
    // If "style_spans" are not specified, don't add "styles" to the output.
    fn encode_line(
        &self,
        client: &Client,
        styles: &StyleMap,
        line: VisualLine,
        text: Option<&Rope>,
        style_spans: Option<&Spans<Style>>,
        last_pos: usize,
    ) -> LineUpdate {
        let start_pos = line.interval.start;
        let pos = line.interval.end;
        let mut cursors = Vec::new();
        let mut selections = Vec::new();
        for region in self.selection.regions_in_range(start_pos, pos) {
            // cursor
            let c = region.end;

            if (c > start_pos && c < pos)
                || (!region.is_upstream() && c == start_pos)
                || (region.is_upstream() && c == pos)
                || (c == pos && c == last_pos)
            {
                cursors.push(c - start_pos);
            }

            // selection with interior
            let sel_start_ix = clamp(region.min(), start_pos, pos) - start_pos;
            let sel_end_ix = clamp(region.max(), start_pos, pos) - start_pos;
            if sel_end_ix > sel_start_ix {
                selections.push((sel_start_ix, sel_end_ix));
            }
        }
        
        // TODO find highlighting
        let hls = Vec::new();
        let mut result = LineUpdate::default();

        if let Some(text) = text {
            result.text = Some(text.slice_to_cow(start_pos..pos).to_string());
        }
        if let Some(style_spans) = style_spans {
            result.styles = self.encode_styles(
                client,
                styles,
                start_pos,
                pos,
                &selections,
                &hls,
                style_spans
            );
        }
        
        result.cursors = cursors;
        result.ln = line.line_num;

        result
    }

    fn send_update_for_plan(
        &mut self,
        text: &Rope,
        client: &Client,
        styles: &StyleMap,
        style_spans: &Spans<Style>,
        plan: &RenderPlan,
        pristine: bool,
    ) {
        // every time current visible range changes, annotations are sent to frontend
        let start_off = self.offset_of_line(text, self.first_line);
        let end_off = self.offset_of_line(text, self.first_line + self.height + 2);
        let visible_range = Interval::new(start_off, end_off);
        let selection_annotations =
            self.selection.get_annotations(visible_range, &self, text);

        let annotations = vec![selection_annotations];

        if !self.lc_shadow.needs_render(plan) {
            let total_lines = self.line_of_offset(text, text.len()) + 1;
            let update =
                Update { ops: vec![UpdateOp::copy(total_lines, 1)], pristine, annotations };
            client.update_view(self.view_id, &update);
            return;
        }

        let mut b = line_cache_shadow::Builder::new();
        let mut ops = Vec::new();
        let mut line_num = 0; // tracks old line cache

        for seg in self.lc_shadow.iter_with_plan(plan) {
            match seg.tactic {
                RenderTactic::Discard => {
                    ops.push(UpdateOp::invalidate(seg.n));
                    b.add_span(seg.n, 0, 0);
                }
                RenderTactic::Preserve | RenderTactic::Render => {
                    // Depending on the state of TEXT_VALID, STYLES_VALID and
                    // CURSOR_VALID, perform one of the following actions:
                    //
                    //   - All the three are valid => send the "copy" op
                    //     (+leading "skip" to catch up with "ln" to update);
                    //
                    //   - Text and styles are valid, cursors are not => same,
                    //     but send an "update" op instead of "copy" to move
                    //     the cursors;
                    //
                    //   - Text or styles are invalid:
                    //     => send "invalidate" if RenderTactic is "Preserve";
                    //     => send "skip"+"insert" (recreate the lines) if
                    //        RenderTactic is "Render".
                    if (seg.validity & line_cache_shadow::TEXT_VALID) != 0
                        && (seg.validity & line_cache_shadow::STYLES_VALID) != 0
                    {
                        let n_skip = seg.their_line_num - line_num;
                        if n_skip > 0 {
                            ops.push(UpdateOp::skip(n_skip));
                        }
                        let line_offset = self.offset_of_line(text, seg.our_line_num);
                        let logical_line = text.line_of_offset(line_offset);
                        if (seg.validity & line_cache_shadow::CURSOR_VALID) != 0 {
                            // ALL_VALID; copy lines as-is
                            ops.push(UpdateOp::copy(seg.n, logical_line + 1));
                        } else {
                            // !CURSOR_VALID; update cursors
                            let start_line = seg.our_line_num;

                            let encoded_lines = self
                                .lines
                                .iter_lines(text, start_line)
                                .take(seg.n)
                                .map(|l| {
                                    self.encode_line(
                                        client,
                                        styles,
                                        l,
                                        None,
                                        None,
                                        text.len(),
                                    )
                                })
                                .collect::<Vec<_>>();

                            let logical_line_opt =
                                if logical_line == 0 { None } else { Some(logical_line + 1) };
                            ops.push(UpdateOp::update(encoded_lines, logical_line_opt));
                        }
                        b.add_span(seg.n, seg.our_line_num, seg.validity);
                        line_num = seg.their_line_num + seg.n;
                    } else if seg.tactic == RenderTactic::Preserve {
                        ops.push(UpdateOp::invalidate(seg.n));
                        b.add_span(seg.n, 0, 0);
                    } else if seg.tactic == RenderTactic::Render {
                        let start_line = seg.our_line_num;
                        let encoded_lines = self
                            .lines
                            .iter_lines(text, start_line)
                            .take(seg.n)
                            .map(|l| {
                                self.encode_line(
                                    client,
                                    styles,
                                    l,
                                    Some(text),
                                    Some(style_spans),
                                    text.len(),
                                )
                            })
                            .collect::<Vec<_>>();
                        debug_assert_eq!(encoded_lines.len(), seg.n);
                        ops.push(UpdateOp::insert(encoded_lines));
                        b.add_span(seg.n, seg.our_line_num, line_cache_shadow::ALL_VALID);
                    }
                }
            }
        }

        self.lc_shadow = b.build();

        let update = Update { ops, pristine, annotations };
        client.update_view(self.view_id, &update);
    }

    /// Update front-end with any changes to view since the last time sent.
    /// The `pristine` argument indicates whether or not the buffer has
    /// unsaved changes.
    pub fn render_if_dirty(
        &mut self,
        text: &Rope,
        client: &Client,
        styles: &StyleMap,
        style_spans: &Spans<Style>,
        pristine: bool,
    ) {
        let height = self.line_of_offset(text, text.len()) + 1;
        let plan = RenderPlan::create(height, self.first_line, self.height);
        self.send_update_for_plan(text, client, styles, style_spans, &plan, pristine);
        if let Some(new_scroll_pos) = self.scroll_to.take() {
            let (line, col) = self.offset_to_line_col(text, new_scroll_pos);
            client.scroll_to(self.view_id, line, col);
        }
    }

    // Send the requested lines even if they're outside the current scroll region.
    pub fn request_lines(
        &mut self,
        text: &Rope,
        client: &Client,
        styles: &StyleMap,
        style_spans: &Spans<Style>,
        first_line: usize,
        last_line: usize,
        pristine: bool,
    ) {
        let height = self.line_of_offset(text, text.len()) + 1;
        let mut plan = RenderPlan::create(height, self.first_line, self.height);
        plan.request_lines(first_line, last_line);
        self.send_update_for_plan(text, client, styles, style_spans, &plan, pristine);
    }

    /// Invalidates front-end's entire line cache, forcing a full render at the next
    /// update cycle. This should be a last resort, updates should generally cause
    /// finer grain invalidation.
    pub fn set_dirty(&mut self, text: &Rope) {
        let height = self.line_of_offset(text, text.len()) + 1;
        let mut b = line_cache_shadow::Builder::new();
        b.add_span(height, 0, 0);
        b.set_dirty(true);
        self.lc_shadow = b.build();
    }

    /// Returns the byte range of the currently visible lines.
    fn interval_of_visible_region(&self, text: &Rope) -> Interval {
        let start = self.offset_of_line(text, self.first_line);
        let end = self.offset_of_line(text, self.first_line + self.height + 1);
        Interval::new(start, end)
    }

    /// Generate line breaks, based on current settings. Currently batch-mode,
    /// and currently in a debugging state.
    pub fn rewrap(
        &mut self,
        text: &Rope,
        width_cache: &mut WidthCache,
        client: &Client,
        spans: &Spans<Style>,
    ) {
        let visible = self.first_line..self.first_line + self.height;
        let inval = self.lines.rewrap_chunk(text, width_cache, client, spans, visible);
        if let Some(InvalLines { start_line, inval_count, new_count }) = inval {
            self.lc_shadow.edit(start_line, start_line + inval_count, new_count);
        }
    }

    /// Updates the view after the text has been modified by the given `delta`.
    /// This method is responsible for updating the cursors, and also for
    /// recomputing line wraps.
    pub fn after_edit(
        &mut self,
        text: &Rope,
        last_text: &Rope,
        delta: &RopeDelta,
        client: &Client,
        width_cache: &mut WidthCache,
        drift: InsertDrift,
    ) {
        let visible = self.first_line..self.first_line + self.height;
        match self.lines.after_edit(text, last_text, delta, width_cache, client, visible) {
            Some(InvalLines { start_line, inval_count, new_count }) => {
                self.lc_shadow.edit(start_line, start_line + inval_count, new_count);
            }
            None => self.set_dirty(text),
        }

        // Any edit cancels a drag. This is good behavior for edits initiated through
        // the front-end, but perhaps not for async edits.
        self.drag_state = None;

        // all annotations that come after the edit need to be invalidated
        let (iv, _) = delta.summary();
        self.annotations.invalidate(iv);

        // Note: for committing plugin edits, we probably want to know the priority
        // of the delta so we can set the cursor before or after the edit, as needed.
        let new_sel = self.selection.apply_delta(delta, true, drift);
        self.set_selection_for_edit(text, new_sel);
    }

    pub fn get_caret_offset(&self) -> Option<usize> {
        match self.selection.len() {
            1 if self.selection[0].is_caret() => {
                let offset = self.selection[0].start;
                Some(offset)
            }
            _ => None,
        }
    }
}

impl LineOffset for View {
    fn offset_of_line(&self, text: &Rope, line: usize) -> usize {
        self.lines.offset_of_visual_line(text, line)
    }

    fn line_of_offset(&self, text: &Rope, offset: usize) -> usize {
        self.lines.visual_line_of_offset(text, offset)
    }
}

// utility function to clamp a value within the given range
fn clamp(x: usize, min: usize, max: usize) -> usize {
    if x < min {
        min
    } else if x < max {
        x
    } else {
        max
    }
}

