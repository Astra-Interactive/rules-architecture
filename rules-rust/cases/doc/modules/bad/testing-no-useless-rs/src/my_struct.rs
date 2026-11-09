#[derive(Debug, Clone, PartialEq)]
struct MyStruct {
    value: i32,
}

#[test]
fn given_struct_when_built_then_value_is_stored() {
    let my_struct = MyStruct { value: 10 };
    assert_eq!(10, my_struct.value);
}
