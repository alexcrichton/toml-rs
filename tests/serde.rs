#![cfg(feature = "serde")]
#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]


extern crate serde;
extern crate toml;

use std::collections::{BTreeMap, HashSet};
use serde::{Deserialize, Serialize, Deserializer};

use toml::{Encoder, Decoder, DecodeError};
use toml::Value;
use toml::Value::{Table, Integer, Array, Float};

macro_rules! t {
    ($e:expr) => (match $e {
        Ok(t) => t,
        Err(e) => panic!("{} failed with {}", stringify!($e), e),
    })
}

macro_rules! encode( ($t:expr) => ({
    let mut e = Encoder::new();
    t!($t.serialize(&mut e));
    e.toml
}) );

macro_rules! decode( ($t:expr) => ({
    let mut d = Decoder::new($t);
    t!(Deserialize::deserialize(&mut d))
}) );

macro_rules! map( ($($k:ident, $v:expr),*) => ({
    let mut _m = BTreeMap::new();
    $(_m.insert(stringify!($k).to_string(), $v);)*
    _m
}) );

#[test]
fn smoke() {
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Foo { a: isize }

    let v = Foo { a: 2 };
    assert_eq!(encode!(v), map! { a, Integer(2) });
    assert_eq!(v, decode!(Table(encode!(v))));
}

#[test]
fn smoke_hyphen() {
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Foo { a_b: isize }

    let v = Foo { a_b: 2 };
    assert_eq!(encode!(v), map! { a_b, Integer(2) });
    assert_eq!(v, decode!(Table(encode!(v))));

    let mut m = BTreeMap::new();
    m.insert("a-b".to_string(), Integer(2));
    assert_eq!(v, decode!(Table(encode!(v))));
}

#[test]
fn nested() {
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Foo { a: isize, b: Bar }
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Bar { a: String }

    let v = Foo { a: 2, b: Bar { a: "test".to_string() } };
    assert_eq!(encode!(v),
               map! {
                   a, Integer(2),
                   b, Table(map! {
                       a, Value::String("test".to_string())
                   })
               });
    assert_eq!(v, decode!(Table(encode!(v))));
}

#[test]
fn application_decode_error() {
    #[derive(PartialEq, Debug)]
    struct Range10(usize);
    impl Deserialize for Range10 {
         fn deserialize<D: Deserializer>(d: &mut D) -> Result<Range10, D::Error> {
             let x: usize = try!(Deserialize::deserialize(d));
             if x > 10 {
                 Err(serde::de::Error::syntax("more than 10"))
             } else {
                 Ok(Range10(x))
             }
         }
    }
    let mut d_good = Decoder::new(Integer(5));
    let mut d_bad1 = Decoder::new(Value::String("not an isize".to_string()));
    let mut d_bad2 = Decoder::new(Integer(11));

    assert_eq!(Ok(Range10(5)), Deserialize::deserialize(&mut d_good));

    let err1: Result<Range10, _> = Deserialize::deserialize(&mut d_bad1);
    assert!(err1.is_err());
    let err2: Result<Range10, _> = Deserialize::deserialize(&mut d_bad2);
    assert!(err2.is_err());
}

#[test]
fn array() {
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Foo { a: Vec<isize> }

    let v = Foo { a: vec![1, 2, 3, 4] };
    assert_eq!(encode!(v),
               map! {
                   a, Array(vec![
                        Integer(1),
                        Integer(2),
                        Integer(3),
                        Integer(4)
                   ])
               });
    assert_eq!(v, decode!(Table(encode!(v))));
}

#[test]
fn tuple() {
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Foo { a: (isize, isize, isize, isize) }

    let v = Foo { a: (1, 2, 3, 4) };
    assert_eq!(encode!(v),
               map! {
                   a, Array(vec![
                        Integer(1),
                        Integer(2),
                        Integer(3),
                        Integer(4)
                   ])
               });
    assert_eq!(v, decode!(Table(encode!(v))));
}

