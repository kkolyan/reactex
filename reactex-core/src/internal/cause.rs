use crate::utils::opt_tiny_vec::OptTinyVec;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone)]
pub struct Cause {
    #[allow(dead_code)]
    inner: Rc<RefCell<CauseInner>>,
}

#[allow(dead_code)]
struct CauseInner {
    title: &'static str,
    reasons: OptTinyVec<Cause>,
}

impl Cause {
    pub fn initial() -> Cause {
        Cause {
            inner: Rc::new(RefCell::new(CauseInner {
                title: "initial",
                reasons: OptTinyVec::default(),
            })),
        }
    }

    pub(crate) fn consequence(
        title: &'static str,
        causes: impl IntoIterator<Item = Cause>,
    ) -> Cause {
        Cause {
            inner: Rc::new(RefCell::new(CauseInner {
                title,
                reasons: OptTinyVec::from_iterable(causes),
            })),
        }
    }
}
