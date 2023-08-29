#[test]
fn must_fail() -> Result<(), &'static str> {
    Err("die!")
}
#[test]
fn must_ok() -> Result<(), &'static str> {
    Ok(())
}
