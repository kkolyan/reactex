use std::backtrace::Backtrace;
use std::cell::RefCell;
use std::fmt::Display;
use std::fmt::Formatter;
use std::panic::catch_unwind;
use std::panic::set_hook;
use std::panic::UnwindSafe;
use std::thread;

use ctor::ctor;

#[derive(Debug)]
pub struct DetailedError {
    pub backtrace: Backtrace,
    pub message: String,
}

impl Display for DetailedError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.message)?;
        let backtrace = render_backtrace(&self.backtrace);
        writeln!(f, "{}", backtrace)?;
        Ok(())
    }
}

fn render_backtrace(backtrace: &Backtrace) -> String {
    let backtrace = format!("{}", backtrace);
    let backtrace = backtrace
        .replace(".\\src\\", ".\\reactex-core\\src\\")
        .replace(".\\tests\\", ".\\reactex-core\\tests\\");
    backtrace
}

#[ctor]
fn register_hook() {
    set_hook(Box::new(|info| {
        PANIC_INFO_STACK.with(|it| {
            let mut it = it.borrow_mut();
            match it.last_mut() {
                Some(ctx) => {
                    ctx.error = Some(DetailedError {
                        backtrace: Backtrace::capture(),
                        message: format!("{}", info),
                    });
                }
                None => {
                    let thread = thread::current();
                    let thread = thread.name().unwrap_or("<unknown>");
                    println!("thread panicked: {:?}\n", thread);
                    println!("{}", info);
                    println!("{}", render_backtrace(&Backtrace::capture()));
                }
            };
        });
    }));
}

struct Frame {
    error: Option<DetailedError>,
}

thread_local! {
    static PANIC_INFO_STACK: RefCell<Vec<Frame>> = RefCell::new(Vec::new())
}

pub fn catch_unwind_detailed<R>(f: impl FnOnce() -> R + UnwindSafe) -> Result<R, DetailedError> {
    PANIC_INFO_STACK.with(|it| it.borrow_mut().push(Frame { error: None }));
    let result = catch_unwind(f);
    let frame = PANIC_INFO_STACK.with(|it| it.borrow_mut().pop()).unwrap();
    match result {
        Ok(it) => Ok(it),
        Err(_) => Err(frame
            .error
            .expect("panic caught without details. this method requires exclusive set_hook.")),
    }
}
