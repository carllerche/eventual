# This crate is deprecated!

This crate is deprecated in favor of [futures](https://crates.io/crates/futures).

# Eventual - Futures & Streams for Rust

Eventual provides a Future & Stream abstraction for Rust as well as a
number of computation builders to operate on them.

[![Build Status](https://travis-ci.org/carllerche/eventual.svg?branch=master)](https://travis-ci.org/carllerche/eventual)

- [API documentation](http://carllerche.github.io/eventual/eventual/index.html)

## Usage

To use `Eventual`, first add this to your `Cargo.toml`:

```toml
[dependencies.eventual]
git = "https://github.com/carllerche/eventual"
```

Then, add this to your crate root:

```rust
extern crate eventual;
```
