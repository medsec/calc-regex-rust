//! Generates `CalcRegex`es and checks their structure explicitely.

use calc_regex::Inner;

fn dummy(_r: &[u8]) -> Option<usize> {
    Some(42)
}

#[allow(dead_code)]
fn dummy_2(_r: &[u8]) -> Option<usize> {
    Some(23)
}

///////////////////////////////////////////////////////////////////////////////
//      Identifier, Regex, Concatenate, Parentheses
///////////////////////////////////////////////////////////////////////////////

#[test]
fn simple_regex() {
    let calc_regex = generate! {
        foo := "foo";
    };
    let root = calc_regex.get_root();
    assert_eq!(root.name, Some("foo".to_owned()));
    if let Inner::Regex(ref regex) = root.inner {
        assert_eq!(regex.as_str(), "^(?-u:foo)$");
    } else {
        panic!("Unexpected Inner: {:?}", root.inner);
    }
}

#[test]
#[should_panic]
fn used_identifier() {
    let _ = generate! {
        foo := "foo";
        foo := foo;
    };
}

#[test]
fn identifier() {
    let calc_regex = generate! {
        foo  = "foo";
        bar := foo;
    };
    let root = calc_regex.get_root();
    assert_eq!(root.name, Some("bar".to_owned()));
    assert_eq!(root.length_bound, None);
    if let Inner::CalcRegex(node_index) = root.inner {
        let node = calc_regex.get_node(node_index);
        assert_eq!(node.name, Some("foo".to_owned()));
        assert_eq!(node.length_bound, Some(3));
        if let Inner::Regex(ref regex) = node.inner {
            assert_eq!(regex.as_str(), "^(?-u:foo)$");
        } else {
            panic!("Unexpected Inner: {:?}", node.inner);
        }
    } else {
        panic!("Unexpected Inner: {:?}", root.inner);
    }
}

#[test]
fn identifier_two_times() {
    let calc_regex = generate! {
        foo  = "foo";
        bar := foo;
        baz := bar;
    };
    let root = calc_regex.get_root();
    assert_eq!(root.name, Some("baz".to_owned()));
    assert_eq!(root.length_bound, None);
    if let Inner::CalcRegex(node_index) = root.inner {
        let node = calc_regex.get_node(node_index);
        assert_eq!(node.name, Some("bar".to_owned()));
        assert_eq!(node.length_bound, None);
        if let Inner::CalcRegex(node_index) = node.inner {
            let node = calc_regex.get_node(node_index);
            assert_eq!(node.name, Some("foo".to_owned()));
            assert_eq!(node.length_bound, Some(3));
            if let Inner::Regex(ref regex) = node.inner {
                assert_eq!(regex.as_str(), "^(?-u:foo)$");
            } else {
                panic!("Unexpected Inner: {:?}", node.inner);
            }
        } else {
            panic!("Unexpected Inner: {:?}", node.inner);
        }
    } else {
        panic!("Unexpected Inner: {:?}", root.inner);
    }
}

#[test]
fn parentheses() {
    let calc_regex = generate! {
        foo := "foo";
        bar := (foo);
    };
    let root = calc_regex.get_root();
    assert_eq!(root.name, Some("bar".to_owned()));
    assert_eq!(root.length_bound, None);
    if let Inner::CalcRegex(node_index) = root.inner {
        let node = calc_regex.get_node(node_index);
        assert_eq!(node.name, Some("foo".to_owned()));
        assert_eq!(node.length_bound, Some(3));
        if let Inner::Regex(ref regex) = node.inner {
            assert_eq!(regex.as_str(), "^(?-u:foo)$");
        } else {
            panic!("Unexpected Inner: {:?}", node.inner);
        }
    } else {
        panic!("Unexpected Inner: {:?}", root.inner);
    }
}

#[test]
fn concatenate_regex() {
    let calc_regex = generate! {
        foo         = "foo";
        bar         = "bar";
        calc_regex := foo, bar;
    };
    let root = calc_regex.get_root();
    assert_eq!(root.name, Some("calc_regex".to_owned()));
    assert_eq!(root.length_bound, None);
    if let Inner::Concat(lhs, rhs) = root.inner {
        let lhs = calc_regex.get_node(lhs);
        assert_eq!(lhs.name, Some("foo".to_owned()));
        assert_eq!(lhs.length_bound, Some(3));
        if let Inner::Regex(ref re) = lhs.inner {
            assert_eq!(re.as_str(), "^(?-u:foo)$");
        } else {
            panic!("Unexpected Inner: {:?}", lhs.inner);
        }
        let rhs = calc_regex.get_node(rhs);
        assert_eq!(rhs.name, Some("bar".to_owned()));
        assert_eq!(rhs.length_bound, Some(3));
        if let Inner::Regex(ref re) = rhs.inner {
            assert_eq!(re.as_str(), "^(?-u:bar)$");
        } else {
            panic!("Unexpected Inner: {:?}", rhs.inner);
        }
    } else {
        panic!("Unexpected Inner: {:?}", root.inner);
    }
}

#[test]
fn concatenate_calc_regex() {
    let calc_regex = generate! {
        foo        := "foo";
        bar        := "bar";
        calc_regex := foo, bar;
    };
    let root = calc_regex.get_root();
    assert_eq!(root.name, Some("calc_regex".to_owned()));
    assert_eq!(root.length_bound, None);
    if let Inner::Concat(lhs, rhs) = root.inner {
        let lhs = calc_regex.get_node(lhs);
        assert_eq!(lhs.name, Some("foo".to_owned()));
        assert_eq!(lhs.length_bound, Some(3));
        if let Inner::Regex(ref re) = lhs.inner {
            assert_eq!(re.as_str(), "^(?-u:foo)$");
        } else {
            panic!("Unexpected Inner: {:?}", lhs.inner);
        }
        let rhs = calc_regex.get_node(rhs);
        assert_eq!(rhs.name, Some("bar".to_owned()));
        assert_eq!(rhs.length_bound, Some(3));
        if let Inner::Regex(ref re) = rhs.inner {
            assert_eq!(re.as_str(), "^(?-u:bar)$");
        } else {
            panic!("Unexpected Inner: {:?}", rhs.inner);
        }
    } else {
        panic!("Unexpected Inner: {:?}", root.inner);
    }
}

#[test]
fn concatenate_regex_same() {
    let calc_regex = generate! {
        foo         = "foo";
        calc_regex := foo, foo;
    };
    let root = calc_regex.get_root();
    assert_eq!(root.name, Some("calc_regex".to_owned()));
    assert_eq!(root.length_bound, None);
    if let Inner::Concat(lhs, rhs) = root.inner {
        assert_eq!(lhs, rhs);
        let lhs = calc_regex.get_node(lhs);
        assert_eq!(lhs.name, Some("foo".to_owned()));
        assert_eq!(lhs.length_bound, Some(3));
        if let Inner::Regex(ref re) = lhs.inner {
            assert_eq!(re.as_str(), "^(?-u:foo)$");
        } else {
            panic!("Unexpected Inner: {:?}", lhs.inner);
        }
    } else {
        panic!("Unexpected Inner: {:?}", root.inner);
    }
}

