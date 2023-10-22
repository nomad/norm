use core::ops::{Index, IndexMut};

use super::fzf_v1::*;
use crate::{CaseMatcher, CaseSensitivity, Match, Metric};

/// TODO: docs
#[cfg_attr(docsrs, doc(cfg(feature = "fzf-v2")))]
#[derive(Clone, Default)]
pub struct FzfV2 {
    /// TODO: docs
    bonus_vector_slab: BonusVectorSlab,

    /// TODO: docs
    candidate_slab: CandidateSlab,

    /// TODO: docs
    case_sensitivity: CaseSensitivity,

    /// TODO: docs
    matched_indices_slab: MatchedIndicesSlab,

    /// TODO: docs
    scoring_matrix_slab: ScoringMatrixSlab,

    /// TODO: docs
    scheme: scheme::Scheme,

    /// TODO: docs
    with_matched_ranges: bool,
}

impl FzfV2 {
    /// TODO: docs
    #[inline]
    fn fuzzy_match(&self, query: &str, candidate: &str) -> Option<Score> {
        // Phase 1.

        let m = query.len();

        let n = candidate.len();

        let mut idx = 0;

        let mut h0 = vec![0; n];

        let mut c0 = vec![0; n];

        let mut b = vec![0; n];

        let mut f = vec![0; m];

        let t = candidate.chars().collect::<Vec<_>>();

        // Phase 2.

        let mut max_score = 0;

        let mut max_score_pos = 0;

        let mut query_chars = query.chars();

        let mut query_char = query_chars.next().expect("query is not empty");

        let mut last_idx = 0;

        let first_query_char = query_char;

        let mut prev_h0: Score = 0;

        let mut prev_class = self.scheme.initial_char_class;

        let mut is_in_gap = false;

        let h0_sub = &mut h0[..];

        let c0_sub = &mut c0[..];

        let b_sub = &mut b[..];

        let case_matcher = self.case_sensitivity.matcher(query);

        let mut pidx = 0;

        for (offset, candidate_char) in candidate.char_indices() {
            let char_class = char_class(candidate_char, &self.scheme);

            let bonus = bonus(prev_class, char_class, &self.scheme);

            b_sub[offset] = bonus;

            prev_class = char_class;

            if case_matcher.eq(query_char, candidate_char) {
                last_idx = offset;
                f[pidx] = offset;
                pidx += 1;

                if let Some(next_char) = query_chars.next() {
                    query_char = next_char;
                }
            }

            if case_matcher.eq(first_query_char, candidate_char) {
                let score =
                    bonus::MATCH + bonus * bonus::FIRST_QUERY_CHAR_MULTIPLIER;

                h0_sub[offset] = score;
                c0_sub[offset] = 1;

                if m == 1 && score > max_score {
                    max_score = score;
                    max_score_pos = offset;
                    if bonus >= bonus::BOUNDARY {
                        break;
                    }
                }

                is_in_gap = false;
            } else {
                let penalty = if is_in_gap {
                    penalty::GAP_EXTENSION
                } else {
                    penalty::GAP_START
                };
                h0_sub[offset] = prev_h0.saturating_sub(penalty);
                c0_sub[offset] = 0;
                is_in_gap = true;
            }

            prev_h0 = h0_sub[offset];
        }

        if pidx != m {
            return None;
        }

        if m == 1 {
            // TODO: return score.
            return Some(max_score);
        }

        println!("f: {f:?}");
        println!("t: {t:?}");
        println!("b: {b:?}");
        println!("h0: {h0:?}");
        println!("c0: {c0:?}");
        println!("last_idx: {last_idx:?}");

        // panic!();

        // Phase 3.
        //
        // Fill in score matrix H.

        let score = phase_3(
            &f,
            &t,
            &b,
            &h0,
            &c0,
            query,
            last_idx,
            self.case_sensitivity,
        );

        Some(score)
    }

    /// TODO: docs
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// TODO: docs
    #[inline]
    pub fn with_case_sensitivity(
        mut self,
        case_sensitivity: CaseSensitivity,
    ) -> Self {
        self.case_sensitivity = case_sensitivity;
        self
    }

    /// TODO: docs
    #[inline]
    pub fn with_matched_ranges(mut self, matched_ranges: bool) -> Self {
        self.with_matched_ranges = matched_ranges;
        self
    }

    /// TODO: docs
    #[inline]
    pub fn with_scoring_scheme(mut self, scheme: FzfScheme) -> Self {
        self.scheme = match scheme {
            FzfScheme::Default => scheme::DEFAULT,
            FzfScheme::Path => scheme::PATH,
            FzfScheme::History => scheme::HISTORY,
        };
        self
    }
}

