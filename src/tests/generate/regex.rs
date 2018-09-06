//! Generates plain regexes wrapped in `CalcRegex`es and checks their structure
//! explicitly.

use calc_regex::Inner;

///////////////////////////////////////////////////////////////////////////////
//      Identifier, String, Parentheses
///////////////////////////////////////////////////////////////////////////////

#[test]
fn string() {
    let calc_regex = generate! {
        foo = "foo";
    };
    let root = calc_regex.get_root();
    assert_eq!(root.name, Some("foo".to_owned()));
    assert_eq!(root.length_bound, Some(3));
    if let Inner::Regex(ref regex) = root.inner {
        assert_eq!(regex.as_str(), "^(?-u:foo)$");
    } else {
        panic!("Unexpected Inner: {:?}", root.inner);
    }
}

#[test]
fn unused_string() {
    #![allow(unused_variables)]
    let calc_regex = generate! {
        foo = "foo!";
        bar = "bar!";
    };
    let root = calc_regex.get_root();
    assert_eq!(root.name, Some("bar".to_owned()));
    assert_eq!(root.length_bound, Some(4));
    if let Inner::Regex(ref regex) = root.inner {
        assert_eq!(regex.as_str(), "^(?-u:bar!)$");
    } else {
        panic!("Unexpected Inner: {:?}", root.inner);
    }
}

#[test]
fn escape() {
    let calc_regex = generate! {
        foo = "*)";
    };
    let root = calc_regex.get_root();
    assert_eq!(root.name, Some("foo".to_owned()));
    assert_eq!(root.length_bound, Some(2));
    if let Inner::Regex(ref regex) = root.inner {
        assert_eq!(regex.as_str(), r"^(?-u:\*\))$");
    } else {
        panic!("Unexpected Inner: {:?}", root.inner);
    }
}

#[test]
#[should_panic]
fn unicode() {
    let _ = generate! {
        foo = "こんにちは";
    };
}

#[test]
fn identifier() {
    let calc_regex = generate! {
        foo = "foo";
        bar = foo;
    };
    let root = calc_regex.get_root();
    assert_eq!(root.name, Some("bar".to_owned()));
    assert_eq!(root.length_bound, Some(3));
    if let Inner::Regex(ref regex) = root.inner {
        assert_eq!(regex.as_str(), "^(?-u:foo)$");
    } else {
        panic!("Unexpected Inner: {:?}", root.inner);
    }
}

#[test]
#[should_panic]
fn calc_regex_identifier() {
    let _ = generate! {
        foo := "foo";
        bar = foo;
    };
}

#[test]
fn identifier_two_times() {
    let calc_regex = generate! {
        foo = "foo";
        bar = foo;
        baz = bar;
    };
    let root = calc_regex.get_root();
    assert_eq!(root.name, Some("baz".to_owned()));
    assert_eq!(root.length_bound, Some(3));
    if let Inner::Regex(ref regex) = root.inner {
        assert_eq!(regex.as_str(), "^(?-u:foo)$");
    } else {
        panic!("Unexpected Inner: {:?}", root.inner);
    }
}

#[test]
fn parentheses() {
    let calc_regex = generate! {
        foo = ("foo!");
    };
    let root = calc_regex.get_root();
    assert_eq!(root.name, Some("foo".to_owned()));
    assert_eq!(root.length_bound, Some(4));
    if let Inner::Regex(ref regex) = root.inner {
        assert_eq!(regex.as_str(), "^(?-u:(foo!))$");
    } else {
        panic!("Unexpected Inner: {:?}", root.inner);
    }
}

#[test]
fn parantheses_variable() {
    let calc_regex = generate! {
        foo = "foo!";
        bar = (foo);
    };
    let root = calc_regex.get_root();
    assert_eq!(root.name, Some("bar".to_owned()));
    assert_eq!(root.length_bound, Some(4));
    if let Inner::Regex(ref regex) = root.inner {
        assert_eq!(regex.as_str(), "^(?-u:(foo!))$");
    } else {
        panic!("Unexpected Inner: {:?}", root.inner);
    }
}

///////////////////////////////////////////////////////////////////////////////
//      Range, Hex
///////////////////////////////////////////////////////////////////////////////

#[test]
fn char_range() {
    let calc_regex = generate! {
        foo = "a" - "z";
    };
    let root = calc_regex.get_root();
    assert_eq!(root.name, Some("foo".to_owned()));
    assert_eq!(root.length_bound, Some(1));
    if let Inner::Regex(ref regex) = root.inner {
        assert_eq!(regex.as_str(), "^(?-u:[a-z])$");
    } else {
        panic!("Unexpected Inner: {:?}", root.inner);
    }
}