#[test]
fn concatenate_calc_regex_same() {
    let calc_regex = generate! {
        foo        := "foo";
        calc_regex := foo, foo;
    };
    let root = calc_regex.get_root();
    assert_eq!(root.name, Some("calc_regex".to_owned()));
    assert_eq!(root.length_bound, None);
    if let Inner::Concat(lhs, rhs) = root.inner {
        assert_eq!(lhs, rhs);
        let lhs = calc_regex.get_node(lhs);
        assert_eq!(lhs.name, Some("foo".to_owned()));
        assert_eq!(lhs.length_bound, Some(3));
        if let Inner::Regex(ref re) = lhs.inner {
            assert_eq!(re.as_str(), "^(?-u:foo)$");
        } else {
            panic!("Unexpected Inner: {:?}", lhs.inner);
        }
    } else {
        panic!("Unexpected Inner: {:?}", root.inner);
    }
}

#[test]
fn concatenate_three_different() {
    let calc_regex = generate! {
        foo        := "foo";
        bar        := "bar";
        baz        := "baz";
        calc_regex := foo, bar, baz;
    };
    let root = calc_regex.get_root();
    assert_eq!(root.name, Some("calc_regex".to_owned()));
    assert_eq!(root.length_bound, None);
    if let Inner::Concat(lhs, rhs) = root.inner {
        let lhs = calc_regex.get_node(lhs);
        assert_eq!(lhs.name, Some("foo".to_owned()));
        assert_eq!(lhs.length_bound, Some(3));
        if let Inner::Regex(ref re) = lhs.inner {
            assert_eq!(re.as_str(), "^(?-u:foo)$");
        } else {
            panic!("Unexpected Inner: {:?}", lhs.inner);
        }
        let rhs = calc_regex.get_node(rhs);
        assert_eq!(rhs.name, None);
        assert_eq!(rhs.length_bound, None);
        if let Inner::Concat(lhs, rhs) = rhs.inner {
            let lhs = calc_regex.get_node(lhs);
            assert_eq!(lhs.name, Some("bar".to_owned()));
            assert_eq!(lhs.length_bound, Some(3));
            if let Inner::Regex(ref re) = lhs.inner {
                assert_eq!(re.as_str(), "^(?-u:bar)$");
            } else {
                panic!("Unexpected Inner: {:?}", lhs.inner);
            }
            let rhs = calc_regex.get_node(rhs);
            assert_eq!(rhs.name, Some("baz".to_owned()));
            assert_eq!(rhs.length_bound, Some(3));
            if let Inner::Regex(ref re) = rhs.inner {
                assert_eq!(re.as_str(), "^(?-u:baz)$");
            } else {
                panic!("Unexpected Inner: {:?}", rhs.inner);
            }
        } else {
            panic!("Unexpected Inner: {:?}", rhs.inner);
        }
    } else {
        panic!("Unexpected Inner: {:?}", root.inner);
    }
}

#[test]
fn concatenate_regex_anonymous() {
    let calc_regex = generate! {
        calc_regex := "foo", "bar", "baz";
    };
    let root = calc_regex.get_root();
    assert_eq!(root.name, Some("calc_regex".to_owned()));
    assert_eq!(root.length_bound, None);
    if let Inner::Concat(lhs, rhs) = root.inner {
        let lhs = calc_regex.get_node(lhs);
        assert_eq!(lhs.name, None);
        assert_eq!(lhs.length_bound, Some(3));
        if let Inner::Regex(ref re) = lhs.inner {
            assert_eq!(re.as_str(), "^(?-u:foo)$");
        } else {
            panic!("Unexpected Inner: {:?}", lhs.inner);
        }
        let rhs = calc_regex.get_node(rhs);
        assert_eq!(rhs.name, None);
        assert_eq!(rhs.length_bound, None);
        if let Inner::Concat(lhs, rhs) = rhs.inner {
            let lhs = calc_regex.get_node(lhs);
            assert_eq!(lhs.name, None);
            assert_eq!(lhs.length_bound, Some(3));
            if let Inner::Regex(ref re) = lhs.inner {
                assert_eq!(re.as_str(), "^(?-u:bar)$");
            } else {
                panic!("Unexpected Inner: {:?}", lhs.inner);
            }
            let rhs = calc_regex.get_node(rhs);
            assert_eq!(rhs.name, None);
            assert_eq!(rhs.length_bound, Some(3));
            if let Inner::Regex(ref re) = rhs.inner {
                assert_eq!(re.as_str(), "^(?-u:baz)$");
            } else {
                panic!("Unexpected Inner: {:?}", rhs.inner);
            }
        } else {
            panic!("Unexpected Inner: {:?}", rhs.inner);
        }
    } else {
        panic!("Unexpected Inner: {:?}", root.inner);
    }
}

#[test]
fn concatenate_regex_mixed_anonymous() {
    let calc_regex = generate! {
        foo         = "foo";
        baz         = "baz";
        calc_regex := foo, "bar", baz;
    };
    let root = calc_regex.get_root();
    assert_eq!(root.name, Some("calc_regex".to_owned()));
    assert_eq!(root.length_bound, None);
    if let Inner::Concat(lhs, rhs) = root.inner {
        let lhs = calc_regex.get_node(lhs);
        assert_eq!(lhs.name, Some("foo".to_owned()));
        assert_eq!(lhs.length_bound, Some(3));
        if let Inner::Regex(ref re) = lhs.inner {
            assert_eq!(re.as_str(), "^(?-u:foo)$");
        } else {
            panic!("Unexpected Inner: {:?}", lhs.inner);
        }
        let rhs = calc_regex.get_node(rhs);
        assert_eq!(rhs.name, None);
        assert_eq!(rhs.length_bound, None);
        if let Inner::Concat(lhs, rhs) = rhs.inner {
            let lhs = calc_regex.get_node(lhs);
            assert_eq!(lhs.name, None);
            assert_eq!(lhs.length_bound, Some(3));
            if let Inner::Regex(ref re) = lhs.inner {
                assert_eq!(re.as_str(), "^(?-u:bar)$");
            } else {
                panic!("Unexpected Inner: {:?}", lhs.inner);
            }
            let rhs = calc_regex.get_node(rhs);
            assert_eq!(rhs.name, Some("baz".to_owned()));
            assert_eq!(rhs.length_bound, Some(3));
            if let Inner::Regex(ref re) = rhs.inner {
                assert_eq!(re.as_str(), "^(?-u:baz)$");
            } else {
                panic!("Unexpected Inner: {:?}", rhs.inner);
            }
        } else {
            panic!("Unexpected Inner: {:?}", rhs.inner);
        }
    } else {
        panic!("Unexpected Inner: {:?}", root.inner);
    }
}

