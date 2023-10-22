use core::ops::Range;

use super::fzf_v1::*;
use crate::{CaseSensitivity, Match, Metric};

/// TODO: docs
#[cfg_attr(docsrs, doc(cfg(feature = "fzf-v2")))]
#[derive(Default)]
pub struct FzfV2 {
    /// TODO: docs
    case_sensitivity: CaseSensitivity,

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
                } else {
                    break;
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

        // Phase 3.
        //
        // Fill in score matrix H.

        let f0 = f[0];

        let width = last_idx - f0 + 1;

        let mut h = vec![0; m * width];

        h[f0..last_idx + 1].copy_from_slice(&h0[f0..last_idx + 1]);

        // Possible length of consecutive chunk at each position.
        let mut c = vec![0; m * width];

        c[f0..last_idx + 1].copy_from_slice(&c0[f0..last_idx + 1]);

        let f_sub = &mut f[1..];

        let p_sub = &query[1..][..f_sub.len()];

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
                let c =
                    unsafe { core::slice::from_raw_parts_mut(c_ptr, c_len) };
                &mut c[row + f - f0 - 1 - width..][..t_sub.len()]
            };

            let h_diag = {
                let h =
                    unsafe { core::slice::from_raw_parts_mut(h_ptr, h_len) };
                &mut h[row + f - f0 - 1 - width..][..t_sub.len()]
            };

            let h_left = {
                let h =
                    unsafe { core::slice::from_raw_parts_mut(h_ptr, h_len) };
                &mut h[row + f - f0 - 1..][..t_sub.len()]
            };

            for (offset, &char) in t_sub.iter().enumerate() {
                let col = offset + f;

                let mut s1 = 0;

                let mut consecutive = 0;

                let s2 = if is_in_gap {
                    h_left[offset].saturating_sub(penalty::GAP_EXTENSION)
                } else {
                    h_left[offset].saturating_sub(penalty::GAP_START)
                };

                if pchar == char {
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

                    if s1 + small_b < s2 {
                        s1 += b_sub[offset];
                        consecutive = 0;
                    } else {
                        s1 += small_b;
                    }
                }

                c_sub[offset] = consecutive;

                is_in_gap = s1 < s2;

                let score = s1.max(s2).max(0);

                if pidx == m - 1 && score > max_score {
                    max_score = score;
                    max_score_pos = col;
                }

                h_sub[offset] = score;
            }
        }

        let j = f0;

        Some(max_score)
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
        &self,
        query: FzfQuery<'_>, // helwo
        candidate: &str,     // Hello World!
    ) -> Option<Match<Self::Distance>> {
        if query.is_empty() {
            return None;
        }

        let score = self.fuzzy_match(query.raw(), candidate)?;

        let distance = FzfDistance::from_score(score);

        Some(Match::new(distance, Vec::new()))
    }
}
