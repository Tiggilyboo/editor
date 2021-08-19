use std::mem;
use std::ops::Range;
use eddy::annotations::{
    AnnotationSlice,
    AnnotationType,
};
use eddy::client::{Update, UpdateOp, OpType, LineUpdate};

#[derive(Debug)]
pub struct Selection {
    line_num: usize,
    start_col: usize,
    end_col: usize,
}

#[derive(Debug)]
pub struct LineCache {
    lines: Vec<Option<LineUpdate>>,
    annotations: Vec<AnnotationSlice>,
    selections: Vec<Selection>,
}

impl LineCache {
    pub fn new() -> Self {
        Self {
            lines: Vec::<Option<LineUpdate>>::new(),
            annotations: Vec::new(),
            selections: Vec::new(),
        }
    }

    fn push_opt_line(&mut self, line: Option<LineUpdate>) {
        self.lines.push(line);
    }

    pub fn apply_update(&mut self, update: Update) {
        let old_cache = mem::replace(self, LineCache::new());
        let mut old_iter = old_cache.lines.into_iter();

        for o in update.ops {
            let n = o.n;
            match o.op {
                OpType::Insert => {
                    if let Some(lines) = o.lines {
                        for line in lines {
                            self.push_opt_line(Some(line));
                        }
                    }
                },
                OpType::Copy => {
                    for _ in 0..n {
                        self.push_opt_line(old_iter.next().unwrap_or_default());
                    }
                },
                OpType::Skip => {
                    for _ in 0..n {
                        let _ = old_iter.next();
                    }
                },
                OpType::Invalidate => {
                    for _ in 0..n {
                        self.push_opt_line(None);
                    }
                },
                OpType::Update => {
                    for line in o.lines {
                        if let Some(mut new_line) = old_iter.next().unwrap_or_default() {
                            self.push_opt_line(Some(new_line));
                        } else {
                            self.push_opt_line(None);
                        }
                    }
                }
                _ => println!("unhandled update operation: {:?}", o.op)
            }
        }

        for anno in update.annotations.iter() {
            let mut anno = anno.clone();
            match anno.annotation_type {
                AnnotationType::Selection => {
                    for range in anno.ranges.iter_mut() {
                        for line_num in range.start_line..range.end_line+1 {
                            if let Some(line) = &self.lines.get(line_num) {
                                if let Some(line) = line {
                                    if let Some(text) = &line.text {
                                        let len = text.len();
                                        let (start_col, end_col) = if range.start_col > range.end_col { 
                                            (range.end_col, range.start_col)
                                        } else { 
                                            (range.start_col, range.end_col)
                                        };
                                        let start_col = if start_col >= len { len } else { start_col };
                                        let end_col = if end_col >= len { len } else { end_col };

                                        let left_utf16 = count_utf16(&text[..start_col]);
                                        let width_utf16 = count_utf16(&text[start_col..end_col]);

                                        range.start_col = left_utf16;
                                        range.end_col = left_utf16 + width_utf16;

                                        self.selections.push(Selection {
                                            line_num,
                                            start_col,
                                            end_col,
                                        });
                                    }
                                }
                            }
                        }
                    }
                },
                _ => self.annotations.push(anno.clone()),
            }
        }

        println!("apply_update: {:?}", self);
    }

    pub fn height(&self) -> usize {
        self.lines.len()
    }

    pub fn get_line(&self, ix: usize) -> Option<&LineUpdate> {
        if ix < self.lines.len() {
            self.lines[ix].as_ref()
        } else {
            None
        }
    }

    pub fn get_selections(&self, line_num: usize) -> Vec<&Selection> {
        self.selections.iter().filter(|s| s.line_num == line_num).collect()
    }

    pub fn clear(&mut self) {
        self.selections.clear();
        self.lines.clear();
        self.annotations.clear();
    }
}

/// Counts the number of utf-16 code units in the given string.
pub fn count_utf16(s: &str) -> usize {
    let mut utf16_count = 0;
    for &b in s.as_bytes() {
        if (b as i8) >= -0x40 { utf16_count += 1; }
        if b >= 0xf0 { utf16_count += 1; }
    }
    utf16_count
}
