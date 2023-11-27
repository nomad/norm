# 📐 norm

[![Latest version]](https://crates.io/crates/norms)
[![Docs badge]][docs]
[![CI]](https://github.com/nomad/norm/actions)

[Latest version]: https://img.shields.io/crates/v/norms.svg
[Docs badge]: https://docs.rs/norms/badge.svg
[CI]: https://github.com/nomad/norm/actions/workflows/ci.yml/badge.svg

norm is ..

## Example usage

```rust
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

[docs]: https://docs.rs/norms
[crates.io]: https://crates.io