#[test]
fn concatenate_parantheses() {
    let calc_regex = generate! {
        foo         = "foo";
        bar         = "bar";
        calc_regex := (foo), (bar);
    };
    let root = calc_regex.get_root();
    assert_eq!(root.name, Some("calc_regex".to_owned()));
    assert_eq!(root.length_bound, None);
    if let Inner::Concat(lhs, rhs) = root.inner {
        let lhs = calc_regex.get_node(lhs);
        assert_eq!(lhs.name, Some("foo".to_owned()));
        assert_eq!(lhs.length_bound, Some(3));
        if let Inner::Regex(ref re) = lhs.inner {
            assert_eq!(re.as_str(), "^(?-u:foo)$");
        } else {
            panic!("Unexpected Inner: {:?}", lhs.inner);
        }
        let rhs = calc_regex.get_node(rhs);
        assert_eq!(rhs.name, Some("bar".to_owned()));
        assert_eq!(rhs.length_bound, Some(3));
        if let Inner::Regex(ref re) = rhs.inner {
            assert_eq!(re.as_str(), "^(?-u:bar)$");
        } else {
            panic!("Unexpected Inner: {:?}", rhs.inner);
        }
    } else {
        panic!("Unexpected Inner: {:?}", root.inner);
    }
}

#[test]
fn concatenate_range_lhs() {
    let calc_regex = generate! {
        foo        := "foo";
        calc_regex := "0"-"9", foo;
    };
    let root = calc_regex.get_root();
    assert_eq!(root.name, Some("calc_regex".to_owned()));
    assert_eq!(root.length_bound, None);
    if let Inner::Concat(lhs, rhs) = root.inner {
        let lhs = calc_regex.get_node(lhs);
        assert_eq!(lhs.name, None);
        assert_eq!(lhs.length_bound, Some(1));
        if let Inner::Regex(ref re) = lhs.inner {
            assert_eq!(re.as_str(), "^(?-u:[0-9])$");
        } else {
            panic!("Unexpected Inner: {:?}", lhs.inner);
        }
        let rhs = calc_regex.get_node(rhs);
        assert_eq!(rhs.name, Some("foo".to_owned()));
        assert_eq!(rhs.length_bound, Some(3));
        if let Inner::Regex(ref re) = rhs.inner {
            assert_eq!(re.as_str(), "^(?-u:foo)$");
        } else {
            panic!("Unexpected Inner: {:?}", rhs.inner);
        }
    } else {
        panic!("Unexpected Inner: {:?}", root.inner);
    }
}

#[test]
fn concatenate_range_rhs() {
    let calc_regex = generate! {
        foo        := "foo";
        calc_regex := foo, "0"-"9";
    };
    let root = calc_regex.get_root();
    assert_eq!(root.name, Some("calc_regex".to_owned()));
    assert_eq!(root.length_bound, None);
    if let Inner::Concat(lhs, rhs) = root.inner {
        let lhs = calc_regex.get_node(lhs);
        assert_eq!(lhs.name, Some("foo".to_owned()));
        assert_eq!(lhs.length_bound, Some(3));
        if let Inner::Regex(ref re) = lhs.inner {
            assert_eq!(re.as_str(), "^(?-u:foo)$");
        } else {
            panic!("Unexpected Inner: {:?}", lhs.inner);
        }
        let rhs = calc_regex.get_node(rhs);
        assert_eq!(rhs.name, None);
        assert_eq!(rhs.length_bound, Some(1));
        if let Inner::Regex(ref re) = rhs.inner {
            assert_eq!(re.as_str(), "^(?-u:[0-9])$");
        } else {
            panic!("Unexpected Inner: {:?}", rhs.inner);
        }
    } else {
        panic!("Unexpected Inner: {:?}", root.inner);
    }
}

///////////////////////////////////////////////////////////////////////////////
//      Repeat
///////////////////////////////////////////////////////////////////////////////

#[test]
fn repeat_regex() {
    let calc_regex = generate! {
        byte        = %0 - %FF;
        calc_regex := byte^3;
    };
    let root = calc_regex.get_root();
    assert_eq!(root.name, Some("calc_regex".to_owned()));
    assert_eq!(root.length_bound, None);
    if let Inner::Repeat(node_index, n) = root.inner {
        assert_eq!(n, 3);
        let node = calc_regex.get_node(node_index);
        assert_eq!(node.name, Some("byte".to_owned()));
        assert_eq!(node.length_bound, Some(1));
        if let Inner::Regex(ref regex) = node.inner {
            assert_eq!(regex.as_str(), r"^(?-u:[\x00-\xFF])$");
        } else {
            panic!("Unexpected Inner: {:?}", node.inner);
        }
    } else {
        panic!("Unexpected Inner: {:?}", root.inner);
    }
}

#[test]
fn repeat_calc_regex() {
    let calc_regex = generate! {
        byte       := %0 - %FF;
        calc_regex := byte^3;
    };
    let root = calc_regex.get_root();
    assert_eq!(root.name, Some("calc_regex".to_owned()));
    assert_eq!(root.length_bound, None);
    if let Inner::Repeat(node_index, n) = root.inner {
        assert_eq!(n, 3);
        let node = calc_regex.get_node(node_index);
        assert_eq!(node.name, Some("byte".to_owned()));
        assert_eq!(node.length_bound, Some(1));
        if let Inner::Regex(ref regex) = node.inner {
            assert_eq!(regex.as_str(), r"^(?-u:[\x00-\xFF])$");
        } else {
            panic!("Unexpected Inner: {:?}", node.inner);
        }
    } else {
        panic!("Unexpected Inner: {:?}", root.inner);
    }
}

#[test]
fn repeat_regex_anonymous() {
    let calc_regex = generate! {
        calc_regex := "foo"^3;
    };
    let root = calc_regex.get_root();
    assert_eq!(root.name, Some("calc_regex".to_owned()));
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
        byte        = %0 - %FF;
        calc_regex := byte^3, "foo";
    };
    let root = calc_regex.get_root();
    assert_eq!(root.name, Some("calc_regex".to_owned()));
    assert_eq!(root.length_bound, None);
    if let Inner::Concat(lhs, rhs) = root.inner {
        let lhs = calc_regex.get_node(lhs);
        assert_eq!(lhs.name, None);
        assert_eq!(lhs.length_bound, None);
        if let Inner::Repeat(node_index, n) = lhs.inner {
            assert_eq!(n, 3);
            let node = calc_regex.get_node(node_index);
            assert_eq!(node.name, Some("byte".to_owned()));
            assert_eq!(node.length_bound, Some(1));
            if let Inner::Regex(ref regex) = node.inner {
                assert_eq!(regex.as_str(), r"^(?-u:[\x00-\xFF])$");
            } else {
                panic!("Unexpected Inner: {:?}", node.inner);
            }
        } else {
            panic!("Unexpected Inner: {:?}", lhs.inner);
        }
        let rhs = calc_regex.get_node(rhs);
        assert_eq!(rhs.name, None);
        assert_eq!(rhs.length_bound, Some(3));
        if let Inner::Regex(ref re) = rhs.inner {
            assert_eq!(re.as_str(), "^(?-u:foo)$");
        } else {
            panic!("Unexpected Inner: {:?}", rhs.inner);
        }
    } else {
        panic!("Unexpected Inner: {:?}", root.inner);
    }
}

