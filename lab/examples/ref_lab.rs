use crate::CustomError::Err001;
use std::cell::Ref;
use std::cell::RefCell;
use std::rc::Rc;

enum CustomError {
    Err001,
    Err002,
}

struct Object {}

struct Database {}

impl Database {
    fn get_something(&self, query: &str) -> Result<&Object, CustomError> {
        todo!()
    }
}

trait LifeCycledDatabase {}

// struct MyRef<'a> {
//     source: Ref<'a, Database>,
//     value:
// }

// we have it in Ref
// pub fn map<U: ?Sized, F>(orig: Ref<'b, T>, f: F) -> Ref<'b, U>
//     where
//         F: FnOnce(&T) -> &U
// {todo!()}
//
// pub fn filter_map<U: ?Sized, F>(orig: Ref<'b, T>, f: F) -> Result<Ref<'b, U>, Self>
//     where
//         F: FnOnce(&T) -> Option<&U>,
// {todo!()}

fn some_smart_map<S, T, E>(
    source: Ref<S>,
    mapper: impl FnOnce(&S) -> Result<&T, E>,
) -> Result<Ref<T>, E> {
    let mut err_cell: Option<E> = None;
    let result = Ref::filter_map(source, |it| match mapper(it) {
        Ok(it) => Some(it),
        Err(err) => {
            err_cell = Some(err);
            None
        }
    });
    result.map_err(|err| err_cell.unwrap())
}

enum ErrCell<E> {
    Ok,
    None,
    Err(E),
}

fn some_smart_map_2<S, T, E>(
    source: Ref<S>,
    mapper: impl FnOnce(&S) -> Result<Option<&T>, E>,
) -> Result<Option<Ref<T>>, E> {
    let mut err_cell: ErrCell<E> = ErrCell::Ok;
    let result = Ref::filter_map(source, |it| match mapper(it) {
        Ok(it) => match it {
            None => {
                err_cell = ErrCell::None;
                None
            }
            Some(it) => {
                err_cell = ErrCell::None;
                Some(it)
            }
        },
        Err(err) => {
            err_cell = ErrCell::Err(err);
            None
        }
    });
    match err_cell {
        ErrCell::Ok => Ok(result.ok()),
        ErrCell::None => Ok(None),
        ErrCell::Err(err) => Err(err),
    }
}

fn main() {
    let db: Rc<RefCell<Database>> = Rc::new(RefCell::new(Database {}));
    let _borrowed_result: Result<Ref<Object>, CustomError> =
        some_smart_map(db.borrow(), |it| Err(Err001));
    let _result2: Result<Option<Ref<Object>>, CustomError> =
        some_smart_map_2(db.borrow(), |it| Err(Err001));
}
