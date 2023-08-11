use std::str::FromStr;

pub enum ComponentError {
    NotFound(ComponentNotFoundError),
    NotFound1(i32),
    NotFound2(String),
}

pub enum ComponentNotFoundError {
    Mapping,
    Data,
}

fn dodo() -> Option<i32> {
    std::fs::read_to_string("dasf.mdas")
        .ok()
        .and_then(|s| i32::from_str(s.as_str()).ok())
}

#[inline(never)]
pub fn do_value() -> Result<i32, ComponentError> {
    dodo().ok_or(ComponentError::NotFound(ComponentNotFoundError::Data))
}

#[inline(never)]
pub fn do_fn() -> Result<i32, ComponentError> {
    dodo().ok_or_else(|| ComponentError::NotFound(ComponentNotFoundError::Data))
}

fn main() {}