#[test]
fn concatenate_repeat_rhs() {
    let calc_regex = generate! {
        byte        = %0 - %FF;
        calc_regex := "foo", byte^3;
    };
    let root = calc_regex.get_root();
    assert_eq!(root.name, Some("calc_regex".to_owned()));
    assert_eq!(root.length_bound, None);
    if let Inner::Concat(lhs, rhs) = root.inner {
        let lhs = calc_regex.get_node(lhs);
        assert_eq!(lhs.name, None);
        assert_eq!(lhs.length_bound, Some(3));
        if let Inner::Regex(ref re) = lhs.inner {
            assert_eq!(re.as_str(), "^(?-u:foo)$");
        } else {
            panic!("Unexpected Inner: {:?}", lhs.inner);
        }
        let rhs = calc_regex.get_node(rhs);
        assert_eq!(rhs.name, None);
        assert_eq!(rhs.length_bound, None);
        if let Inner::Repeat(node_index, n) = rhs.inner {
            assert_eq!(n, 3);
            let node = calc_regex.get_node(node_index);
            assert_eq!(node.name, Some("byte".to_owned()));
            assert_eq!(node.length_bound, Some(1));
            if let Inner::Regex(ref regex) = node.inner {
                assert_eq!(regex.as_str(), r"^(?-u:[\x00-\xFF])$");
            } else {
                panic!("Unexpected Inner: {:?}", node.inner);
            }
        } else {
            panic!("Unexpected Inner: {:?}", rhs.inner);
        }
    } else {
        panic!("Unexpected Inner: {:?}", root.inner);
    }
}

///////////////////////////////////////////////////////////////////////////////
//      Length Count
///////////////////////////////////////////////////////////////////////////////

#[test]
fn length_count() {
    let calc_regex = generate! {
        foo         = "f", "o"*;
        digit       = "0" - "9";
        calc_regex := digit.dummy, foo#dummy;
    };
    let root = calc_regex.get_root();
    assert_eq!(root.name, Some("calc_regex".to_owned()));
    assert_eq!(root.length_bound, None);
    if let Inner::LengthCount { r, s, t, ref f } = root.inner {
        let r = calc_regex.get_node(r);
        assert_eq!(r.name, Some("digit".to_owned()));
        assert_eq!(r.length_bound, Some(1));
        if let Inner::Regex(ref re) = r.inner {
            assert_eq!(re.as_str(), "^(?-u:[0-9])$");
        } else {
            panic!("Unexpected Inner: {:?}", r.inner);
        }
        assert!(s.is_none());
        let t = calc_regex.get_node(t);
        assert_eq!(t.name, Some("foo".to_owned()));
        assert_eq!(t.length_bound, None);
        if let Inner::Regex(ref re) = t.inner {
            assert_eq!(re.as_str(), "^(?-u:fo*)$");
        } else {
            panic!("Unexpected Inner: {:?}", t.inner);
        }
        assert_eq!(f(b""), Some(42));
    } else {
        panic!("Unexpected Inner: {:?}", root.inner);
    }
}

#[test]
fn length_count_s() {
    let calc_regex = generate! {
        foo         = "f", "o"*;
        bar         = "bar";
        digit       = "0" - "9";
        calc_regex := digit.dummy, bar, foo#dummy;
    };
    let root = calc_regex.get_root();
    assert_eq!(root.name, Some("calc_regex".to_owned()));
    assert_eq!(root.length_bound, None);
    if let Inner::LengthCount { r, s, t, ref f } = root.inner {
        let r = calc_regex.get_node(r);
        assert_eq!(r.name, Some("digit".to_owned()));
        assert_eq!(r.length_bound, Some(1));
        if let Inner::Regex(ref re) = r.inner {
            assert_eq!(re.as_str(), "^(?-u:[0-9])$");
        } else {
            panic!("Unexpected Inner: {:?}", r.inner);
        }
        assert!(s.is_some());
        let s = calc_regex.get_node(s.unwrap());
        assert_eq!(s.name, Some("bar".to_owned()));
        assert_eq!(s.length_bound, Some(3));
        if let Inner::Regex(ref re) = s.inner {
            assert_eq!(re.as_str(), "^(?-u:bar)$");
        } else {
            panic!("Unexpected Inner: {:?}", s.inner);
        }
        let t = calc_regex.get_node(t);
        assert_eq!(t.name, Some("foo".to_owned()));
        assert_eq!(t.length_bound, None);
        if let Inner::Regex(ref re) = t.inner {
            assert_eq!(re.as_str(), "^(?-u:fo*)$");
        } else {
            panic!("Unexpected Inner: {:?}", t.inner);
        }
        assert_eq!(f(b""), Some(42));
    } else {
        panic!("Unexpected Inner: {:?}", root.inner);
    }
}

#[test]
fn length_count_kleene_star() {
    let calc_regex = generate! {
        foo         = "foo";
        digit       = "0" - "9";
        calc_regex := digit.dummy, (foo*)#dummy;
    };
    let root = calc_regex.get_root();
    assert_eq!(root.name, Some("calc_regex".to_owned()));
    assert_eq!(root.length_bound, None);
    if let Inner::LengthCount { r, s, t, ref f } = root.inner {
        let r = calc_regex.get_node(r);
        assert_eq!(r.name, Some("digit".to_owned()));
        assert_eq!(r.length_bound, Some(1));
        if let Inner::Regex(ref re) = r.inner {
            assert_eq!(re.as_str(), "^(?-u:[0-9])$");
        } else {
            panic!("Unexpected Inner: {:?}", r.inner);
        }
        assert!(s.is_none());
        let t = calc_regex.get_node(t);
        assert_eq!(t.name, None);
        assert_eq!(t.length_bound, None);
        if let Inner::KleeneStar(re) = t.inner {
            let re = calc_regex.get_node(re);
            assert_eq!(re.name, Some("foo".to_owned()));
            assert_eq!(re.length_bound, Some(3));
            if let Inner::Regex(ref re) = re.inner {
                assert_eq!(re.as_str(), "^(?-u:foo)$");
            } else {
                panic!("Unexpected Inner: {:?}", t.inner);
            }
        }
        assert_eq!(f(b""), Some(42));
    } else {
        panic!("Unexpected Inner: {:?}", root.inner);
    }
}

