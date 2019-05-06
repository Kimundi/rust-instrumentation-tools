use call_trace::inject_with;

struct My;

macro_rules! short {
    ($self:expr) => ($self.trace())
}

impl My {
    #[inline]
    #[inject_with(self.trace())]
    /// test
    fn foo(&mut self) {
        self.bar();
    }

    #[inject_with(self.trace())]
    fn bar(&mut self) {
        let x = self.baz();
        let y = self.baz();
        println!("x + y = {}", x + y);
    }

    #[inject_with(short!(self))]
    fn baz(&mut self) -> i32 {
        15
    }

    fn trace<T: std::fmt::Debug, F: FnOnce() -> T>(&mut self) -> impl FnOnce(F) -> T {
        |f| {
            println!("before");
            let r = f();
            println!("after = {:?}", r);
            r
        }
    }
}

#[test]
fn test() {
    My.foo();
}
