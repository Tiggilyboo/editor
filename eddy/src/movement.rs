use std::cmp::max;

use super::{
    Motion,
    Quantity,
};
use rope::{
    Rope,
    LinesMetric, 
    Cursor,
};
use super::line_offset::LineOffset;
use super::words::WordCursor;
use super::selection::{
    SelRegion,
    HorizPos,
    Selection,
};

/// When paging through a file, the number of lines from the previous page
/// that will also be visible in the next.
const SCROLL_OVERLAP: isize = 2;

/// Computes the actual desired amount of scrolling (generally slightly
/// less than the height of the viewport, to allow overlap).
fn scroll_height(height: usize) -> isize {
    max(height as isize - SCROLL_OVERLAP, 1)
}

/// Based on the current selection position this will return the cursor position, the current line, and the
/// total number of lines of the file.
fn selection_position(
    r: SelRegion,
    lo: &dyn LineOffset,
    text: &Rope,
    move_up: bool,
    modify: bool,
) -> (HorizPos, usize) {
    // The active point of the selection
    let active = if modify {
        r.end
    } else if move_up {
        r.min()
    } else {
        r.max()
    };
    let col = if let Some(col) = r.horiz { col } else { lo.offset_to_line_col(text, active).1 };
    let line = lo.line_of_offset(text, active);

    (col, line)
}

/// Compute movement based on vertical motion by the given number of lines.
///
/// Note: in non-exceptional cases, this function preserves the `horiz`
/// field of the selection region.
fn vertical_motion(
    r: SelRegion,
    lo: &dyn LineOffset,
    text: &Rope,
    line_delta: isize,
    modify: bool,
) -> (usize, Option<HorizPos>) {
    let (col, line) = selection_position(r, lo, text, line_delta < 0, modify);
    let n_lines = lo.line_of_offset(text, text.len());

    // This code is quite careful to avoid integer overflow.
    // TODO: write tests to verify
    if line_delta < 0 && (-line_delta as usize) > line {
        return (0, Some(col));
    }
    let line = if line_delta < 0 {
        line - (-line_delta as usize)
    } else {
        line.saturating_add(line_delta as usize)
    };
    if line > n_lines {
        return (text.len(), Some(col));
    }
    let new_offset = lo.line_col_to_offset(text, line, col);
    (new_offset, Some(col))
}

/// Compute movement based on vertical motion by the given number of lines skipping
/// any line that is shorter than the current cursor position.
fn vertical_motion_exact_pos(
    r: SelRegion,
    lo: &dyn LineOffset,
    text: &Rope,
    move_up: bool,
    modify: bool,
) -> (usize, Option<HorizPos>) {
    let (col, init_line) = selection_position(r, lo, text, move_up, modify);
    let n_lines = lo.line_of_offset(text, text.len());

    let mut line_length =
        lo.offset_of_line(text, init_line.saturating_add(1)) - lo.offset_of_line(text, init_line);
    if move_up && init_line == 0 {
        return (lo.line_col_to_offset(text, init_line, col), Some(col));
    }
    let mut line = if move_up { init_line - 1 } else { init_line.saturating_add(1) };

    // If the active columns is longer than the current line, use the current line length.
    let col = if line_length < col { line_length - 1 } else { col };

    loop {
        line_length = lo.offset_of_line(text, line + 1) - lo.offset_of_line(text, line);

        // If the line is longer than the current cursor position, break.
        // We use > instead of >= because line_length includes newline.
        if line_length > col {
            break;
        }

        // If you are trying to add a selection past the end of the file or before the first line, return original selection
        if line >= n_lines || (line == 0 && move_up) {
            line = init_line;
            break;
        }

        line = if move_up { line - 1 } else { line.saturating_add(1) };
    }

    (lo.line_col_to_offset(text, line, col), Some(col))
}


/// Compute the result of movement on one selection region.
///
/// # Arguments
///
/// * `height` - viewport height
pub fn region_movement(
    m: Motion,
    q: Quantity,
    r: SelRegion,
    lo: &dyn LineOffset,
    height: usize,
    text: &Rope,
    modify: bool,
) -> SelRegion {
    let (offset, horiz) = match q {
        Quantity::Word => match m {
            Motion::Backward => {
                let mut word_cursor = WordCursor::new(text, r.end);
                let offset = word_cursor.prev_boundary().unwrap_or(0);
                (offset, None)
            },
            Motion::Forward => {
                let mut word_cursor = WordCursor::new(text, r.end);
                let offset = word_cursor.next_boundary().unwrap_or_else(|| text.len());
                (offset, None)
            },
            _ => unimplemented!(),
        },
        Quantity::Character => match m {
            Motion::Backward => {
                if r.is_caret() || modify {
                    if let Some(offset) = text.prev_grapheme_offset(r.end) {
                        (offset, None)
                    } else {
                        (0, r.horiz)
                    }
                } else {
                    (r.min(), None)
                }
            },
            Motion::Forward => {
                if r.is_caret() || modify {
                    if let Some(offset) = text.next_grapheme_offset(r.end) {
                        (offset, None)
                    } else {
                        (r.end, r.horiz)
                    }
                } else {
                    (r.max(), None)
                }
            },
            Motion::Above => vertical_motion(r, lo, text, -1, modify), 
            Motion::Below => vertical_motion(r, lo, text, 1, modify),
            _ => unimplemented!(),
        },
        Quantity::Line => match m {
            Motion::First => {
                let line = lo.line_of_offset(text, r.end);
                let offset = lo.offset_of_line(text, line);
                (offset, None)
            },
            Motion::Last => {
                let line = lo.line_of_offset(text, r.end);
                let mut offset = text.len();

                // calculate end of line
                let next_line_offset = lo.offset_of_line(text, line + 1);
                if line < lo.line_of_offset(text, offset) {
                    if let Some(prev) = text.prev_grapheme_offset(next_line_offset) {
                        offset = prev;
                    }
                }
                (offset, None)
            },
            Motion::Begin => {
                let mut cursor = Cursor::new(&text, r.end);
                let offset = cursor.prev::<LinesMetric>().unwrap_or(0);
                (offset, None)
            },
            Motion::End => {
                let mut offset = r.end;
                let mut cursor = Cursor::new(&text, offset);
                if let Some(next_para_offset) = cursor.next::<LinesMetric>() {
                    if cursor.is_boundary::<LinesMetric>() {
                        if let Some(eol) = text.prev_grapheme_offset(next_para_offset) {
                            offset = eol;
                        }
                    } else if cursor.pos() == text.len() {
                        offset = text.len();
                    }
                    (offset, None)
                } else {
                    //in this case we are already on a last line so just moving to EOL
                    (text.len(), None)
                } 
            },
            _ => unimplemented!(),
        },
        Quantity::Page => match m {
            Motion::Above => vertical_motion(r, lo, text, -scroll_height(height), modify),
            Motion::Below => vertical_motion(r, lo, text, scroll_height(height), modify),
            _ => unimplemented!(),
        },
        _ => unimplemented!(),
    };

    SelRegion::new(if modify { r.start } else { offset }, offset).with_horiz(horiz)
}

pub fn selection_movement(
    m: Motion,
    q: Quantity,
    s: &Selection,
    lo: &dyn LineOffset,
    height: usize,
    text: &Rope,
    modify: bool,
) -> Selection {
    let mut sel = Selection::new();
    for &r in s.iter() {
        let new_region = region_movement(m, q, r, lo, height, text, modify);
        sel.add_region(new_region);
    }

    sel
}