#[test]
fn length_count_s_kleene_star() {
    let calc_regex = generate! {
        foo         = "foo";
        bar         = "bar";
        digit       = "0" - "9";
        calc_regex := digit.dummy, bar, (foo*)#dummy;
    };
    let root = calc_regex.get_root();
    assert_eq!(root.name, Some("calc_regex".to_owned()));
    assert_eq!(root.length_bound, None);
    if let Inner::LengthCount { r, s, t, ref f } = root.inner {
        let r = calc_regex.get_node(r);
        assert_eq!(r.name, Some("digit".to_owned()));
        assert_eq!(r.length_bound, Some(1));
        if let Inner::Regex(ref re) = r.inner {
            assert_eq!(re.as_str(), "^(?-u:[0-9])$");
        } else {
            panic!("Unexpected Inner: {:?}", r.inner);
        }
        assert!(s.is_some());
        let s = calc_regex.get_node(s.unwrap());
        assert_eq!(s.name, Some("bar".to_owned()));
        assert_eq!(s.length_bound, Some(3));
        if let Inner::Regex(ref re) = s.inner {
            assert_eq!(re.as_str(), "^(?-u:bar)$");
        } else {
            panic!("Unexpected Inner: {:?}", s.inner);
        }
        let t = calc_regex.get_node(t);
        assert_eq!(t.name, None);
        assert_eq!(t.length_bound, None);
        if let Inner::KleeneStar(re) = t.inner {
            let re = calc_regex.get_node(re);
            assert_eq!(re.name, Some("foo".to_owned()));
            assert_eq!(re.length_bound, Some(3));
            if let Inner::Regex(ref re) = re.inner {
                assert_eq!(re.as_str(), "^(?-u:foo)$");
            } else {
                panic!("Unexpected Inner: {:?}", t.inner);
            }
        }
        assert_eq!(f(b""), Some(42));
    } else {
        panic!("Unexpected Inner: {:?}", root.inner);
    }
}

#[test]
fn length_count_anonymous_regex() {
    let calc_regex = generate! {
        calc_regex := ("0" - "9").dummy, "foo" | "bar", ("o"+)#dummy;
    };
    let root = calc_regex.get_root();
    assert_eq!(root.name, Some("calc_regex".to_owned()));
    assert_eq!(root.length_bound, None);
    if let Inner::LengthCount { r, s, t, ref f } = root.inner {
        let r = calc_regex.get_node(r);
        assert_eq!(r.name, None);
        assert_eq!(r.length_bound, Some(1));
        if let Inner::Regex(ref re) = r.inner {
            assert_eq!(re.as_str(), "^(?-u:[0-9])$");
        } else {
            panic!("Unexpected Inner: {:?}", r.inner);
        }
        assert!(s.is_some());
        let s = calc_regex.get_node(s.unwrap());
        assert_eq!(s.name, None);
        assert_eq!(s.length_bound, Some(3));
        if let Inner::Regex(ref re) = s.inner {
            assert_eq!(re.as_str(), "^(?-u:foo|bar)$");
        } else {
            panic!("Unexpected Inner: {:?}", s.inner);
        }
        let t = calc_regex.get_node(t);
        assert_eq!(t.name, None);
        assert_eq!(t.length_bound, None);
        if let Inner::Regex(ref re) = t.inner {
            assert_eq!(re.as_str(), "^(?-u:o+)$");
        } else {
            panic!("Unexpected Inner: {:?}", t.inner);
        }
        assert_eq!(f(b""), Some(42));
    } else {
        panic!("Unexpected Inner: {:?}", root.inner);
    }
}

#[test]
fn length_count_anonymous_calc_regex() {
    let calc_regex = generate! {
        calc_regex := (("0" - "9")^3).dummy,
                      "foo" | "bar" , "baz",
                      ("f", "o"*)#dummy;
    };
    let root = calc_regex.get_root();
    assert_eq!(root.name, Some("calc_regex".to_owned()));
    assert_eq!(root.length_bound, None);
    if let Inner::LengthCount { r, s, t, ref f } = root.inner {
        let r = calc_regex.get_node(r);
        assert_eq!(r.name, None);
        assert_eq!(r.length_bound, Some(3));
        if let Inner::Regex(ref re) = r.inner {
            assert_eq!(re.as_str(), "^(?-u:([0-9]){3})$");
        } else {
            panic!("Unexpected Inner: {:?}", r.inner);
        }
        assert!(s.is_some());
        let s = calc_regex.get_node(s.unwrap());
        assert_eq!(s.name, None);
        assert_eq!(s.length_bound, None);
        if let Inner::Concat(lhs, rhs) = s.inner {
            let lhs = calc_regex.get_node(lhs);
            assert_eq!(lhs.name, None);
            assert_eq!(lhs.length_bound, Some(3));
            if let Inner::Regex(ref re) = lhs.inner {
                assert_eq!(re.as_str(), "^(?-u:foo|bar)$");
            } else {
                panic!("Unexpected Inner: {:?}", lhs.inner);
            }
            let rhs = calc_regex.get_node(rhs);
            assert_eq!(rhs.name, None);
            assert_eq!(rhs.length_bound, Some(3));
            if let Inner::Regex(ref re) = rhs.inner {
                assert_eq!(re.as_str(), "^(?-u:baz)$");
            } else {
                panic!("Unexpected Inner: {:?}", rhs.inner);
            }
        } else {
            panic!("Unexpected Inner: {:?}", s.inner);
        }
        let t = calc_regex.get_node(t);
        assert_eq!(t.name, None);
        assert_eq!(t.length_bound, None);
        if let Inner::Concat(lhs, rhs) = t.inner {
            let lhs = calc_regex.get_node(lhs);
            assert_eq!(lhs.name, None);
            assert_eq!(lhs.length_bound, Some(1));
            if let Inner::Regex(ref re) = lhs.inner {
                assert_eq!(re.as_str(), "^(?-u:f)$");
            } else {
                panic!("Unexpected Inner: {:?}", lhs.inner);
            }
            let rhs = calc_regex.get_node(rhs);
            assert_eq!(rhs.name, None);
            assert_eq!(rhs.length_bound, None);
            if let Inner::Regex(ref re) = rhs.inner {
                assert_eq!(re.as_str(), "^(?-u:o*)$");
            } else {
                panic!("Unexpected Inner: {:?}", rhs.inner);
            }
        } else {
            panic!("Unexpected Inner: {:?}", t.inner);
        }
        assert_eq!(f(b""), Some(42));
    } else {
        panic!("Unexpected Inner: {:?}", root.inner);
    }
}

