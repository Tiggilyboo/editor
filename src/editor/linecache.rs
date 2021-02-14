use std::mem;
use std::ops::Range;
use serde_json::Value;
use serde::Deserialize;

use rpc::{
    Annotation,
    AnnotationType,
};

pub struct LineCache {
    lines: Vec<Option<Line>>,
    annotations: Vec<Annotation>,
    selections: Vec<Selection>,
}

#[derive(Debug)]
pub struct Line {
    text: String,
    cursor: Vec<usize>,
    styles: Vec<StyleSpan>,
    line_num: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct StyleSpan {
    pub style_id: usize,
    pub range: Range<usize>,
}

pub struct Selection {
    pub line_num: usize,
    pub start_col: usize,
    pub end_col: usize,
}

impl Line {
    pub fn from_json(v: &Value) -> Line {
        let text = v["text"].as_str().to_owned();
        let line_num = v["ln"].as_u64();
        let mut cursor = Vec::new();
        if let Some(arr) = v["cursor"].as_array() {
            for c in arr {
                let offset_utf8 = c.as_u64().unwrap() as usize;
                if let Some(text) = text {
                    cursor.push(count_utf16(&text[..offset_utf8]));
                } else {
                    cursor.push(offset_utf8);
                }
            }
        }

        let mut styles = Vec::new();
        if let Some(arr) = v["styles"].as_array() {
            let mut ix: i64 = 0;
            for triple in arr.chunks(3) {
                let start = ix + triple[0].as_i64().unwrap();
                let end = start + triple[1].as_i64().unwrap();
                // TODO: count utf from last end, if <=
                let start_utf16 = if let Some(text) = text {
                    count_utf16(&text[..start as usize])
                } else {
                    start as usize
                };
                let end_utf16 = start_utf16 + if let Some(text) = text {
                    count_utf16(&text[start as usize .. end as usize])
                } else {
                    end as usize
                };
                let style_id = triple[2].as_u64().unwrap() as usize;
                let style_span = StyleSpan {
                    style_id,
                    range: start_utf16..end_utf16,
                };
                styles.push(style_span);
                ix = end;
            }
        }

        Line { 
            text: text.unwrap_or_default().to_string(), 
            line_num,
            cursor, 
            styles,
        }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn cursor(&self) -> &[usize] {
        &self.cursor
    }

    pub fn styles(&self) -> &[StyleSpan] {
        &self.styles
    }

    pub fn line_num(&self) -> Option<u64> {
        self.line_num
    }
}

impl LineCache {
    pub fn new() -> LineCache {
        LineCache {
            lines: Vec::new(),
            annotations: Vec::new(),
            selections: Vec::new(),
        }
    }

    fn push_opt_line(&mut self, line: Option<Line>) {
        self.lines.push(line);
    }

    pub fn apply_update(&mut self, update: &Value) {
        let old_cache = mem::replace(self, LineCache::new());
        let mut old_iter = old_cache.lines.into_iter();

        for op in update["ops"].as_array().unwrap() {
            if let Some(op_type) = op["op"].as_str() {
                match op_type {
                    "ins" => {
                        for line in op["lines"].as_array().unwrap() {
                            let line = Line::from_json(line);
                            self.push_opt_line(Some(line));
                        }
                    },
                    "copy" => {
                        let n = op["n"].as_u64().unwrap();
                        for _ in 0..n {
                            self.push_opt_line(old_iter.next().unwrap_or_default());
                        }
                    },
                    "skip" => {
                        let n = op["n"].as_u64().unwrap();
                        for _ in 0..n {
                            let _ = old_iter.next();
                        }
                    },
                    "invalidate" => {
                        let n = op["n"].as_u64().unwrap();
                        for _ in 0..n {
                            self.push_opt_line(None);
                        }
                    },
                    "update" => {
                        for line in op["lines"].as_array().unwrap() {
                            let line = Line::from_json(line);
                            if let Some(mut new_line) = old_iter.next().unwrap_or_default() {
                                new_line.cursor = line.cursor;
                                self.push_opt_line(Some(new_line));
                            } else {
                                self.push_opt_line(None);
                            }
                        }
                        println!("update received: {}", update);
                    }
                    _ => println!("unhandled update operation: {:?}", op_type)
                }
            }
        }

        for raw_anno in update["annotations"].as_array().unwrap() {
            let mut anno = <Annotation>::deserialize(raw_anno)
                .expect("unable to deserialize annotation");

            match anno.annotation_type {
                AnnotationType::Selection => {
                    for range in anno.ranges.iter_mut() {
                        for line_num in range.start_line..range.end_line+1 {
                            if let Some(line) = &self.lines.get(line_num) {
                                if let Some(line) = line {
                                    let len = line.text.len();
                                    let (start_col, end_col) = if range.start_col > range.end_col { 
                                        (range.end_col, range.start_col)
                                    } else { 
                                        (range.start_col, range.end_col)
                                    };
                                    let start_col = if start_col >= len { len } else { start_col };
                                    let end_col = if end_col >= len { len } else { end_col };

                                    let left_utf16 = count_utf16(&line.text[..start_col]);
                                    let width_utf16 = count_utf16(&line.text[start_col..end_col]);

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
                },
                _ => self.annotations.push(anno),
            }
        }
    }

    pub fn height(&self) -> usize {
        self.lines.len()
    }

    pub fn get_line(&self, ix: usize) -> Option<&Line> {
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
