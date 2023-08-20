use justerror::Error;

pub type WorldResult<T = ()> = Result<T, WorldError>;

#[Error]
#[derive(Eq, PartialEq)]
pub enum WorldError {
    Entity(#[from] EntityError),
    Component(#[from] ComponentError),
}

#[Error]
#[derive(Eq, PartialEq)]
pub enum ComponentError {
    NotFound,
}

#[Error]
#[derive(Eq, PartialEq)]
pub enum EntityError {
    NotExists,
    NotCommitted,
    IsStale,
}