#[test]
#[should_panic]
fn char_range_lower_grater() {
    let _ = generate! {
        foo = "d" - "a";
    };
}

#[test]
#[should_panic]
fn range_multiple_chars() {
    let _ = generate! {
        foo = "abc" - "z";
    };
}

#[test]
fn hex_value() {
    let calc_regex = generate! {
        foo = %42;
    };
    let root = calc_regex.get_root();
    assert_eq!(root.name, Some("foo".to_owned()));
    assert_eq!(root.length_bound, Some(1));
    if let Inner::Regex(ref regex) = root.inner {
        assert_eq!(regex.as_str(), r"^(?-u:\x42)$");
    } else {
        panic!("Unexpected Inner: {:?}", root.inner);
    }
}

#[test]
#[should_panic]
fn hex_value_invalid() {
    let _ = generate! {
        foo = %GG;
    };
}

#[test]
fn hex_value_formatting() {
    let calc_regex = generate! {
        foo = %f;
    };
    let root = calc_regex.get_root();
    assert_eq!(root.name, Some("foo".to_owned()));
    assert_eq!(root.length_bound, Some(1));
    if let Inner::Regex(ref regex) = root.inner {
        assert_eq!(regex.as_str(), r"^(?-u:\x0F)$");
    } else {
        panic!("Unexpected Inner: {:?}", root.inner);
    }
}

#[test]
fn hex_range() {
    let calc_regex = generate! {
        foo = %0 - %FF;
    };
    let root = calc_regex.get_root();
    assert_eq!(root.name, Some("foo".to_owned()));
    assert_eq!(root.length_bound, Some(1));
    if let Inner::Regex(ref regex) = root.inner {
        assert_eq!(regex.as_str(), r"^(?-u:[\x00-\xFF])$");
    } else {
        panic!("Unexpected Inner: {:?}", root.inner);
    }
}

#[test]
#[should_panic]
fn hex_range_non_hex_value() {
    let _ = generate! {
        foo = %0 - %GG;
    };
}

#[test]
#[should_panic]
fn hex_range_lower_grater() {
    let _ = generate! {
        foo = %FF - %F;
    };
}

///////////////////////////////////////////////////////////////////////////////
//      Choice
///////////////////////////////////////////////////////////////////////////////

#[test]
fn choice_identifier_lhs_longer() {
    let calc_regex = generate! {
        foo = "fooo!";
        bar = "bar!";
        baz = foo | bar;
    };
    let root = calc_regex.get_root();
    assert_eq!(root.name, Some("baz".to_owned()));
    assert_eq!(root.length_bound, Some(5));
    if let Inner::Regex(ref regex) = root.inner {
        assert_eq!(regex.as_str(), "^(?-u:fooo!|bar!)$");
    } else {
        panic!("Unexpected Inner: {:?}", root.inner);
    }
}

#[test]
fn choice_identifier_rhs_longer() {
    let calc_regex = generate! {
        foo = "foo!";
        bar = "baaar!";
        baz = foo | bar;
    };
    let root = calc_regex.get_root();
    assert_eq!(root.name, Some("baz".to_owned()));
    assert_eq!(root.length_bound, Some(6));
    if let Inner::Regex(ref regex) = root.inner {
        assert_eq!(regex.as_str(), "^(?-u:foo!|baaar!)$");
    } else {
        panic!("Unexpected Inner: {:?}", root.inner);
    }
}

#[test]
fn choice_string() {
    let calc_regex = generate! {
        foo = "foo" | "bar";
    };
    let root = calc_regex.get_root();
    assert_eq!(root.name, Some("foo".to_owned()));
    assert_eq!(root.length_bound, Some(3));
    if let Inner::Regex(ref regex) = root.inner {
        assert_eq!(regex.as_str(), "^(?-u:foo|bar)$");
    } else {
        panic!("Unexpected Inner: {:?}", root.inner);
    }
}

#[test]
fn choice_char_range() {
    let calc_regex = generate! {
        foo = "a" - "z" | "A" - "Z";
    };
    let root = calc_regex.get_root();
    assert_eq!(root.name, Some("foo".to_owned()));
    assert_eq!(root.length_bound, Some(1));
    if let Inner::Regex(ref regex) = root.inner {
        assert_eq!(regex.as_str(), "^(?-u:[a-z]|[A-Z])$");
    } else {
        panic!("Unexpected Inner: {:?}", root.inner);
    }
}