impl Metric for FzfV2 {
    type Query<'a> = FzfQuery<'a>;

    type Distance = FzfDistance;

    #[inline]
    fn distance(
        &mut self,
        query: FzfQuery<'_>, // helwo
        candidate: &str,     // Hello World!
    ) -> Option<Match<Self::Distance>> {
        if query.is_empty() {
            return None;
        }

        let query = query.raw();

        let case_matcher = self.case_sensitivity.matcher(query);

        let candidate = self.candidate_slab.alloc(candidate);

        let (matched_indices, bonus_vector) = matched_indices(
            &mut self.matched_indices_slab,
            &mut self.bonus_vector_slab,
            query,
            candidate,
            &case_matcher,
            &self.scheme,
        )?;

        let (scoring_matrix, score) = score(
            &mut self.scoring_matrix_slab,
            query,
            candidate,
            &matched_indices,
            &bonus_vector,
            &case_matcher,
            &self.scheme,
        );

        println!("matched_indices: {:?}", matched_indices);
        println!("bonus_vector: {:?}", bonus_vector);

        todo!()

        // let distance = FzfDistance::from_score(score);

        // Some(Match::new(distance, Vec::new()))
    }
}

/// TODO: docs
#[inline]
fn matched_indices<'idx, 'bonus>(
    indices_slab: &'idx mut MatchedIndicesSlab,
    bonuses_slab: &'bonus mut BonusVectorSlab,
    query: &str,
    candidate: Candidate<'_>,
    case_matcher: &CaseMatcher,
    scheme: &scheme::Scheme,
) -> Option<(MatchedIndices<'idx>, BonusVector<'bonus>)> {
    let mut query_chars = query.chars();

    let mut query_char = query_chars.next().expect("query is not empty");

    let mut prev_class = scheme.initial_char_class;

    let mut bonuses = bonuses_slab.alloc(candidate);

    let mut matched_idxs = indices_slab.alloc(query);

    for (char_idx, candidate_char) in candidate.char_idxs() {
        let char_class = char_class(candidate_char, scheme);
        let bonus = bonus(prev_class, char_class, scheme);
        prev_class = char_class;

        bonuses[char_idx] = bonus;

        if case_matcher.eq(query_char, candidate_char) {
            matched_idxs.push(char_idx);

            if let Some(next_char) = query_chars.next() {
                query_char = next_char;
            }
        }
    }

    // TODO: use query.char_len()
    if matched_idxs.len() == query.chars().count() {
        Some((matched_idxs, bonuses))
    } else {
        None
    }
}

/// TODO: docs
#[inline]
fn score<'scoring>(
    matrix_slab: &'scoring mut ScoringMatrixSlab,
    query: &str,
    candidate: Candidate,
    matched_indices: &MatchedIndices,
    bonus_vector: &BonusVector,
    case_matcher: &CaseMatcher,
    scheme: &scheme::Scheme,
) -> (ScoringMatrix<'scoring>, Score) {
    todo!();
}