#[test]
fn inner_structs_with_options() {
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Foo {
        a: Option<Box<Foo>>,
        b: Bar,
    }
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Bar {
        a: String,
        b: f64,
    }

    let v = Foo {
        a: Some(Box::new(Foo {
            a: None,
            b: Bar { a: "foo".to_string(), b: 4.5 },
        })),
        b: Bar { a: "bar".to_string(), b: 1.0 },
    };
    assert_eq!(encode!(v),
               map! {
                   a, Table(map! {
                       b, Table(map! {
                           a, Value::String("foo".to_string()),
                           b, Float(4.5)
                       })
                   }),
                   b, Table(map! {
                       a, Value::String("bar".to_string()),
                       b, Float(1.0)
                   })
               });
    assert_eq!(v, decode!(Table(encode!(v))));
}

#[test]
fn hashmap() {
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Foo {
        map: BTreeMap<String, isize>,
        set: HashSet<char>,
    }

    let v = Foo {
        map: {
            let mut m = BTreeMap::new();
            m.insert("foo".to_string(), 10);
            m.insert("bar".to_string(), 4);
            m
        },
        set: {
            let mut s = HashSet::new();
            s.insert('a');
            s
        },
    };
    assert_eq!(encode!(v),
        map! {
            map, Table(map! {
                foo, Integer(10),
                bar, Integer(4)
            }),
            set, Array(vec![Value::String("a".to_string())])
        }
    );
    assert_eq!(v, decode!(Table(encode!(v))));
}

#[test]
fn tuple_struct() {
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Foo(isize, String, f64);
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Bar {
        whee: Foo,
    }

    let v = Bar {
        whee: Foo(1, "foo".to_string(), 4.5)
    };
    assert_eq!(
        encode!(v),
        map! {
            whee, Value::Array(vec![
                Integer(1),
                Value::String("foo".to_string()),
                Float(4.5),
            ])
        }
    );
    assert_eq!(v, decode!(Table(encode!(v))));
}

#[test]
fn table_array() {
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Foo { a: Vec<Bar>, }
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Bar { a: isize }

    let v = Foo { a: vec![Bar { a: 1 }, Bar { a: 2 }] };
    assert_eq!(
        encode!(v),
        map! {
            a, Array(vec![
                Table(map!{ a, Integer(1) }),
                Table(map!{ a, Integer(2) }),
            ])
        }
    );
    assert_eq!(v, decode!(Table(encode!(v))));
}

#[test]
fn type_errors() {
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Foo { bar: isize }

    let mut d = Decoder::new(Table(map! {
        bar, Float(1.0)
    }));
    let a: Result<Foo, DecodeError> = Deserialize::deserialize(&mut d);
    // serde uses FromPrimitive, that's why this works
    match a {
        Ok(..) => panic!("should not have decoded"),
        Err(e) => {
            assert_eq!(format!("{}", e),
                       "expected a value of type `integer`, but \
                        found a value of type `float` for the key `bar`");
        }
    }
}

#[test]
fn missing_errors() {
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Foo { bar: isize }

    let mut d = Decoder::new(Table(map! {
    }));
    let a: Result<Foo, DecodeError> = Deserialize::deserialize(&mut d);
    match a {
        Ok(..) => panic!("should not have decoded"),
        Err(e) => {
            assert_eq!(format!("{}", e),
                       "expected a value for the key `bar`");
        }
    }
}

#[test]
fn parse_enum() {
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Foo { a: E }
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    enum E {
        Bar(isize),
        Baz(f64),
        Last(Foo2),
    }
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Foo2 {
        test: String,
    }

    let v = Foo { a: E::Bar(10) };
    // technically serde is correct here. a single element tuple still is a tuple and therefor
    // a sequence
    assert_eq!(
        encode!(v),
        map! { a, Integer(10) }
    );
    assert_eq!(v, decode!(Table(encode!(v))));

    let v = Foo { a: E::Baz(10.2) };
    assert_eq!(
        encode!(v),
        map! { a, Float(10.2) }
    );
    assert_eq!(v, decode!(Table(encode!(v))));

    let v = Foo { a: E::Last(Foo2 { test: "test".to_string() }) };
    assert_eq!(
        encode!(v),
        map! { a, Table(map! { test, Value::String("test".to_string()) }) }
    );
    assert_eq!(v, decode!(Table(encode!(v))));
}