#[test]
fn concatenate_length_count() {
    let calc_regex = generate! {
        foo         = "f", "o"*;
        digit       = "0" - "9";
        calc_regex := "foo", digit.dummy, foo#dummy, "bar";
    };
    let root = calc_regex.get_root();
    assert_eq!(root.name, Some("calc_regex".to_owned()));
    assert_eq!(root.length_bound, None);
    if let Inner::Concat(lhs, rhs) = root.inner {
        let lhs = calc_regex.get_node(lhs);
        assert_eq!(lhs.name, None);
        assert_eq!(lhs.length_bound, Some(3));
        if let Inner::Regex(ref re) = lhs.inner {
            assert_eq!(re.as_str(), "^(?-u:foo)$");
        } else {
            panic!("Unexpected Inner: {:?}", lhs.inner);
        }
        let rhs = calc_regex.get_node(rhs);
        assert_eq!(rhs.name, None);
        assert_eq!(rhs.length_bound, None);
        if let Inner::Concat(lhs, rhs) = rhs.inner {
            let lhs = calc_regex.get_node(lhs);
            assert_eq!(lhs.name, None);
            assert_eq!(lhs.length_bound, None);
            if let Inner::LengthCount { r, s, t, ref f } = lhs.inner {
                let r = calc_regex.get_node(r);
                assert_eq!(r.name, Some("digit".to_owned()));
                assert_eq!(r.length_bound, Some(1));
                if let Inner::Regex(ref re) = r.inner {
                    assert_eq!(re.as_str(), "^(?-u:[0-9])$");
                } else {
                    panic!("Unexpected Inner: {:?}", r.inner);
                }
                assert!(s.is_none());
                let t = calc_regex.get_node(t);
                assert_eq!(t.name, Some("foo".to_owned()));
                assert_eq!(t.length_bound, None);
                if let Inner::Regex(ref re) = t.inner {
                    assert_eq!(re.as_str(), "^(?-u:fo*)$");
                } else {
                    panic!("Unexpected Inner: {:?}", t.inner);
                }
                assert_eq!(f(b""), Some(42));
            } else {
                panic!("Unexpected Inner: {:?}", lhs.inner);
            }
            let rhs = calc_regex.get_node(rhs);
            assert_eq!(rhs.name, None);
            assert_eq!(rhs.length_bound, Some(3));
            if let Inner::Regex(ref re) = rhs.inner {
                assert_eq!(re.as_str(), "^(?-u:bar)$");
            } else {
                panic!("Unexpected Inner: {:?}", rhs.inner);
            }
        } else {
            panic!("Unexpected Inner: {:?}", rhs.inner);
        }
    } else {
        panic!("Unexpected Inner: {:?}", root.inner);
    }
}

#[test]
fn concatenate_length_count_s() {
    let calc_regex = generate! {
        foo         = "f", "o"*;
        bar         = "bar";
        digit       = "0" - "9";
        calc_regex := "foo", digit.dummy, bar, foo#dummy, "bar";
    };
    let root = calc_regex.get_root();
    assert_eq!(root.name, Some("calc_regex".to_owned()));
    assert_eq!(root.length_bound, None);
    if let Inner::Concat(lhs, rhs) = root.inner {
        let lhs = calc_regex.get_node(lhs);
        assert_eq!(lhs.name, None);
        assert_eq!(lhs.length_bound, Some(3));
        if let Inner::Regex(ref re) = lhs.inner {
            assert_eq!(re.as_str(), "^(?-u:foo)$");
        } else {
            panic!("Unexpected Inner: {:?}", lhs.inner);
        }
        let rhs = calc_regex.get_node(rhs);
        assert_eq!(rhs.name, None);
        assert_eq!(rhs.length_bound, None);
        if let Inner::Concat(lhs, rhs) = rhs.inner {
            let lhs = calc_regex.get_node(lhs);
            assert_eq!(lhs.name, None);
            assert_eq!(lhs.length_bound, None);
            if let Inner::LengthCount { r, s, t, ref f } = lhs.inner {
                let r = calc_regex.get_node(r);
                assert_eq!(r.name, Some("digit".to_owned()));
                assert_eq!(r.length_bound, Some(1));
                if let Inner::Regex(ref re) = r.inner {
                    assert_eq!(re.as_str(), "^(?-u:[0-9])$");
                } else {
                    panic!("Unexpected Inner: {:?}", r.inner);
                }
                assert!(s.is_some());
                let s = calc_regex.get_node(s.unwrap());
                assert_eq!(s.name, Some("bar".to_owned()));
                assert_eq!(s.length_bound, Some(3));
                if let Inner::Regex(ref re) = s.inner {
                    assert_eq!(re.as_str(), "^(?-u:bar)$");
                } else {
                    panic!("Unexpected Inner: {:?}", s.inner);
                }
                let t = calc_regex.get_node(t);
                assert_eq!(t.name, Some("foo".to_owned()));
                assert_eq!(t.length_bound, None);
                if let Inner::Regex(ref re) = t.inner {
                    assert_eq!(re.as_str(), "^(?-u:fo*)$");
                } else {
                    panic!("Unexpected Inner: {:?}", t.inner);
                }
                assert_eq!(f(b""), Some(42));
            } else {
                panic!("Unexpected Inner: {:?}", lhs.inner);
            }
            let rhs = calc_regex.get_node(rhs);
            assert_eq!(rhs.name, None);
            assert_eq!(rhs.length_bound, Some(3));
            if let Inner::Regex(ref re) = rhs.inner {
                assert_eq!(re.as_str(), "^(?-u:bar)$");
            } else {
                panic!("Unexpected Inner: {:?}", rhs.inner);
            }
        } else {
            panic!("Unexpected Inner: {:?}", rhs.inner);
        }
    } else {
        panic!("Unexpected Inner: {:?}", root.inner);
    }
}

#[test]
#[should_panic]
fn length_count_invalid() {
    let _ = generate! {
        foo         = "f", "o"*;
        digit       = "0" - "9";
        calc_regex := digit.dummy, foo#dummy_2;
    };
}

///////////////////////////////////////////////////////////////////////////////
//      Occurrence Count
///////////////////////////////////////////////////////////////////////////////

#[test]
fn occurrence_count() {
    let calc_regex = generate! {
        foo         = ("a" - "z")^3;
        digit       = "0" - "9";
        calc_regex := digit.dummy, foo^dummy;
    };
    let root = calc_regex.get_root();
    assert_eq!(root.name, Some("calc_regex".to_owned()));
    assert_eq!(root.length_bound, None);
    if let Inner::OccurrenceCount { r, s, t, ref f } = root.inner {
        let r = calc_regex.get_node(r);
        assert_eq!(r.name, Some("digit".to_owned()));
        assert_eq!(r.length_bound, Some(1));
        if let Inner::Regex(ref re) = r.inner {
            assert_eq!(re.as_str(), "^(?-u:[0-9])$");
        } else {
            panic!("Unexpected Inner: {:?}", r.inner);
        }
        assert!(s.is_none());
        let t = calc_regex.get_node(t);
        assert_eq!(t.name, Some("foo".to_owned()));
        assert_eq!(t.length_bound, Some(3));
        if let Inner::Regex(ref re) = t.inner {
            assert_eq!(re.as_str(), "^(?-u:([a-z]){3})$");
        } else {
            panic!("Unexpected Inner: {:?}", t.inner);
        }
        assert_eq!(f(b""), Some(42));
    } else {
        panic!("Unexpected Inner: {:?}", root.inner);
    }
}