fn phase_3(
    f: &[usize],
    t: &[char],
    b: &[Score],
    h0: &[Score],
    c0: &[usize],
    query: &str,
    last_idx: usize,
    case_sensitivity: CaseSensitivity,
) -> Score {
    let m = query.len();

    let mut max_score = 0;

    let mut max_score_pos = 0;

    let f0 = f[0];

    let width = last_idx - f0 + 1;

    let mut h = vec![0; m * width];

    h[..last_idx + 1 - f0].copy_from_slice(&h0[f0..last_idx + 1]);

    // Possible length of consecutive chunk at each position.
    let mut c = vec![0; m * width];

    c[..last_idx + 1 - f0].copy_from_slice(&c0[f0..last_idx + 1]);

    let f_sub = &f[1..];

    let p_sub = &query[1..][..f_sub.len()];

    let case_matcher = case_sensitivity.matcher(query);

    for (offset, f) in f_sub.iter().enumerate() {
        let c_len = c.len();
        let c_ptr = c.as_mut_ptr();

        let h_len = h.len();
        let h_ptr = h.as_mut_ptr();

        let pchar = p_sub.chars().nth(offset).unwrap();
        let pidx = offset + 1;
        let row = pidx * width;
        let mut is_in_gap = false;
        let t_sub = &t[*f..last_idx + 1];
        let b_sub = &b[*f..][..t_sub.len()];
        let c_sub = &mut c[row + f - f0..][..t_sub.len()];
        let h_sub = &mut h[row + f - f0..][..t_sub.len()];

        let c_diag = {
            let c = unsafe { core::slice::from_raw_parts_mut(c_ptr, c_len) };
            &mut c[row + f - f0 - 1 - width..][..t_sub.len()]
        };

        let h_diag = {
            let h = unsafe { core::slice::from_raw_parts_mut(h_ptr, h_len) };
            &mut h[row + f - f0 - 1 - width..][..t_sub.len()]
        };

        let h_left = {
            let h = unsafe { core::slice::from_raw_parts_mut(h_ptr, h_len) };
            &mut h[row + f - f0 - 1..][..t_sub.len()]
        };

        // println!("-------------");
        // println!("pchar: {:?}", pchar);
        // println!("pidx: {:?}", pidx);
        // println!("row: {:?}", row);
        // println!("inGap: {:?}", is_in_gap);
        // println!("Tsub: {:?}", t_sub);
        // println!("Bsub: {:?}", b_sub);
        // println!("Csub: {:?}", c_sub);
        // println!("Cdiag: {:?}", c_diag);
        // println!("Hsub: {:?}", h_sub);
        // println!("Hdiag: {:?}", h_diag);
        // println!("Hleft: {:?}", h_left);
        // println!("-------------");

        h_left[0] = 0;

        for (offset, &char) in t_sub.iter().enumerate() {
            let col = offset + f;

            let mut s1 = 0;

            let mut consecutive = 0;

            let s2 = if is_in_gap {
                h_left[offset] as i32 - penalty::GAP_EXTENSION as i32
            } else {
                h_left[offset] as i32 - penalty::GAP_START as i32
            };

            if case_matcher.eq(pchar, char) {
                s1 = h_diag[offset] + bonus::MATCH;
                let mut small_b = b_sub[offset];
                consecutive = c_diag[offset] + 1;

                if consecutive > 1 {
                    let fb = b[col - consecutive + 1];
                    if small_b >= bonus::BOUNDARY && small_b > fb {
                        consecutive = 1;
                    } else {
                        small_b = bonus::CONSECUTIVE.max(fb).max(small_b);
                    }
                }

                if (s1 as i32) + (small_b as i32) < s2 {
                    s1 += b_sub[offset];
                    consecutive = 0;
                } else {
                    s1 += small_b;
                }
            }

            // println!("-------------");
            // println!("s1: {:?}", s1);
            // println!("s2: {:?}", s2);
            // println!("offset: {:?}", offset);

            c_sub[offset] = consecutive;

            is_in_gap = (s1 as i32) < s2;

            let score = (s1 as i32).max(s2).max(0) as Score;

            if pidx == m - 1 && score > max_score {
                max_score = score;
                max_score_pos = col;
            }

            // println!("-------------");
            // println!("score: {:?}", score);
            // println!("offset: {:?}", offset);

            h_sub[offset] = score;
        }
    }

    // println!("f0: {:?}", f0);
    // println!("width: {:?}", width);
    // println!("h: {:?}", h);
    // println!("c: {:?}", c);
    // println!("f_sub: {:?}", f_sub);
    // println!("p_bub: {:?}", p_sub);

    let j = f0;

    max_score
}

/// TODO: docs
#[derive(Clone)]
struct CandidateSlab {
    chars: Vec<char>,
    char_indices: Vec<usize>,
}

impl Default for CandidateSlab {
    #[inline]
    fn default() -> Self {
        let chars = vec!['\0'; 16];
        let char_indices = vec![0; 16];
        Self { chars, char_indices }
    }
}

impl CandidateSlab {
    /// TODO: docs
    #[inline]
    fn alloc<'a>(&'a mut self, candidate: &str) -> Candidate<'a> {
        // Here we compare the byte length of the candidate string with the
        // current char length of the slab. This is fine since the byte length
        // is always greater than or equal to the char length.
        //
        // The worst case scenario is that we allocate more space than we need
        // to, but that's fine since we'll reuse the space later.
        if candidate.len() > self.chars.len() {
            self.chars.resize(candidate.len(), '\0');
            self.char_indices.resize(candidate.len(), 0);
        }

        let mut len = 0;

        for (offset, char) in candidate.char_indices() {
            self.chars[len] = char;
            self.char_indices[len] = offset;
            len += 1;
        }

        Candidate {
            chars: &self.chars[..len],
            char_offsets: &self.char_indices[..len],
        }
    }
}

/// TODO: docs
#[derive(Clone, Copy)]
struct Candidate<'a> {
    chars: &'a [char],
    char_offsets: &'a [usize],
}

