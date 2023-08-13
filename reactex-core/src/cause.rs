use std::rc::Rc;
use std::cell::RefCell;
use crate::opt_tiny_vec::OptTinyVec;

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

    pub(crate) fn create_consequence(&self, title: String) -> Cause {
        Cause {
            inner: Rc::new(RefCell::new(CauseInner {
                title,
                reasons: OptTinyVec::single(self.clone()),
            })),
        }
    }
}
