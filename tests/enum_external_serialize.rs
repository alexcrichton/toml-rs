#[macro_use]
extern crate serde_derive;
extern crate toml;

#[derive(Debug, Serialize, PartialEq)]
enum TheEnum {
    Plain,
    Tuple(i64, bool),
    NewType(String),
    Struct { value: i64 },
}

#[derive(Debug, Serialize, PartialEq)]
struct Val {
    val: TheEnum,
}

#[derive(Debug, Serialize, PartialEq)]
struct Multi {
    enums: Vec<TheEnum>,
}

#[test]
fn enum_unit_serializes_to_string_when_standalone() {
    assert_eq!(r#""Plain""#, toml::to_string(&TheEnum::Plain).unwrap());
}

#[test]
fn enum_tuple_serializes_to_inline_table() {
    assert_eq!(
        r#"{ Tuple = { 0 = -123, 1 = true } }"#,
        toml::to_string(&TheEnum::Tuple(-123, true)).unwrap()
    );
}
