# 📐 norm

[![Latest version]](https://crates.io/crates/norms)
[![Docs badge]][docs]
[![CI]](https://github.com/nomad/norm/actions)

[Latest version]: https://img.shields.io/crates/v/norms.svg
[Docs badge]: https://docs.rs/norms/badge.svg
[CI]: https://github.com/nomad/norm/actions/workflows/ci.yml/badge.svg

norm is a collection of different distance metrics on stings. This problem is
sometimes referred to as "string similarity search", or more colloquially
"fuzzy matching".

## Available metrics

- `FzfV1`: port of the algorithm used by fzf when launching with `--algo=v1`;
- `FzfV2`: port of the algorithm used by fzf when launching without any extra
  flags or with `--algo=v2`;

## Performance

Performance is a top priority for this crate. Our goal is to have the fastest
implementation of every metric algorithm we provide, across all languages.
[Here][bench] you can find a number of benchmarks comparing norm's metrics to
each other, as well as to other popular libraries.

## Example usage

```rust
use norm::fzf::{FzfParser, FzfV2};
use norm::Metric;

let mut fzf = FzfV2::new();

let mut parser = FzfParser::new();

let query = parser.parse("aa");

let cities = ["Geneva", "Ulaanbaatar", "New York City", "Adelaide"];

let mut results = cities
    .iter()
    .copied()
    .filter_map(|city| fzf.distance(query, city).map(|dist| (city, dist)))
    .collect::<Vec<_>>();

// We sort the results by distance in ascending order, so that the best match
// will be at the front of the vector.
results.sort_by_key(|(_city, dist)| *dist);

assert_eq!(results.len(), 2);
assert_eq!(results[0].0, "Adelaide");
assert_eq!(results[1].0, "Ulaanbaatar");

// We can also find out which sub-strings of each candidate matched the query.

let mut ranges = Vec::new();

let _ = fzf.distance_and_ranges(query, results[0].0, &mut ranges);
assert_eq!(ranges.len(), 2);
assert_eq!(ranges[0], 0..1); // "A" in "Adelaide"
assert_eq!(ranges[1], 4..5); // "a" in "Adelaide"

ranges.clear();

let _ = fzf.distance_and_ranges(query, results[1].0, &mut ranges);
assert_eq!(ranges.len(), 1);
assert_eq!(ranges[0], 2..4); // The first "aa" in "Ulaanbaatar"
```

# A note on the crate's naming scheme

norm's `package.name` is `norms`, while its `lib.name` is `norm`. This is
because the package name has to be unique to be published to [crates.io], but
unfortunately `norm` is already taken by a crate squatter.
What this means is that you should import norm as `norms` in your `Cargo.toml`,
and `use` it as `norm` in your source code.

For example:

```toml
# Cargo.toml
[dependencies]
norms = { version = "0.1", features = ["fzf-v2"] }
```

```rust
// main.rs
use norm::fzf::FzfV2;

fn main() {
    println!("{:?}", FzfV2::new());
}
```

[bench]: https://github.com/noib3/fuzzy-benches
[crates.io]: https://crates.io
[docs]: https://docs.rs/norms
