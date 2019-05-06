use call_trace::{trace, Event};

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
    call_trace::thread_register_callback(|_||_, event| {
        if event == Event::Call {
            println!("Hello world");
        }
    });
    call_trace::thread_register_callback(|prev|move|ctx, event| {
        if let Some(prev) = &prev {
            prev(ctx, event);
        }
        match event {
            Event::Call => println!("> {:?}", ctx.top()),
            Event::Return => println!("< {:?}", ctx.top()),
        }
    });

    foo();
}
