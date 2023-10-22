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

        // println!("f: {f:?}");
        // println!("t: {t:?}");
        // println!("b: {b:?}");
        // println!("h0: {h0:?}");
        // println!("c0: {c0:?}");
        // println!("last_idx: {last_idx:?}");

        // Phase 3.
        //
        // Fill in score matrix H.

        let score = phase_3(
            &mut f,
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

fn phase_2() {}

fn phase_3(
    f: &mut [usize],
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

    let f_sub = &mut f[1..];

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