#[test]
fn unused_fields() {
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Foo { a: isize }

    let v = Foo { a: 2 };
    let mut d = Decoder::new(Table(map! {
        a, Integer(2),
        b, Integer(5)
    }));
    assert_eq!(v, t!(Deserialize::deserialize(&mut d)));

    assert_eq!(d.toml, Some(Table(map! {
        b, Integer(5)
    })));
}

#[test]
fn unused_fields2() {
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Foo { a: Bar }
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Bar { a: isize }

    let v = Foo { a: Bar { a: 2 } };
    let mut d = Decoder::new(Table(map! {
        a, Table(map! {
            a, Integer(2),
            b, Integer(5)
        })
    }));
    assert_eq!(v, t!(Deserialize::deserialize(&mut d)));

    assert_eq!(d.toml, Some(Table(map! {
        a, Table(map! {
            b, Integer(5)
        })
    })));
}

#[test]
fn unused_fields3() {
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Foo { a: Bar }
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Bar { a: isize }

    let v = Foo { a: Bar { a: 2 } };
    let mut d = Decoder::new(Table(map! {
        a, Table(map! {
            a, Integer(2)
        })
    }));
    assert_eq!(v, t!(Deserialize::deserialize(&mut d)));

    assert_eq!(d.toml, None);
}

#[test]
fn unused_fields4() {
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Foo { a: BTreeMap<String, String> }

    let v = Foo { a: map! { a, "foo".to_string() } };
    let mut d = Decoder::new(Table(map! {
        a, Table(map! {
            a, Value::String("foo".to_string())
        })
    }));
    assert_eq!(v, t!(Deserialize::deserialize(&mut d)));

    assert_eq!(d.toml, None);
}

#[test]
fn unused_fields5() {
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Foo { a: Vec<String> }

    let v = Foo { a: vec!["a".to_string()] };
    let mut d = Decoder::new(Table(map! {
        a, Array(vec![Value::String("a".to_string())])
    }));
    assert_eq!(v, t!(Deserialize::deserialize(&mut d)));

    assert_eq!(d.toml, None);
}

#[test]
fn unused_fields6() {
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Foo { a: Option<Vec<String>> }

    let v = Foo { a: Some(vec![]) };
    let mut d = Decoder::new(Table(map! {
        a, Array(vec![])
    }));
    assert_eq!(v, t!(Deserialize::deserialize(&mut d)));

    assert_eq!(d.toml, None);
}

#[test]
fn unused_fields7() {
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Foo { a: Vec<Bar> }
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Bar { a: isize }

    let v = Foo { a: vec![Bar { a: 1 }] };
    let mut d = Decoder::new(Table(map! {
        a, Array(vec![Table(map! {
            a, Integer(1),
            b, Integer(2)
        })])
    }));
    assert_eq!(v, t!(Deserialize::deserialize(&mut d)));

    assert_eq!(d.toml, Some(Table(map! {
        a, Array(vec![Table(map! {
            b, Integer(2)
        })])
    })));
}

#[test]
fn empty_arrays() {
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Foo { a: Vec<Bar> }
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Bar;

    let v = Foo { a: vec![] };
    let mut d = Decoder::new(Table(map! {}));
    assert_eq!(v, t!(Deserialize::deserialize(&mut d)));
}

#[test]
fn empty_arrays2() {
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Foo { a: Option<Vec<Bar>> }
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Bar;

    let v = Foo { a: None };
    let mut d = Decoder::new(Table(map! {}));
    assert_eq!(v, t!(Deserialize::deserialize(&mut d)));

    let v = Foo { a: Some(vec![]) };
    let mut d = Decoder::new(Table(map! {
        a, Array(vec![])
    }));
    assert_eq!(v, t!(Deserialize::deserialize(&mut d)));
}