#[test]
fn occurrence_count_s() {
    let calc_regex = generate! {
        foo         = "f" | "o";
        bar         = "bar";
        digit       = "0" - "9";
        calc_regex := digit.dummy, bar, foo^dummy;
    };
    let root = calc_regex.get_root();
    assert_eq!(root.name, Some("calc_regex".to_owned()));
    assert_eq!(root.length_bound, None);
    if let Inner::OccurrenceCount { r, s, t, ref f } = root.inner {
        let r = calc_regex.get_node(r);
        assert_eq!(r.name, Some("digit".to_owned()));
        assert_eq!(r.length_bound, Some(1));
        if let Inner::Regex(ref re) = r.inner {
            assert_eq!(re.as_str(), "^(?-u:[0-9])$");
        } else {
            panic!("Unexpected Inner: {:?}", r.inner);
        }
        assert!(s.is_some());
        let s = calc_regex.get_node(s.unwrap());
        assert_eq!(s.name, Some("bar".to_owned()));
        assert_eq!(s.length_bound, Some(3));
        if let Inner::Regex(ref re) = s.inner {
            assert_eq!(re.as_str(), "^(?-u:bar)$");
        } else {
            panic!("Unexpected Inner: {:?}", s.inner);
        }
        let t = calc_regex.get_node(t);
        assert_eq!(t.name, Some("foo".to_owned()));
        assert_eq!(t.length_bound, Some(1));
        if let Inner::Regex(ref re) = t.inner {
            assert_eq!(re.as_str(), "^(?-u:f|o)$");
        } else {
            panic!("Unexpected Inner: {:?}", t.inner);
        }
        assert_eq!(f(b""), Some(42));
    } else {
        panic!("Unexpected Inner: {:?}", root.inner);
    }
}

#[test]
#[should_panic]
fn occurrence_count_anonymous_regex() {
    let _ = generate! {
        calc_regex := ("0" - "9").dummy, "foo" | "bar", ("o"*)^dummy;
    };
    // let root = calc_regex.get_root();
    // assert_eq!(root.name, Some("calc_regex".to_owned()));
    // assert_eq!(root.length_bound, None);
    // if let Inner::OccurrenceCount { r, s, t, ref f } = root.inner {
    //     let r = calc_regex.get_node(r);
    //     assert_eq!(r.name, None);
    //     assert_eq!(r.length_bound, Some(1));
    //     if let Inner::Regex(ref re) = r.inner {
    //         assert_eq!(re.as_str(), "^(?-u:[0-9])$");
    //     } else {
    //         panic!("Unexpected Inner: {:?}", r.inner);
    //     }
    //     assert!(s.is_some());
    //     let s = calc_regex.get_node(s.unwrap());
    //     assert_eq!(s.name, None);
    //     assert_eq!(s.length_bound, Some(3));
    //     if let Inner::Regex(ref re) = s.inner {
    //         assert_eq!(re.as_str(), "^(?-u:foo|bar)$");
    //     } else {
    //         panic!("Unexpected Inner: {:?}", s.inner);
    //     }
    //     let t = calc_regex.get_node(t);
    //     assert_eq!(t.name, None);
    //     assert_eq!(t.length_bound, None);
    //     if let Inner::Regex(ref re) = t.inner {
    //         assert_eq!(re.as_str(), "^(?-u:o*)$");
    //     } else {
    //         panic!("Unexpected Inner: {:?}", t.inner);
    //     }
    //     assert_eq!(f(b""), Some(42));
    // } else {
    //     panic!("Unexpected Inner: {:?}", root.inner);
    // }
}

#[test]
#[should_panic]
fn occurrence_count_anonymous_calc_regex() {
    let _ = generate! {
        calc_regex := (("0" - "9")^3).dummy,
                      "foo" | "bar" , "baz",
                      ("f", "o"*)^dummy;
    };
    // let root = calc_regex.get_root();
    // assert_eq!(root.name, Some("calc_regex".to_owned()));
    // assert_eq!(root.length_bound, None);
    // if let Inner::OccurrenceCount { r, s, t, ref f } = root.inner {
    //     let r = calc_regex.get_node(r);
    //     assert_eq!(r.name, None);
    //     assert_eq!(r.length_bound, None);
    //     if let Inner::Repeat(node_index, n) = r.inner {
    //         assert_eq!(n, 3);
    //         let node = calc_regex.get_node(node_index);
    //         assert_eq!(node.name, None);
    //         assert_eq!(node.length_bound, Some(1));
    //         if let Inner::Regex(ref re) = node.inner {
    //             assert_eq!(re.as_str(), "^(?-u:[0-9])$");
    //         } else {
    //             panic!("Unexpected Inner: {:?}", node.inner);
    //         }
    //     } else {
    //         panic!("Unexpected Inner: {:?}", r.inner);
    //     }
    //     assert!(s.is_some());
    //     let s = calc_regex.get_node(s.unwrap());
    //     assert_eq!(s.name, None);
    //     assert_eq!(s.length_bound, None);
    //     if let Inner::Concat(lhs, rhs) = s.inner {
    //         let lhs = calc_regex.get_node(lhs);
    //         assert_eq!(lhs.name, None);
    //         assert_eq!(lhs.length_bound, Some(3));
    //         if let Inner::Regex(ref re) = lhs.inner {
    //             assert_eq!(re.as_str(), "^(?-u:foo|bar)$");
    //         } else {
    //             panic!("Unexpected Inner: {:?}", lhs.inner);
    //         }
    //         let rhs = calc_regex.get_node(rhs);
    //         assert_eq!(rhs.name, None);
    //         assert_eq!(rhs.length_bound, Some(3));
    //         if let Inner::Regex(ref re) = rhs.inner {
    //             assert_eq!(re.as_str(), "^(?-u:baz)$");
    //         } else {
    //             panic!("Unexpected Inner: {:?}", rhs.inner);
    //         }
    //     } else {
    //         panic!("Unexpected Inner: {:?}", s.inner);
    //     }
    //     let t = calc_regex.get_node(t);
    //     assert_eq!(t.name, None);
    //     assert_eq!(t.length_bound, None);
    //     if let Inner::Concat(lhs, rhs) = t.inner {
    //         let lhs = calc_regex.get_node(lhs);
    //         assert_eq!(lhs.name, None);
    //         assert_eq!(lhs.length_bound, Some(1));
    //         if let Inner::Regex(ref re) = lhs.inner {
    //             assert_eq!(re.as_str(), "^(?-u:f)$");
    //         } else {
    //             panic!("Unexpected Inner: {:?}", lhs.inner);
    //         }
    //         let rhs = calc_regex.get_node(rhs);
    //         assert_eq!(rhs.name, None);
    //         assert_eq!(rhs.length_bound, None);
    //         if let Inner::Regex(ref re) = rhs.inner {
    //             assert_eq!(re.as_str(), "^(?-u:o*)$");
    //         } else {
    //             panic!("Unexpected Inner: {:?}", rhs.inner);
    //         }
    //     } else {
    //         panic!("Unexpected Inner: {:?}", t.inner);
    //     }
    //     assert_eq!(f(b""), Some(42));
    // } else {
    //     panic!("Unexpected Inner: {:?}", root.inner);
    // }
}