impl core::fmt::Debug for Candidate<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.chars.iter().collect::<String>().fmt(f)
    }
}

/// TODO: docs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CandidateCharIdx(usize);

impl<'a> Candidate<'a> {
    /// TODO: docs
    #[inline]
    fn chars(&self) -> impl Iterator<Item = char> + '_ {
        self.chars.iter().copied()
    }

    /// TODO: docs
    #[inline]
    fn char_idxs(
        &self,
    ) -> impl Iterator<Item = (CandidateCharIdx, char)> + '_ {
        self.chars
            .iter()
            .enumerate()
            .map(|(idx, &char)| (CandidateCharIdx(idx), char))
    }

    /// TODO: docs
    #[inline]
    fn char_len(&self) -> usize {
        self.chars.len()
    }

    /// TODO: docs
    #[inline]
    fn char_offsets(&self) -> impl Iterator<Item = (usize, char)> + '_ {
        self.char_offsets
            .iter()
            .zip(self.chars)
            .map(|(&offset, &char)| (offset, char))
    }

    /// TODO: docs
    #[inline]
    fn nth_char(&self, idx: usize) -> char {
        self.chars[idx]
    }

    /// TODO: docs
    #[inline]
    fn nth_char_offset(&self, idx: usize) -> usize {
        self.char_offsets[idx]
    }
}

/// TODO: docs
#[derive(Clone)]
struct MatchedIndicesSlab {
    vec: Vec<CandidateCharIdx>,
}

impl Default for MatchedIndicesSlab {
    #[inline]
    fn default() -> Self {
        Self { vec: vec![CandidateCharIdx(0); 16] }
    }
}

impl MatchedIndicesSlab {
    #[inline]
    /// TODO: docs
    fn alloc<'a>(&'a mut self, query: &str) -> MatchedIndices<'a> {
        let char_len = query.chars().count();

        if char_len > self.vec.len() {
            self.vec.resize(char_len, CandidateCharIdx(0));
        }

        MatchedIndices::new(&mut self.vec[..char_len])
    }
}

/// TODO: docs
struct MatchedIndices<'a> {
    indices: &'a mut [CandidateCharIdx],
    len: usize,
}

impl<'a> MatchedIndices<'a> {
    #[inline]
    fn new(indices: &'a mut [CandidateCharIdx]) -> Self {
        Self { indices, len: 0 }
    }

    /// TODO: docs
    #[inline]
    fn push(&mut self, idx: CandidateCharIdx) {
        self.indices[self.len] = idx;
        self.len += 1;
    }

    /// TODO: docs
    #[inline]
    fn len(&self) -> usize {
        self.len
    }
}

impl core::fmt::Debug for MatchedIndices<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.indices[..self.len].fmt(f)
    }
}

/// TODO: docs
#[derive(Clone)]
struct BonusVectorSlab {
    vec: Vec<Score>,
}

impl Default for BonusVectorSlab {
    #[inline]
    fn default() -> Self {
        Self { vec: vec![0; 16] }
    }
}

impl BonusVectorSlab {
    /// TODO: docs
    #[inline]
    fn alloc<'a>(&'a mut self, candidate: Candidate) -> BonusVector<'a> {
        let char_len = candidate.char_len();

        if char_len > self.vec.len() {
            self.vec.resize(char_len, 0);
        }

        BonusVector { indices: &mut self.vec[..char_len] }
    }
}

/// TODO: docs
struct BonusVector<'a> {
    indices: &'a mut [Score],
}

impl core::fmt::Debug for BonusVector<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.indices.fmt(f)
    }
}

impl Index<CandidateCharIdx> for BonusVector<'_> {
    type Output = Score;

    #[inline]
    fn index(&self, index: CandidateCharIdx) -> &Self::Output {
        &self.indices[index.0]
    }
}

impl IndexMut<CandidateCharIdx> for BonusVector<'_> {
    #[inline]
    fn index_mut(&mut self, index: CandidateCharIdx) -> &mut Self::Output {
        &mut self.indices[index.0]
    }
}

/// TODO: docs
#[derive(Default, Clone)]
struct ScoringMatrixSlab {
    vec: Vec<Score>,
}

impl ScoringMatrixSlab {
    /// TODO: docs
    #[inline]
    fn alloc<'a>(
        &'a mut self,
        query: &str,
        candidate: Candidate,
    ) -> ScoringMatrix<'a> {
        todo!();
    }
}

/// TODO: docs
#[derive(Default)]
struct ScoringMatrix<'a> {
    vec: &'a [Score],
}
