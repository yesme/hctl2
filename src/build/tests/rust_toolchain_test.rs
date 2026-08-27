#[test]
fn uses_rust_2024() {
    let value = Some(42);
    let Some(answer) = value else {
        panic!("expected a value");
    };
    assert_eq!(answer, 42);
}