#[test]
fn concatenate_occurrence_count() {
    let calc_regex = generate! {
        foo         = "f" | "o";
        digit       = "0" - "9";
        calc_regex := "foo", digit.dummy, foo^dummy, "bar";
    };
    let root = calc_regex.get_root();
    assert_eq!(root.name, Some("calc_regex".to_owned()));
    assert_eq!(root.length_bound, None);
    if let Inner::Concat(lhs, rhs) = root.inner {
        let lhs = calc_regex.get_node(lhs);
        assert_eq!(lhs.name, None);
        assert_eq!(lhs.length_bound, Some(3));
        if let Inner::Regex(ref re) = lhs.inner {
            assert_eq!(re.as_str(), "^(?-u:foo)$");
        } else {
            panic!("Unexpected Inner: {:?}", lhs.inner);
        }
        let rhs = calc_regex.get_node(rhs);
        assert_eq!(rhs.name, None);
        assert_eq!(rhs.length_bound, None);
        if let Inner::Concat(lhs, rhs) = rhs.inner {
            let lhs = calc_regex.get_node(lhs);
            assert_eq!(lhs.name, None);
            assert_eq!(lhs.length_bound, None);
            if let Inner::OccurrenceCount { r, s, t, ref f } = lhs.inner {
                let r = calc_regex.get_node(r);
                assert_eq!(r.name, Some("digit".to_owned()));
                assert_eq!(r.length_bound, Some(1));
                if let Inner::Regex(ref re) = r.inner {
                    assert_eq!(re.as_str(), "^(?-u:[0-9])$");
                } else {
                    panic!("Unexpected Inner: {:?}", r.inner);
                }
                assert!(s.is_none());
                let t = calc_regex.get_node(t);
                assert_eq!(t.name, Some("foo".to_owned()));
                assert_eq!(t.length_bound, Some(1));
                if let Inner::Regex(ref re) = t.inner {
                    assert_eq!(re.as_str(), "^(?-u:f|o)$");
                } else {
                    panic!("Unexpected Inner: {:?}", t.inner);
                }
                assert_eq!(f(b""), Some(42));
            } else {
                panic!("Unexpected Inner: {:?}", lhs.inner);
            }
            let rhs = calc_regex.get_node(rhs);
            assert_eq!(rhs.name, None);
            assert_eq!(rhs.length_bound, Some(3));
            if let Inner::Regex(ref re) = rhs.inner {
                assert_eq!(re.as_str(), "^(?-u:bar)$");
            } else {
                panic!("Unexpected Inner: {:?}", rhs.inner);
            }
        } else {
            panic!("Unexpected Inner: {:?}", rhs.inner);
        }
    } else {
        panic!("Unexpected Inner: {:?}", root.inner);
    }
}

#[test]
fn concatenate_occurrence_count_s() {
    let calc_regex = generate! {
        foo         = "f" | "o";
        bar         = "bar";
        digit       = "0" - "9";
        calc_regex := "foo", digit.dummy, bar, foo^dummy, "bar";
    };
    let root = calc_regex.get_root();
    assert_eq!(root.name, Some("calc_regex".to_owned()));
    assert_eq!(root.length_bound, None);
    if let Inner::Concat(lhs, rhs) = root.inner {
        let lhs = calc_regex.get_node(lhs);
        assert_eq!(lhs.name, None);
        assert_eq!(lhs.length_bound, Some(3));
        if let Inner::Regex(ref re) = lhs.inner {
            assert_eq!(re.as_str(), "^(?-u:foo)$");
        } else {
            panic!("Unexpected Inner: {:?}", lhs.inner);
        }
        let rhs = calc_regex.get_node(rhs);
        assert_eq!(rhs.name, None);
        assert_eq!(rhs.length_bound, None);
        if let Inner::Concat(lhs, rhs) = rhs.inner {
            let lhs = calc_regex.get_node(lhs);
            assert_eq!(lhs.name, None);
            assert_eq!(lhs.length_bound, None);
            if let Inner::OccurrenceCount { r, s, t, ref f } = lhs.inner {
                let r = calc_regex.get_node(r);
                assert_eq!(r.name, Some("digit".to_owned()));
                assert_eq!(r.length_bound, Some(1));
                if let Inner::Regex(ref re) = r.inner {
                    assert_eq!(re.as_str(), "^(?-u:[0-9])$");
                } else {
                    panic!("Unexpected Inner: {:?}", r.inner);
                }
                assert!(s.is_some());
                let s = calc_regex.get_node(s.unwrap());
                assert_eq!(s.name, Some("bar".to_owned()));
                assert_eq!(s.length_bound, Some(3));
                if let Inner::Regex(ref re) = s.inner {
                    assert_eq!(re.as_str(), "^(?-u:bar)$");
                } else {
                    panic!("Unexpected Inner: {:?}", s.inner);
                }
                let t = calc_regex.get_node(t);
                assert_eq!(t.name, Some("foo".to_owned()));
                assert_eq!(t.length_bound, Some(1));
                if let Inner::Regex(ref re) = t.inner {
                    assert_eq!(re.as_str(), "^(?-u:f|o)$");
                } else {
                    panic!("Unexpected Inner: {:?}", t.inner);
                }
                assert_eq!(f(b""), Some(42));
            } else {
                panic!("Unexpected Inner: {:?}", lhs.inner);
            }
            let rhs = calc_regex.get_node(rhs);
            assert_eq!(rhs.name, None);
            assert_eq!(rhs.length_bound, Some(3));
            if let Inner::Regex(ref re) = rhs.inner {
                assert_eq!(re.as_str(), "^(?-u:bar)$");
            } else {
                panic!("Unexpected Inner: {:?}", rhs.inner);
            }
        } else {
            panic!("Unexpected Inner: {:?}", rhs.inner);
        }
    } else {
        panic!("Unexpected Inner: {:?}", root.inner);
    }
}

#[test]
#[should_panic]
fn occurrence_count_invalid() {
    let _ = generate! {
        foo         = "f", "o"*;
        digit       = "0" - "9";
        calc_regex := digit.dummy, foo^dummy_2;
    };
}