#[test]
fn choice_parentheses() {
    let calc_regex = generate! {
        foo = "foo!";
        bar = "bar!";
        baz = (foo | bar);
    };
    let root = calc_regex.get_root();
    assert_eq!(root.name, Some("baz".to_owned()));
    assert_eq!(root.length_bound, Some(4));
    if let Inner::Regex(ref regex) = root.inner {
        assert_eq!(regex.as_str(), "^(?-u:(foo!|bar!))$");
    } else {
        panic!("Unexpected Inner: {:?}", root.inner);
    }
}

#[test]
fn choice_three_times() {
    let calc_regex = generate! {
        foo = "foo!";
        bar = "bar!";
        baz = foo | bar | bar;
    };
    let root = calc_regex.get_root();
    assert_eq!(root.name, Some("baz".to_owned()));
    assert_eq!(root.length_bound, Some(4));
    if let Inner::Regex(ref regex) = root.inner {
        assert_eq!(regex.as_str(), "^(?-u:foo!|bar!|bar!)$");
    } else {
        panic!("Unexpected Inner: {:?}", root.inner);
    }
}

#[test]
fn choice_concat() {
    let calc_regex = generate! {
        foo = "foo!";
        bar = "bar!";
        baz = foo | foo, bar | bar;
    };
    let root = calc_regex.get_root();
    assert_eq!(root.name, Some("baz".to_owned()));
    assert_eq!(root.length_bound, Some(8));
    if let Inner::Regex(ref regex) = root.inner {
        assert_eq!(regex.as_str(), "^(?-u:foo!|foo!bar!|bar!)$");
    } else {
        panic!("Unexpected Inner: {:?}", root.inner);
    }
}

#[test]
fn concatenate_choice_identifier() {
    let calc_regex = generate! {
        foo = "foo!" | "baar!";
        bar = foo, "baz!";
    };
    let root = calc_regex.get_root();
    assert_eq!(root.name, Some("bar".to_owned()));
    assert_eq!(root.length_bound, Some(9));
    if let Inner::Regex(ref regex) = root.inner {
        assert_eq!(regex.as_str(), "^(?-u:(foo!|baar!)baz!)$");
    } else {
        panic!("Unexpected Inner: {:?}", root.inner);
    }
}

///////////////////////////////////////////////////////////////////////////////
//      Kleene Star, Kleene Plus
///////////////////////////////////////////////////////////////////////////////

#[test]
fn kleene_star_identifier() {
    let calc_regex = generate! {
        foo = "foo!";
        bar = foo*;
    };
    let root = calc_regex.get_root();
    assert_eq!(root.name, Some("bar".to_owned()));
    assert_eq!(root.length_bound, None);
    if let Inner::Regex(ref regex) = root.inner {
        assert_eq!(regex.as_str(), "^(?-u:(foo!)*)$");
    } else {
        panic!("Unexpected Inner: {:?}", root.inner);
    }
}

#[test]
fn kleene_star_string() {
    let calc_regex = generate! {
        foo = "foo!"*;
    };
    let root = calc_regex.get_root();
    assert_eq!(root.name, Some("foo".to_owned()));
    assert_eq!(root.length_bound, None);
    if let Inner::Regex(ref regex) = root.inner {
        assert_eq!(regex.as_str(), "^(?-u:(foo!)*)$");
    } else {
        panic!("Unexpected Inner: {:?}", root.inner);
    }
}

#[test]
fn kleene_star_atomic() {
    let calc_regex = generate! {
        foo = "f"*;
    };
    let root = calc_regex.get_root();
    assert_eq!(root.name, Some("foo".to_owned()));
    assert_eq!(root.length_bound, None);
    if let Inner::Regex(ref regex) = root.inner {
        assert_eq!(regex.as_str(), "^(?-u:f*)$");
    } else {
        panic!("Unexpected Inner: {:?}", root.inner);
    }
}

#[test]
fn kleene_plus_identifier() {
    let calc_regex = generate! {
        foo = "foo!";
        bar = foo+;
    };
    let root = calc_regex.get_root();
    assert_eq!(root.name, Some("bar".to_owned()));
    assert_eq!(root.length_bound, None);
    if let Inner::Regex(ref regex) = root.inner {
        assert_eq!(regex.as_str(), "^(?-u:(foo!)+)$");
    } else {
        panic!("Unexpected Inner: {:?}", root.inner);
    }
}

