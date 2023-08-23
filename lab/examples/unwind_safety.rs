#![feature(negative_impls)]

use std::cell::RefCell;
use std::panic::catch_unwind;
use std::panic::RefUnwindSafe;
use std::panic::UnwindSafe;

struct UnwindSafeWhenOwned {}
struct UnwindSafeWhenBorrowed {}

impl !RefUnwindSafe for UnwindSafeWhenOwned {}
impl !UnwindSafe for UnwindSafeWhenBorrowed {}

struct NotUnwindSafeWhenOwned {}
struct NotUnwindSafeWhenBorrowed {}

impl !RefUnwindSafe for NotUnwindSafeWhenBorrowed {}
impl !UnwindSafe for NotUnwindSafeWhenOwned {}

fn main() {
    // positives

    let owned = UnwindSafeWhenOwned {};
    catch_unwind(|| {
        let _y = owned;
    })
    .unwrap();

    let owned = RefCell::new(0);
    catch_unwind(|| {
        let _y = owned;
    })
    .unwrap();

    let borrowed = &UnwindSafeWhenBorrowed {};
    catch_unwind(|| {
        let _y = borrowed;
    })
    .unwrap();

    // negatives

    let borrowed = &NotUnwindSafeWhenBorrowed {};
    catch_unwind(|| {
        let _y = borrowed;
    })
    .unwrap();

    let owned = NotUnwindSafeWhenOwned {};
    catch_unwind(|| {
        let _y = owned;
    })
    .unwrap();

    let borrowed = &RefCell::new(0);
    catch_unwind(|| {
        let _y = borrowed;
    })
    .unwrap();

    // `&mut T` declared !UnwindSafe in stdlib, so `&mut 0` is not allowed to move in
    let owned = &mut 0;
    catch_unwind(|| {
        let _y = owned;
    })
    .unwrap();

    let owned = catch_unwind(|| NotUnwindSafeWhenOwned {}).unwrap();
}

trait UnwindSafeTrait: UnwindSafe {
    fn dodo() -> NotUnwindSafeWhenOwned;
}
