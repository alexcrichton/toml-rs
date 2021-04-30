extern crate toml;
use toml::Value;

macro_rules! bad {
    ($toml:expr, $msg:expr) => {
        match $toml.parse::<toml::Value>() {
            Ok(s) => panic!("parsed to: {:#?}", s),
            Err(e) => assert_eq!(e.to_string(), $msg),
        }
    };
}

#[test]
fn bad() {
    bad!("a = 01", "invalid number at line 1 column 6");
    bad!("a = 1__1", "invalid number at line 1 column 5");
    bad!("a = 1_", "invalid number at line 1 column 5");
    bad!("''", "expected an equals, found eof at line 1 column 3");
    bad!("a = 9e99999", "invalid number at line 1 column 5");

    bad!(
        "a = \"\u{7f}\"",
        "invalid character in string: `\\u{7f}` at line 1 column 6"
    );
    bad!(
        "a = '\u{7f}'",
        "invalid character in string: `\\u{7f}` at line 1 column 6"
    );

    bad!("a = -0x1", "invalid number at line 1 column 5");
    bad!(
        "a = 0x-1",
        "failed to parse datetime for key `a` at line 1 column 5"
    );

    // Dotted keys.
    bad!(
        "a.b.c = 1
         a.b = 2
        ",
        "duplicate key: `b` for key `a` at line 1 column 9"
    );
    bad!(
        "a = 1
         a.b = 2",
        "dotted key attempted to extend non-table type at line 1 column 5"
    );
    bad!(
        "a = {k1 = 1, k1.name = \"joe\"}",
        "dotted key attempted to extend non-table type at line 1 column 11"
    );
}

#[test]
fn inserting_value() {

    let insert_error = "failed to insert value";
    let cast_error = "failed to cast value";

    let mut some_value: Value = toml::from_str("a=1").expect("failed to create Value");
    some_value
        .insert("b", Value::Integer(2))
        .expect(insert_error);
    some_value
        .insert("c", Value::Integer(3))
        .expect(insert_error);

    assert_eq!(some_value["a"].as_integer().expect(cast_error), 1);
    assert_eq!(some_value["b"].as_integer().expect(cast_error), 2);
    assert_eq!(some_value["c"].as_integer().expect(cast_error), 3);
}
