# Calc Regex

An engine for compiling *calc-regular* expressions and parsing input against
them.

The concept of calc-regular expressions was introduced by Grosch, Koenig, and
Lucks with their publication [Taming the Length Field in Binary Data:
Calc-Regular Languages][1].
It is meant for formalizing data serialization languages.
This project aims to provide a practical implementation of that concept.

## Build

This project requires an installation of the [Rust programming language][2].
It is built using `cargo`, which comes with Rust.

Download dependencies and compile:

    cargo build

Builds will be placed in `target/debug/`.

Run tests:

    cargo test

Build documentation:

    cargo doc

Documentation will be placed in `target/doc/`.
This project’s starting page is `target/doc/calc_regex/index.html`.

Generate coverage report (requires `kcov`):

    cargo test --no-run
    ./cov

Coverage reports will be placed in `target/cov`.
The starting page is `target/cov/index.html`.

Clean builds:

    rm -rf target

## Usage

To use this project, create an new cargo package (if you haven’t already):

    cargo new <name>

Include `calc_regex` as a dependency in your `Cargo.toml`, e.g.:

    [dependencies]
    calc_regex = { path = "path/to/calc_regex" }

In your `main.rs` or `lib.rs`, include `calc_regex` and the macro `generate`:

    #[macro_use(generate)]
    extern crate calc_regex;

See `src/tests/` for usage examples.
See the documentation for explanation and complete reference of available
types and methods.


[1]: http://spw17.langsec.org/papers/grosch-taming-length-fiels.pdf
[2]: https://www.rust-lang.org