#[test]
fn kleene_plus_string() {
    let calc_regex = generate! {
        foo = "foo!"+;
    };
    let root = calc_regex.get_root();
    assert_eq!(root.name, Some("foo".to_owned()));
    assert_eq!(root.length_bound, None);
    if let Inner::Regex(ref regex) = root.inner {
        assert_eq!(regex.as_str(), "^(?-u:(foo!)+)$");
    } else {
        panic!("Unexpected Inner: {:?}", root.inner);
    }
}

#[test]
fn kleene_plus_atomic() {
    let calc_regex = generate! {
        foo = "f"+;
    };
    let root = calc_regex.get_root();
    assert_eq!(root.name, Some("foo".to_owned()));
    assert_eq!(root.length_bound, None);
    if let Inner::Regex(ref regex) = root.inner {
        assert_eq!(regex.as_str(), "^(?-u:f+)$");
    } else {
        panic!("Unexpected Inner: {:?}", root.inner);
    }
}

///////////////////////////////////////////////////////////////////////////////
//      Repeat
///////////////////////////////////////////////////////////////////////////////

#[test]
fn repeat_identifier() {
    let calc_regex = generate! {
        foo   = "foo";
        regex = foo^3;
    };
    let root = calc_regex.get_root();
    assert_eq!(root.name, Some("regex".to_owned()));
    assert_eq!(root.length_bound, Some(9));
    if let Inner::Regex(ref regex) = root.inner {
        assert_eq!(regex.as_str(), "^(?-u:(foo){3})$");
    } else {
        panic!("Unexpected Inner: {:?}", root.inner);
    }
}

#[test]
fn repeat_identifier_atomic() {
    let calc_regex = generate! {
        byte  = %0 - %FF;
        regex = byte^3;
    };
    let root = calc_regex.get_root();
    assert_eq!(root.name, Some("regex".to_owned()));
    assert_eq!(root.length_bound, Some(3));
    if let Inner::Regex(ref regex) = root.inner {
        assert_eq!(regex.as_str(), r"^(?-u:[\x00-\xFF]{3})$");
    } else {
        panic!("Unexpected Inner: {:?}", root.inner);
    }
}

#[test]
fn repeat_string() {
    let calc_regex = generate! {
        regex = "foo"^3;
    };
    let root = calc_regex.get_root();
    assert_eq!(root.name, Some("regex".to_owned()));
    assert_eq!(root.length_bound, Some(9));
    if let Inner::Regex(ref regex) = root.inner {
        assert_eq!(regex.as_str(), "^(?-u:(foo){3})$");
    } else {
        panic!("Unexpected Inner: {:?}", root.inner);
    }
}

#[test]
fn concatenate_repeat_lhs() {
    let calc_regex = generate! {
        byte  = %0 - %FF;
        regex = byte^3, "foo";
    };
    let root = calc_regex.get_root();
    assert_eq!(root.name, Some("regex".to_owned()));
    assert_eq!(root.length_bound, Some(6));
    if let Inner::Regex(ref regex) = root.inner {
        assert_eq!(regex.as_str(), r"^(?-u:[\x00-\xFF]{3}foo)$");
    } else {
        panic!("Unexpected Inner: {:?}", root.inner);
    }
}

#[test]
fn concatenate_repeat_rhs() {
    let calc_regex = generate! {
        byte  = %0 - %FF;
        regex = "foo", byte^3;
    };
    let root = calc_regex.get_root();
    assert_eq!(root.name, Some("regex".to_owned()));
    assert_eq!(root.length_bound, Some(6));
    if let Inner::Regex(ref regex) = root.inner {
        assert_eq!(regex.as_str(), r"^(?-u:foo[\x00-\xFF]{3})$");
    } else {
        panic!("Unexpected Inner: {:?}", root.inner);
    }
}

///////////////////////////////////////////////////////////////////////////////
//      Concatenate, Combination
///////////////////////////////////////////////////////////////////////////////

#[test]
fn concatenate() {
    let calc_regex = generate! {
        foo = "foo!";
        bar = "bar!";
        baz = foo, bar, foo;
    };
    let root = calc_regex.get_root();
    assert_eq!(root.name, Some("baz".to_owned()));
    assert_eq!(root.length_bound, Some(12));
    if let Inner::Regex(ref regex) = root.inner {
        assert_eq!(regex.as_str(), "^(?-u:foo!bar!foo!)$");
    } else {
        panic!("Unexpected Inner: {:?}", root.inner);
    }
}

