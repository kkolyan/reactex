
fn main() {
    let numbers = vec![1, 2, 3];
    let x = Some(&numbers[..]);
    match x {
        None => {}
        Some([first, middle @ .., last]) => {}
        _ => {}
    }
}