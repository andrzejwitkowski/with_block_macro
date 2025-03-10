```markdown
# Trailing Closure Macro for Rust

A Rust procedural macro that enables Ruby-like trailing block syntax for more elegant code.

## Usage

Add to your Cargo.toml:
```toml
[dependencies]
trailing_closure_macro = { git = "https://github.com/andrzejwitkowski/with_block_macro" }
```

Then in your code:

```rust
use trailing_closure_macro::with_block;

fn takes_closure(f: impl FnOnce()) {
    f();
}

fn main() {
    with_block! {takes_closure() {
        println!("Hello from trailing closure!");
    }}
}
 ```

## Examples
### Basic Function Call
```rust
fn process_data(callback: impl FnOnce()) {
    println!("Processing data...");
    callback();
    println!("Processing complete!");
}

// Instead of this:
process_data(|| {
    println!("Data is being processed");
});

// You can write this:
with_block! {process_data() {
    println!("Data is being processed");
}}
 ```

### Method Calls
```rust
struct DataProcessor;

impl DataProcessor {
    fn process(&self, on_complete: impl FnOnce()) {
        println!("Starting process...");
        on_complete();
        println!("Process finished!");
    }
}

let processor = DataProcessor;

with_block! {processor.process() {
    println!("Custom processing logic here");
}}
 ```

### With Arguments
```rust
fn analyze_number(num: i32, callback: impl FnOnce()) {
    println!("Analyzing number: {}", num);
    callback();
}

with_block! {analyze_number(42) {
    println!("Number analysis in progress");
}}
 ```

## Benefits
- More readable code for callback-heavy APIs
- Mimics block syntax from languages like Ruby
- Maintains Rust's type safety and performance
- Makes UI and event-driven code more elegant