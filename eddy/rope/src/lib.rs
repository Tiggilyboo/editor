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

//! Trees for text.

extern crate bytecount;
extern crate memchr;
extern crate regex;
extern crate unicode_segmentation;

pub mod breaks;
pub mod compare;
pub mod delta;
pub mod diff;
pub mod engine;
pub mod find;
pub mod interval;
pub mod multiset;
pub mod rope;
pub mod spans;
pub mod tree;

pub use crate::delta::{Builder as DeltaBuilder, Delta, DeltaElement, Transformer};
pub use crate::interval::Interval;
pub use crate::rope::{LinesMetric, Rope, RopeDelta, RopeInfo};
pub use crate::tree::{Cursor, Metric};
