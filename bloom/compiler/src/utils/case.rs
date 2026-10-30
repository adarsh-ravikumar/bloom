use std::fmt::Write;

pub enum Case {
    PascalCase,
    CamelCase,
    SnakeCase,
}

enum Char {
    Lower,
    Upper,
    Number,
    Symbol,
}

pub fn normalize_case(source: &str, case: Case) -> String {
    assert!(source.len() > 0, "Expected non-empty id");

    let mut tokens = Vec::new();

    let mut prev_idx = 0usize;

    let mut prev_char: Char = match source.as_bytes().first().unwrap() {
        b'a'..b'z' => Char::Lower,
        b'A'..b'Z' => Char::Upper,
        _ => Char::Symbol,
    };

    for (i, byte) in source.bytes().enumerate() {
        if matches!(prev_char, Char::Lower | Char::Number)
            && matches!(byte, b'A'..b'Z')
            && prev_idx != i - 1
        {
            tokens.push(source[prev_idx..i].to_ascii_lowercase());
            prev_idx = i;
        } else if matches!(prev_char, Char::Upper)
            && matches!(byte, b'a'..b'z')
            && prev_idx != i - 1
        {
            tokens.push(source[prev_idx..i - 1].to_ascii_lowercase());
            prev_idx = i - 1;
        } else if byte == b'_' {
            if prev_idx < i {
                tokens.push(source[prev_idx..i].to_ascii_lowercase());
            }

            prev_idx = i + 1;
        }

        prev_char = match byte {
            b'a'..b'z' => Char::Lower,
            b'A'..b'Z' => Char::Upper,
            b'0'..b'9' => Char::Number,
            _ => Char::Symbol,
        };
    }

    if prev_idx < source.len() {
        tokens.push(source[prev_idx..].to_ascii_lowercase());
    }

    let mut res = String::new();

    for (i, part) in tokens.iter().enumerate() {
        match case {
            Case::CamelCase => {
                if i == 0 {
                    res.push_str(part);
                } else {
                    res.push_str(&part[0..1].to_ascii_uppercase());
                    res.push_str(&part[1..]);
                }
            }

            Case::PascalCase => {
                res.push_str(&part[0..1].to_ascii_uppercase());
                res.push_str(&part[1..]);
            }

            Case::SnakeCase => {
                if i > 0 {
                    res.push('_');
                }
                res.push_str(part);
            }
        }
    }

    res
}
