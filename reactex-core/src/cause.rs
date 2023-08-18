use crate::opt_tiny_vec::OptTinyVec;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone)]
pub struct Cause {
    inner: Rc<RefCell<CauseInner>>,
}

struct CauseInner {
    title: String,
    reasons: OptTinyVec<Cause>,
}

impl Cause {
    pub fn initial() -> Cause {
        Cause {
            inner: Rc::new(RefCell::new(CauseInner {
                title: "initial".to_string(),
                reasons: OptTinyVec::default(),
            })),
        }
    }

    pub(crate) fn consequence(title: String, causes: impl IntoIterator<Item = Cause>) -> Cause {
        Cause {
            inner: Rc::new(RefCell::new(CauseInner {
                title,
                reasons: OptTinyVec::from_iterable(causes),
            })),
        }
    }
}
