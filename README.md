# Trailing Closure Macro for Rust

A Rust procedural macro that enables Ruby-like trailing block syntax.

## Usage

Add to your Cargo.toml:
\\\	oml
[dependencies]
trailing_closure_macro = { git = \
https://github.com/YOUR_USERNAME/trailing_closure_macro.git\ }
\\\

Then in your code:
\\\ust
use trailing_closure_macro::with_block;

fn takes_closure(f: impl FnOnce()) {
    f();
}

fn main() {
    with_block! {takes_closure() {
        println!(\Hello
from
trailing
closure!\);
    }}
}
\\\

