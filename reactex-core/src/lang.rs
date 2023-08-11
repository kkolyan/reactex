use std::cell::Ref;

enum ErrCell<E> {
    Ok,
    None,
    Err(E),
}

pub fn ref_map_result_option<S, T, E>(
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
                err_cell = ErrCell::Ok;
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
