#![warn(missing_docs)]

/*!

# Example (simple)
```
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

fn main() {
    foo();
}
```

Output:

```text
[call-trace/tests/call-trace.rs:4] => foo()
[call-trace/tests/call-trace.rs:10] => bar()
[call-trace/tests/call-trace.rs:17] => baz()
[call-trace/tests/call-trace.rs:17] <= baz()
[call-trace/tests/call-trace.rs:17] => baz()
[call-trace/tests/call-trace.rs:17] <= baz()
x + y = 30
[call-trace/tests/call-trace.rs:10] <= bar()
[call-trace/tests/call-trace.rs:4] <= foo()
```

# Example (custom callback)

```
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

fn main() {
    call_trace::thread_register_callback(|_||_, event| {
        // discard the previous callback handler
        if event == Event::Call {
            println!("Hello world");
        }
    });
    call_trace::thread_register_callback(|prev|move|ctx, event| {
        // call the previous callback handler
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
```

Output

```text
Hello world
> CallContext { file: "call-trace/tests/call-trace-register.rs", line: 4, fn_name: "foo" }
Hello world
> CallContext { file: "call-trace/tests/call-trace-register.rs", line: 10, fn_name: "bar" }
Hello world
> CallContext { file: "call-trace/tests/call-trace-register.rs", line: 17, fn_name: "baz" }
< CallContext { file: "call-trace/tests/call-trace-register.rs", line: 17, fn_name: "baz" }
Hello world
> CallContext { file: "call-trace/tests/call-trace-register.rs", line: 17, fn_name: "baz" }
< CallContext { file: "call-trace/tests/call-trace-register.rs", line: 17, fn_name: "baz" }
x + y = 30
< CallContext { file: "call-trace/tests/call-trace-register.rs", line: 10, fn_name: "bar" }
< CallContext { file: "call-trace/tests/call-trace-register.rs", line: 4, fn_name: "foo" }
```

# Example (custom context)

```
use call_trace::{trace_with, CallContext};

struct My;

impl My {
    #[inline]
    #[trace_with(self.trace())]
    /// test
    fn foo(&mut self) {
        self.bar();
    }

    #[trace_with(self.trace())]
    fn bar(&mut self) {
        let x = self.baz();
        let y = self.baz();
        println!("x + y = {}", x + y);
    }

    #[trace_with(self.trace())]
    fn baz(&mut self) -> i32 {
        15
    }

    fn trace<T, F: FnOnce() -> T>(&mut self) -> impl FnOnce(F, CallContext) -> T {
        |f, ctx| {
            println!("> {:?}", ctx);
            let r = f();
            println!("< {:?}", ctx);
            r
        }
    }
}

fn main() {
    My.foo();
}
```

Output

```text
> CallContext { file: "call-trace/tests/call-trace-with.rs", line: 7, fn_name: "foo" }
> CallContext { file: "call-trace/tests/call-trace-with.rs", line: 13, fn_name: "bar" }
> CallContext { file: "call-trace/tests/call-trace-with.rs", line: 20, fn_name: "baz" }
< CallContext { file: "call-trace/tests/call-trace-with.rs", line: 20, fn_name: "baz" }
> CallContext { file: "call-trace/tests/call-trace-with.rs", line: 20, fn_name: "baz" }
< CallContext { file: "call-trace/tests/call-trace-with.rs", line: 20, fn_name: "baz" }
x + y = 30
< CallContext { file: "call-trace/tests/call-trace-with.rs", line: 13, fn_name: "bar" }
< CallContext { file: "call-trace/tests/call-trace-with.rs", line: 7, fn_name: "foo" }
```
*/

pub use call_trace_macro::trace;
pub use call_trace_macro::trace_with;
pub use call_trace_macro::inject_with;
use std::cell::RefCell;
use std::rc::Rc;

/// A callback. It gets called by `#[trace]`.
pub type Callback = Rc<dyn Fn(&mut Context, Event)>;

/// The thread-local callback context for `#[trace]`.
pub struct Context {
    callback: Option<Callback>,
    stack: Vec<CallContext>,
}
impl Context {
    fn new() -> Self {
        let mut r = Context {
            callback: None,
            stack: Vec::new(),
        };
        r.register_callback(|_| Self::default_callback);
        r
    }

    /// The default callback registered by `Self::new()`.
    pub fn default_callback(ctx: &mut Context, event: Event) {
        match event {
            Event::Call => {
                eprintln!("[{}:{}] => {}()", ctx.top().file, ctx.top().line, ctx.top().fn_name);
            }
            Event::Return => {
                eprintln!("[{}:{}] <= {}()", ctx.top().file, ctx.top().line, ctx.top().fn_name);
            }
        }

    }

    /// Returns the current call stack.
    pub fn stack(&self) -> &[CallContext] {
        &self.stack
    }

    /// Returns the top of the current call stack.
    pub fn top(&self) -> &CallContext {
        self.stack.last().expect("something went wrong with the callstack")
    }

    /// Registers a callback. A previously registered callback is
    /// passed to the outer closure.
    pub fn register_callback<F, G>(&mut self, f: F)
    where F: FnOnce(Option<Callback>) -> G,
          G: Fn(&mut Context, Event) + 'static
    {
        self.callback = Some(Rc::new(f(self.callback.take())));
    }

    /// Unregisters a callback.
    pub fn unregister_callback(&mut self) -> Option<Callback> {
        self.callback.take()
    }
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
/// Contains information about the current call site.
pub struct CallContext {
    /// Current file, as returned by `file!()`
    pub file: &'static str,

    /// Current line, as returned by `line!()`. Will point at the `#[trace]` attribute.
    pub line: u32,

    /// Name of the called function. Does not contain the module path.
    pub fn_name: &'static str,
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
/// Indicates why the callback got called by `#[trace]`
pub enum Event {
    /// The function just got called.
    Call,

    /// The function is about to return.
    Return,
}

thread_local! {
    static CONTEXT: RefCell<Context> = RefCell::new(Context::new());
}

/// Access the thread-local context of `#[trace]`
pub fn thread_access_with<T, F: FnOnce(&mut Context) -> T>(f: F) -> T {
    CONTEXT.with(move |ctx| f(&mut ctx.borrow_mut()))
}

/// Registers a callback in the thread-local context. This is a shortcut
/// for accessing the `Context` via `thread_access_with()`.
pub fn thread_register_callback<F, G>(f: F)
    where F: FnOnce(Option<Callback>) -> G,
          G: Fn(&mut Context, Event) + 'static
{
    thread_access_with(move |ctx| {
        ctx.register_callback(f)
    })
}

/// Unregisters a callback in the thread-local context. This is a shortcut
/// for accessing the `Context` via `thread_access_with()`.
pub fn thread_unregister_callback() -> Option<Callback> {
    thread_access_with(move |ctx| {
        ctx.unregister_callback()
    })
}

fn on_event(cctx: &CallContext, event: Event) {
    CONTEXT.with(|ctx| {
        let mut ctx = ctx.borrow_mut();
        if let Event::Call = event {
            ctx.stack.push(cctx.clone());
        }
        let ctx = &mut *ctx;
        if let Some(cb) = &ctx.callback {
            let cb: Callback = cb.clone();
            cb(ctx, event);
        }
        if let Event::Return = event {
            ctx.stack.pop().expect("something went wrong with the call stack");
        }
    })
}

/// Called by `#[trace]`.
pub fn on_trace<T, F: FnOnce() -> T>(f: F, ctx: CallContext) -> T {
    on_event(&ctx, Event::Call);
    let r = f();
    on_event(&ctx, Event::Return);
    r
}
