use call_trace::{trace_with, CallContext};

struct My;

macro_rules! trace_target {
    ($self:expr) => ( $self.trace() )
}

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

    #[trace_with(trace_target!(self))]
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

#[test]
fn test() {
    My.foo();
}
