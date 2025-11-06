fn add(a: usize, b: usize) -> usize {
    a + b
}

fn is_even(i: usize) -> bool {
    i.is_multiple_of(2)
}

#[test]
fn test_integration() {
    let result = add(2, 2);
    assert!(is_even(result));
}
