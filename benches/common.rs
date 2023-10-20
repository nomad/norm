use criterion::{
    measurement::WallTime,
    BenchmarkGroup,
    BenchmarkId,
    Throughput,
};
use norm::CaseSensitivity;

pub trait Metric {
    type Query<'a>: Copy + FromStr<'a>;

    fn dist(&self, query: Self::Query<'_>, candidate: &str);
    fn with_case_sensitivity(self, case_sensitivity: CaseSensitivity) -> Self;
    fn with_matched_ranges(self, matched_ranges: bool) -> Self;
}

pub trait FromStr<'a> {
    fn from_str(s: &'a str) -> Self;
}

#[inline]
fn bench<'a, M, C>(
    group: &mut BenchmarkGroup<WallTime>,
    id: BenchmarkId,
    metric: &M,
    query: &str,
    candidates: C,
) where
    M: Metric,
    C: IntoIterator<Item = &'a str>,
    C::IntoIter: ExactSizeIterator + Clone,
{
    let candidates = candidates.into_iter();

    group.throughput(Throughput::Elements(candidates.len() as u64));

    group.bench_function(id, |b| {
        let query = M::Query::from_str(query);

        b.iter(|| {
            for candidate in candidates.clone() {
                metric.dist(query, candidate);
            }
        })
    });
}

fn param(
    case: CaseSensitivity,
    with_ranges: bool,
    suffix: Option<&str>,
) -> String {
    let mut s = String::new();

    let case = match case {
        CaseSensitivity::Sensitive => "case_sensitive",
        CaseSensitivity::Insensitive => "case_insensitive",
        CaseSensitivity::Smart => "case_smart",
    };

    s.push_str(case);

    let ranges = if with_ranges { "_with_ranges" } else { "" };

    s.push_str(ranges);

    if let Some(suffix) = suffix {
        s.push('_');
        s.push_str(suffix);
    }

    s
}

#[inline]
pub fn short<M: Metric>(
    mut metric: M,
    suffix: Option<&str>,
    mut group: BenchmarkGroup<WallTime>,
) where
    M: Metric,
{
    for case in [
        CaseSensitivity::Sensitive,
        CaseSensitivity::Insensitive,
        CaseSensitivity::Smart,
    ] {
        for ranges in [true, false] {
            let id = BenchmarkId::new("short", param(case, ranges, suffix));

            metric =
                metric.with_case_sensitivity(case).with_matched_ranges(ranges);

            bench(
                &mut group,
                id,
                &metric,
                "jelly",
                core::iter::once("jellyfish"),
            );
        }
    }
}