#[test]
fn concatenate_star() {
    let calc_regex = generate! {
        foo = "foo!";
        bar = "bar!";
        baz = foo, bar*;
    };
    let root = calc_regex.get_root();
    assert_eq!(root.name, Some("baz".to_owned()));
    assert_eq!(root.length_bound, None);
    if let Inner::Regex(ref regex) = root.inner {
        assert_eq!(regex.as_str(), "^(?-u:foo!(bar!)*)$");
    } else {
        panic!("Unexpected Inner: {:?}", root.inner);
    }
}

#[test]
fn choice_combination_0() {
    let calc_regex = generate! {
        foo = "foo!";
        bar = "bar!";
        baz = "bla" | (foo, bar);
    };
    let root = calc_regex.get_root();
    assert_eq!(root.name, Some("baz".to_owned()));
    assert_eq!(root.length_bound, Some(8));
    if let Inner::Regex(ref regex) = root.inner {
        assert_eq!(regex.as_str(), "^(?-u:bla|(foo!bar!))$");
    } else {
        panic!("Unexpected Inner: {:?}", root.inner);
    }
}

#[test]
fn choice_combination_1() {
    let calc_regex = generate! {
        foo = "foo!";
        bar = "bar!";
        baz = "bla" | (foo, bar*);
    };
    let root = calc_regex.get_root();
    assert_eq!(root.name, Some("baz".to_owned()));
    assert_eq!(root.length_bound, None);
    if let Inner::Regex(ref regex) = root.inner {
        assert_eq!(regex.as_str(), "^(?-u:bla|(foo!(bar!)*))$");
    } else {
        panic!("Unexpected Inner: {:?}", root.inner);
    }
}

#[test]
fn choice_combination_2() {
    let calc_regex = generate! {
        foo = "foo!";
        bar = "bar!";
        baz = "bla" | (foo*, bar);
    };
    let root = calc_regex.get_root();
    assert_eq!(root.name, Some("baz".to_owned()));
    assert_eq!(root.length_bound, None);
    if let Inner::Regex(ref regex) = root.inner {
        assert_eq!(regex.as_str(), "^(?-u:bla|((foo!)*bar!))$");
    } else {
        panic!("Unexpected Inner: {:?}", root.inner);
    }
}

#[test]
fn choice_combination_3() {
    let calc_regex = generate! {
        foo = "foo!";
        bar = "bar!";
        baz = "bla" | (foo, bar)*;
    };
    let root = calc_regex.get_root();
    assert_eq!(root.name, Some("baz".to_owned()));
    assert_eq!(root.length_bound, None);
    if let Inner::Regex(ref regex) = root.inner {
        assert_eq!(regex.as_str(), "^(?-u:bla|(foo!bar!)*)$");
    } else {
        panic!("Unexpected Inner: {:?}", root.inner);
    }
}

#[test]
fn choice_combination_4() {
    let calc_regex = generate! {
        foo = "foo!";
        bar = "bar!";
        baz = "bla"* | (foo, bar);
    };
    let root = calc_regex.get_root();
    assert_eq!(root.name, Some("baz".to_owned()));
    assert_eq!(root.length_bound, None);
    if let Inner::Regex(ref regex) = root.inner {
        assert_eq!(regex.as_str(), "^(?-u:(bla)*|(foo!bar!))$");
    } else {
        panic!("Unexpected Inner: {:?}", root.inner);
    }
}

#[test]
fn choice_combination_5() {
    let calc_regex = generate! {
        foo = "foo!";
        bar = "bar!";
        baz = "bla"* | (foo, bar)*;
    };
    let root = calc_regex.get_root();
    assert_eq!(root.name, Some("baz".to_owned()));
    assert_eq!(root.length_bound, None);
    if let Inner::Regex(ref regex) = root.inner {
        assert_eq!(regex.as_str(), "^(?-u:(bla)*|(foo!bar!)*)$");
    } else {
        panic!("Unexpected Inner: {:?}", root.inner);
    }
}

#[test]
fn choice_combination_6() {
    let calc_regex = generate! {
        foo = "foo!";
        bar = "bar!";
        baz = (foo, bar) | "bla" ;
    };
    let root = calc_regex.get_root();
    assert_eq!(root.name, Some("baz".to_owned()));
    assert_eq!(root.length_bound, Some(8));
    if let Inner::Regex(ref regex) = root.inner {
        assert_eq!(regex.as_str(), "^(?-u:(foo!bar!)|bla)$");
    } else {
        panic!("Unexpected Inner: {:?}", root.inner);
    }
}
