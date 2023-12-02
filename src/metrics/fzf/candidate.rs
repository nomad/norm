use super::*;
use crate::utils::*;
use crate::{Candidate, CandidateMatches};

/// TODO: docs
pub(super) struct CandidateV2<'a> {
    /// TODO: docs
    bonuses: &'a mut [Bonus],

    /// TODO: docs
    base: Candidate<'a>,

    /// TODO: docs
    initial_char_class: CharClass,

    /// TODO: docs
    opts: CandidateOpts,
}

/// TODO: docs
#[derive(Clone, Copy)]
pub(super) struct CandidateOpts {
    /// TODO: docs
    pub char_eq: CharEq,

    /// TODO: docs
    pub is_case_sensitive: bool,
}

impl Default for CandidateOpts {
    #[inline(always)]
    fn default() -> Self {
        Self { char_eq: char_eq(false, false), is_case_sensitive: false }
    }
}

impl CandidateOpts {
    #[inline(always)]
    pub fn new(is_case_sensitive: bool, is_normalized: bool) -> Self {
        Self {
            char_eq: char_eq(is_case_sensitive, is_normalized),
            is_case_sensitive,
        }
    }
}

impl<'a> CandidateV2<'a> {
    #[inline(always)]
    pub fn bonus_at(&mut self, char_idx: usize, scheme: &Scheme) -> Score {
        let bonus = &mut self.bonuses[char_idx];

        if bonus.is_set() {
            return bonus.value();
        }

        let prev_class = if char_idx == 0 {
            self.initial_char_class
        } else {
            char_class(self.char(char_idx - 1), scheme)
        };

        let this_class = char_class(self.char(char_idx), scheme);

        let bonus = &mut self.bonuses[char_idx];

        bonus.set(compute_bonus(prev_class, this_class, scheme));

        bonus.value()
    }

    #[inline(always)]
    pub fn char(&self, char_idx: usize) -> char {
        self.base.char(char_idx)
    }

    #[inline(always)]
    pub fn char_len(&self) -> usize {
        self.base.char_len()
    }

    #[inline(always)]
    pub fn into_base(self) -> Candidate<'a> {
        self.base
    }

    #[inline(always)]
    pub fn matches(&self, ch: char) -> CandidateMatches<'a> {
        self.base.matches(ch, self.opts.is_case_sensitive, self.opts.char_eq)
    }

    #[inline(always)]
    pub fn matches_from(
        &self,
        char_offset: usize,
        ch: char,
    ) -> CandidateMatches<'a> {
        self.base.matches_from(
            char_offset,
            ch,
            self.opts.is_case_sensitive,
            self.opts.char_eq,
        )
    }

    #[inline(always)]
    pub fn new(
        base: Candidate<'a>,
        bonus_slab: &'a mut BonusSlab,
        initial_char_class: CharClass,
        opts: CandidateOpts,
    ) -> Self {
        let bonuses = bonus_slab.alloc(base.char_len());
        Self { base, bonuses, initial_char_class, opts }
    }
}
