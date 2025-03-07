use trailing_closure_macro::with_block;

fn takes_closure(f: impl FnOnce() -> ()) {
    f();
}

struct TestStruct;

impl TestStruct {
    fn method_with_closure(&self, f: impl FnOnce() -> ()) {
        f();
    }
}

#[test]
fn test_function_call() {
    let mut value = 0;
    
    // Fix: Remove the parentheses from the macro invocation
    with_block! {takes_closure() {
        value = 42;
    }}
    
    assert_eq!(value, 42);
}

#[test]
fn test_method_call() {
    let mut value = 0;
    let test_struct = TestStruct;
    
    // Fix: Remove the parentheses from the macro invocation
    with_block! {test_struct.method_with_closure() {
        value = 42;
    }}
    
    assert_eq!(value, 42);
}

#[test]
fn test_with_arguments() {
    fn function_with_args(x: i32, f: impl FnOnce() -> ()) {
        assert_eq!(x, 10);
        f();
    }
    
    let mut value = 0;
    
    // Fix: Remove the parentheses from the macro invocation
    with_block! {function_with_args(10) {
        value = 42;
    }}
    
    assert_eq!(value, 42);
}