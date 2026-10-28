// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! A small reader for Valve's text format (`libraryfolders.vdf`, `appmanifest_*.acf`).
//!
//! Hand-written because the format is simple and a crate would add size and a licence entry.
//! Bad input gives an error, never a panic; nesting is capped so a hostile file cannot overflow the stack.

use std::fmt;

const MAX_DEPTH: usize = 32;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    Str(String),
    Map(Vec<(String, Value)>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub offset: usize,
    pub message: &'static str,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} (at byte {})", self.message, self.offset)
    }
}

impl std::error::Error for ParseError {}

impl Value {
    /// The text of a string value.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::Str(s) => Some(s),
            Value::Map(_) => None,
        }
    }

    /// The entries of a map value.
    pub fn entries(&self) -> &[(String, Value)] {
        match self {
            Value::Map(m) => m,
            Value::Str(_) => &[],
        }
    }

    /// The first child with this key (keys compare case-insensitively, as Steam's do).
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.entries()
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(key))
            .map(|(_, v)| v)
    }

    /// Follows a path of keys.
    pub fn path(&self, keys: &[&str]) -> Option<&Value> {
        keys.iter().try_fold(self, |v, k| v.get(k))
    }
}

/// Parses a whole document into a map holding its top-level keys.
pub fn parse(text: &str) -> Result<Value, ParseError> {
    let mut p = Parser {
        s: text.as_bytes(),
        i: 0,
    };
    let map = p.map(0, true)?;
    Ok(Value::Map(map))
}

struct Parser<'a> {
    s: &'a [u8],
    i: usize,
}

enum Token {
    Str(String),
    Open,
    Close,
    End,
}

impl Parser<'_> {
    fn err<T>(&self, message: &'static str) -> Result<T, ParseError> {
        Err(ParseError {
            offset: self.i,
            message,
        })
    }

    fn skip_space_and_comments(&mut self) {
        while self.i < self.s.len() {
            match self.s[self.i] {
                b' ' | b'\t' | b'\r' | b'\n' => self.i += 1,
                b'/' if self.s.get(self.i + 1) == Some(&b'/') => {
                    while self.i < self.s.len() && self.s[self.i] != b'\n' {
                        self.i += 1;
                    }
                }
                _ => break,
            }
        }
    }

    fn token(&mut self) -> Result<Token, ParseError> {
        self.skip_space_and_comments();
        let Some(&c) = self.s.get(self.i) else {
            return Ok(Token::End);
        };
        match c {
            b'{' => {
                self.i += 1;
                Ok(Token::Open)
            }
            b'}' => {
                self.i += 1;
                Ok(Token::Close)
            }
            b'"' => {
                self.i += 1;
                let mut out = Vec::new();
                loop {
                    let Some(&b) = self.s.get(self.i) else {
                        return self.err("unterminated string");
                    };
                    self.i += 1;
                    match b {
                        b'"' => break,
                        b'\\' => {
                            let Some(&e) = self.s.get(self.i) else {
                                return self.err("unterminated string");
                            };
                            self.i += 1;
                            out.push(match e {
                                b'n' => b'\n',
                                b't' => b'\t',
                                b'\\' => b'\\',
                                b'"' => b'"',
                                other => other,
                            });
                        }
                        other => out.push(other),
                    }
                }
                Ok(Token::Str(String::from_utf8_lossy(&out).into_owned()))
            }
            _ => self.err("expected a quoted string, { or }"),
        }
    }

    /// Reads `"key" "value"` and `"key" { ... }` pairs until `}` (or the end, at the top level).
    fn map(&mut self, depth: usize, top: bool) -> Result<Vec<(String, Value)>, ParseError> {
        if depth > MAX_DEPTH {
            return self.err("nested too deeply");
        }
        let mut out = Vec::new();
        loop {
            match self.token()? {
                Token::End if top => return Ok(out),
                Token::End => return self.err("missing closing }"),
                Token::Close if !top => return Ok(out),
                Token::Close => return self.err("unexpected }"),
                Token::Open => return self.err("{ without a key"),
                Token::Str(key) => match self.token()? {
                    Token::Str(v) => out.push((key, Value::Str(v))),
                    Token::Open => {
                        let inner = self.map(depth + 1, false)?;
                        out.push((key, Value::Map(inner)));
                    }
                    _ => return self.err("a key needs a value"),
                },
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LIBRARY: &str = r#"
"libraryfolders"
{
    "0"
    {
        "path"      "C:\\Program Files (x86)\\Steam"
        "apps"
        {
            "228980"    "123"
        }
    }
    "1"
    {
        "path"      "D:\\SteamLibrary" // second drive
        "apps"
        {
            "2246340"   "41000000000"
        }
    }
}
"#;

    #[test]
    fn parses_a_library_file() {
        let v = parse(LIBRARY).unwrap();
        let lib = v.get("libraryfolders").unwrap();
        assert_eq!(lib.entries().len(), 2);
        assert_eq!(
            lib.path(&["1", "path"]).and_then(Value::as_str),
            Some("D:\\SteamLibrary")
        );
        assert!(lib.path(&["1", "apps", "2246340"]).is_some());
        assert!(lib.path(&["0", "apps", "2246340"]).is_none());
    }

    #[test]
    fn keys_compare_case_insensitively() {
        let v = parse(r#""AppState" { "BuildID" "7" }"#).unwrap();
        assert_eq!(
            v.path(&["appstate", "buildid"]).and_then(Value::as_str),
            Some("7")
        );
    }

    #[test]
    fn malformed_input_is_an_error_not_a_panic() {
        for bad in [
            "\"a\"",
            "\"a\" {",
            "\"a\" { \"b\" }",
            "}",
            "{",
            "\"unterminated",
            "\"a\" \"b\\",
            "bare words",
        ] {
            assert!(parse(bad).is_err(), "{bad:?}");
        }
    }

    #[test]
    fn deep_nesting_is_refused() {
        let deep = "\"k\" {".repeat(200) + &"}".repeat(200);
        assert!(parse(&deep).is_err());
    }

    #[test]
    fn empty_and_comment_only_documents_are_empty_maps() {
        assert_eq!(parse("").unwrap(), Value::Map(vec![]));
        assert_eq!(parse("// nothing\n").unwrap(), Value::Map(vec![]));
    }

    #[test]
    fn escapes_are_decoded() {
        let v = parse(r#""k" "a\"b\\c\n""#).unwrap();
        assert_eq!(v.get("k").and_then(Value::as_str), Some("a\"b\\c\n"));
    }
}
