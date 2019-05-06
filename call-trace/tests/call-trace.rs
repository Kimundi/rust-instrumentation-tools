use call_trace::trace;

#[inline]
#[trace]
/// test
fn foo() {
    bar();
}

#[trace]
fn bar() {
    let x = baz();
    let y = baz();
    println!("x + y = {}", x + y);
}

#[trace]
fn baz() -> i32 {
    15
}

#[test]
fn test() {
    foo();
}
