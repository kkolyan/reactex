use crate::utils::opt_tiny_vec::OptTinyVec;
use std::fmt::Debug;
use std::fmt::Display;
use std::fmt::Formatter;
use std::rc::Rc;

#[derive(Clone)]
pub struct Cause {
    #[allow(dead_code)]
    inner: Rc<CauseInner>,
}

impl Display for Cause {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        print_tree(0, self, f)
    }
}

fn print_tree(indent: usize, cause: &Cause, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}-> {}", "  ".repeat(indent), cause.inner.title)?;
    for reason in cause.inner.reasons.iter() {
        print_tree(indent + 1, reason, f)?;
    }
    Ok(())
}

impl Debug for Cause {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

#[allow(dead_code)]
struct CauseInner {
    title: &'static str,
    reasons: OptTinyVec<Cause>,
}

impl Cause {
    pub fn initial() -> Cause {
        Cause {
            inner: Rc::new(CauseInner {
                title: "initial",
                reasons: OptTinyVec::default(),
            }),
        }
    }

    pub(crate) fn consequence(
        title: &'static str,
        causes: impl IntoIterator<Item = Cause>,
    ) -> Cause {
        Cause {
            inner: Rc::new(CauseInner {
                title,
                reasons: OptTinyVec::from_iterable(causes),
            }),
        }
    }
}
