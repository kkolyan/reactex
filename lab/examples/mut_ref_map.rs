fn main() {
    let indices: Vec<usize> = vec![1, 2];
    let mut values = vec![7, 8, 9, 12];

    let filtered_values = indices.iter().map(|it| values.get_mut(*it).unwrap());
    for x in filtered_values {
        *x *= 10;
    }

    assert_eq!(values, vec![7, 80, 90, 12]);
}
