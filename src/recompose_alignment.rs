// Copyright 2012-2015 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::decompose_alignment::DecompositionsAlignment;
use core::fmt::{self, Write};
use smallvec::SmallVec;

#[derive(Clone)]
enum RecompositionState {
    Composing,
    Purging(usize),
    Finished(usize),
}

/// External iterator for a string recomposition's characters.
#[derive(Clone)]
pub struct RecompositionsAlignment<I> {
    iter: DecompositionsAlignment<I>,
    state: RecompositionState,
    buffer: SmallVec<[(char, isize); 4]>,
    composee: Option<(char, isize)>,
    last_ccc: Option<u8>,
}

#[inline]
pub fn new_canonical<I: Iterator<Item = char>>(iter: I) -> RecompositionsAlignment<I> {
    RecompositionsAlignment {
        iter: super::decompose_alignment::new_canonical(iter),
        state: self::RecompositionState::Composing,
        buffer: SmallVec::new(),
        composee: None,
        last_ccc: None,
    }
}

#[inline]
pub fn new_compatible<I: Iterator<Item = char>>(iter: I) -> RecompositionsAlignment<I> {
    RecompositionsAlignment {
        iter: super::decompose_alignment::new_compatible(iter),
        state: self::RecompositionState::Composing,
        buffer: SmallVec::new(),
        composee: None,
        last_ccc: None,
    }
}

impl<I: Iterator<Item = char>> Iterator for RecompositionsAlignment<I> {
    type Item = (char, isize);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        use self::RecompositionState::*;

        loop {
            match self.state {
                Composing => {
                    for (ch, change) in self.iter.by_ref() {
                        let ch_class = super::char::canonical_combining_class(ch);
                        let k = match self.composee {
                            None => {
                                if ch_class != 0 {
                                    return Some((ch, change));
                                }
                                self.composee = Some((ch, change));
                                continue;
                            }
                            Some(k) => k,
                        };
                        match self.last_ccc {
                            None => match super::char::compose(k.0, ch) {
                                Some(r) => {
                                    self.composee = Some((r, k.1 + change - 1));
                                    continue;
                                }
                                None => {
                                    if ch_class == 0 {
                                        self.composee = Some((ch, change));
                                        return Some(k);
                                    }
                                    self.buffer.push((ch, change));
                                    self.last_ccc = Some(ch_class);
                                }
                            },
                            Some(l_class) => {
                                if l_class >= ch_class {
                                    // `ch` is blocked from `composee`
                                    if ch_class == 0 {
                                        self.composee = Some((ch, change));
                                        self.last_ccc = None;
                                        self.state = Purging(0);
                                        return Some(k);
                                    }
                                    self.buffer.push((ch, change));
                                    self.last_ccc = Some(ch_class);
                                    continue;
                                }
                                match super::char::compose(k.0, ch) {
                                    Some(r) => {
                                        self.composee = Some((r, k.1 + change - 1));
                                        continue;
                                    }
                                    None => {
                                        self.buffer.push((ch, change));
                                        self.last_ccc = Some(ch_class);
                                    }
                                }
                            }
                        }
                    }
                    self.state = Finished(0);
                    if self.composee.is_some() {
                        return self.composee.take();
                    }
                }
                Purging(next) => match self.buffer.get(next).cloned() {
                    None => {
                        self.buffer.clear();
                        self.state = Composing;
                    }
                    s => {
                        self.state = Purging(next + 1);
                        return s;
                    }
                },
                Finished(next) => match self.buffer.get(next).cloned() {
                    None => {
                        self.buffer.clear();
                        return self.composee.take();
                    }
                    s => {
                        self.state = Finished(next + 1);
                        return s;
                    }
                },
            }
        }
    }
}

impl<I: Iterator<Item = char> + Clone> fmt::Display for RecompositionsAlignment<I> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for c in self.clone() {
            f.write_char(c.0)?;
        }
        Ok(())
    }
}
