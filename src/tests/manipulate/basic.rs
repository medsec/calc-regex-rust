//! Tests for basic manipulation of `CalcRegex`es.

use ::*;
use calc_regex::Inner;

///////////////////////////////////////////////////////////////////////////////
//      Set Root
///////////////////////////////////////////////////////////////////////////////

#[test]
fn set_root() {
    #![allow(unused_variables)]
    let mut calc_regex = generate! {
        foo := "foo!";
        bar := "bar";
    };
    calc_regex.set_root_by_name("foo").unwrap();
    let root = calc_regex.get_root();
    assert_eq!(root.name, Some("foo".to_owned()));
    assert_eq!(root.length_bound, Some(4));
    if let Inner::Regex(ref regex) = root.inner {
        assert_eq!(regex.as_str(), "^(?-u:foo!)$");
    } else {
        panic!("Unexpected Inner: {:?}", root.inner);
    }
}

#[test]
fn set_root_invalid_name() {
    #![allow(unused_variables)]
    let mut calc_regex = generate! {
        foo := "foo!";
        bar := "bar";
    };
    let err = calc_regex.set_root_by_name("baz").unwrap_err();
    if let NameError::NoSuchName { ref name } = err {
        assert_eq!(name, "baz");
    } else {
        panic!("Unexpected error: {:?}", err);
    }
}

///////////////////////////////////////////////////////////////////////////////
//      Set Length Bounds
///////////////////////////////////////////////////////////////////////////////

#[test]
fn set_root_length_bound() {
    let mut calc_regex = generate! {
        foo = "f", "o"*, "!";
    };
    calc_regex.set_root_length_bound(7);
    let root = calc_regex.get_root();
    assert_eq!(root.name, Some("foo".to_owned()));
    assert_eq!(root.length_bound, Some(7));
    if let Inner::Regex(ref regex) = root.inner {
        assert_eq!(regex.as_str(), "^(?-u:fo*!)$");
    } else {
        panic!("Unexpected Inner: {:?}", root.inner);
    }
}

#[test]
fn set_length_bound() {
    let mut calc_regex = generate! {
        foo = "f", "o"*, "!";
    };
    calc_regex.set_length_bound("foo", 7).unwrap();
    let root = calc_regex.get_root();
    assert_eq!(root.name, Some("foo".to_owned()));
    assert_eq!(root.length_bound, Some(7));
    if let Inner::Regex(ref regex) = root.inner {
        assert_eq!(regex.as_str(), "^(?-u:fo*!)$");
    } else {
        panic!("Unexpected Inner: {:?}", root.inner);
    }
}

#[test]
fn set_length_bound_invalid() {
    let mut calc_regex = generate! {
        foo = "f", "o"*, "!";
    };
    let err = calc_regex.set_length_bound("bar", 7).unwrap_err();
    if let NameError::NoSuchName { ref name } = err {
        assert_eq!(name, "bar");
    } else {
        panic!("Unexpected error: {:?}", err);
    }
}

#[test]
fn set_length_bound_various() {
    let mut calc_regex = generate! {
        foo = "f", "o"*, "!";
        bar = "b", "a"*, "r!";
        foobar := foo, bar;
        baz := foobar, bar;
    };
    calc_regex.set_root_length_bound(23);
    calc_regex.set_length_bound("foo", 7).unwrap();
    calc_regex.set_length_bound("bar", 8).unwrap();
    let root = calc_regex.get_root();
    assert_eq!(root.name, Some("baz".to_owned()));
    assert_eq!(root.length_bound, Some(23));
    if let Inner::Concat(lhs, rhs) = root.inner {
        let lhs = calc_regex.get_node(lhs);
        assert_eq!(lhs.name, Some("foobar".to_owned()));
        assert_eq!(lhs.length_bound, None);
        if let Inner::Concat(lhs, rhs) = lhs.inner {
            let lhs = calc_regex.get_node(lhs);
            assert_eq!(lhs.name, Some("foo".to_owned()));
            assert_eq!(lhs.length_bound, Some(7));
            if let Inner::Regex(ref regex) = lhs.inner {
                assert_eq!(regex.as_str(), "^(?-u:fo*!)$");
            } else {
                panic!("Unexpected Inner: {:?}", lhs.inner);
            }
            let rhs = calc_regex.get_node(rhs);
            assert_eq!(rhs.name, Some("bar".to_owned()));
            assert_eq!(rhs.length_bound, Some(8));
            if let Inner::Regex(ref regex) = rhs.inner {
                assert_eq!(regex.as_str(), "^(?-u:ba*r!)$");
            } else {
                panic!("Unexpected Inner: {:?}", rhs.inner);
            }
        } else {
            panic!("Unexpected Inner: {:?}", lhs.inner);
        }
        let rhs = calc_regex.get_node(rhs);
        assert_eq!(rhs.name, Some("bar".to_owned()));
        assert_eq!(rhs.length_bound, Some(8));
        if let Inner::Regex(ref regex) = rhs.inner {
            assert_eq!(regex.as_str(), "^(?-u:ba*r!)$");
        } else {
            panic!("Unexpected Inner: {:?}", rhs.inner);
        }
    } else {
        panic!("Unexpected Inner: {:?}", root.inner);
    }

}

///////////////////////////////////////////////////////////////////////////////
//      Clone
///////////////////////////////////////////////////////////////////////////////

#[test]
fn clone() {
    let calc_regex = generate! {
        foo := "foo";
    };
    let clone = calc_regex.clone();
    let root = clone.get_root();
    assert_eq!(root.name, Some("foo".to_owned()));
    assert_eq!(root.length_bound, Some(3));
    if let Inner::Regex(ref regex) = root.inner {
        assert_eq!(regex.as_str(), "^(?-u:foo)$");
    } else {
        panic!("Unexpected Inner: {:?}", root.inner);
    }
}

#[test]
fn clone_and_set_root() {
    #![allow(unused_variables)]
    let mut calc_regex = generate! {
        foo := "foo";
        bar := "bar!";
    };
    let clone = calc_regex.clone();
    calc_regex.set_root_by_name("foo").unwrap();
    let root = clone.get_root();
    assert_eq!(root.name, Some("bar".to_owned()));
    assert_eq!(root.length_bound, Some(4));
    if let Inner::Regex(ref regex) = root.inner {
        assert_eq!(regex.as_str(), "^(?-u:bar!)$");
    } else {
        panic!("Unexpected Inner: {:?}", root.inner);
    }
}

#[test]
fn clone_and_set_length_bound() {
    let mut calc_regex = generate! {
        foo = "f", "o"*, "!";
    };
    let clone = calc_regex.clone();
    calc_regex.set_root_length_bound(9);
    let root = clone.get_root();
    assert_eq!(root.name, Some("foo".to_owned()));
    assert_eq!(root.length_bound, None);
    if let Inner::Regex(ref regex) = root.inner {
        assert_eq!(regex.as_str(), "^(?-u:fo*!)$");
    } else {
        panic!("Unexpected Inner: {:?}", root.inner);
    }
}
