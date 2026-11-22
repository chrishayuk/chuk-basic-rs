//! Just enough JSON to speak the language server protocol.

use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Json {
    Null,
    Bool(bool),
    Num(f64),
    Str(String),
    Arr(Vec<Json>),
    /// Members in the order they were written.
    Obj(Vec<(String, Json)>),
}

impl Json {
    pub fn obj<const N: usize>(members: [(&str, Json); N]) -> Json {
        Json::Obj(
            members
                .into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect(),
        )
    }

    pub fn str(text: impl Into<String>) -> Json {
        Json::Str(text.into())
    }

    pub fn num(n: impl Into<f64>) -> Json {
        Json::Num(n.into())
    }

    /// A member of an object; `Null` for anything that is not there.
    pub fn get(&self, key: &str) -> &Json {
        match self {
            Json::Obj(members) => members
                .iter()
                .find(|(k, _)| k == key)
                .map_or(&Json::Null, |(_, v)| v),
            _ => &Json::Null,
        }
    }

    /// An element of an array; `Null` for anything that is not there.
    pub fn at(&self, index: usize) -> &Json {
        match self {
            Json::Arr(items) => items.get(index).unwrap_or(&Json::Null),
            _ => &Json::Null,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Json::Str(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_u32(&self) -> Option<u32> {
        match self {
            Json::Num(n) if *n >= 0.0 && n.fract() == 0.0 && *n <= f64::from(u32::MAX) => {
                Some(*n as u32)
            }
            _ => None,
        }
    }

    pub fn parse(text: &str) -> Option<Json> {
        let mut parser = Parser {
            chars: text.chars().collect(),
            pos: 0,
        };
        let value = parser.value()?;
        parser.space();
        (parser.pos == parser.chars.len()).then_some(value)
    }
}

struct Parser {
    chars: Vec<char>,
    pos: usize,
}

impl Parser {
    fn space(&mut self) {
        while self.chars.get(self.pos).is_some_and(|c| c.is_whitespace()) {
            self.pos += 1;
        }
    }

    fn eat(&mut self, c: char) -> bool {
        self.space();
        let found = self.chars.get(self.pos) == Some(&c);
        if found {
            self.pos += 1;
        }
        found
    }

    fn word(&mut self, word: &str, value: Json) -> Option<Json> {
        let end = self.pos + word.len();
        let found = self.chars.get(self.pos..end)?.iter().collect::<String>() == word;
        self.pos = end;
        found.then_some(value)
    }

    fn value(&mut self) -> Option<Json> {
        self.space();
        match *self.chars.get(self.pos)? {
            'n' => self.word("null", Json::Null),
            't' => self.word("true", Json::Bool(true)),
            'f' => self.word("false", Json::Bool(false)),
            '"' => self.string().map(Json::Str),
            '[' => {
                self.pos += 1;
                let mut items = Vec::new();
                if self.eat(']') {
                    return Some(Json::Arr(items));
                }
                loop {
                    items.push(self.value()?);
                    if self.eat(']') {
                        return Some(Json::Arr(items));
                    }
                    if !self.eat(',') {
                        return None;
                    }
                }
            }
            '{' => {
                self.pos += 1;
                let mut members = Vec::new();
                if self.eat('}') {
                    return Some(Json::Obj(members));
                }
                loop {
                    self.space();
                    let key = self.string()?;
                    if !self.eat(':') {
                        return None;
                    }
                    members.push((key, self.value()?));
                    if self.eat('}') {
                        return Some(Json::Obj(members));
                    }
                    if !self.eat(',') {
                        return None;
                    }
                }
            }
            _ => {
                let from = self.pos;
                while self
                    .chars
                    .get(self.pos)
                    .is_some_and(|c| c.is_ascii_digit() || "+-.eE".contains(*c))
                {
                    self.pos += 1;
                }
                let text: String = self.chars[from..self.pos].iter().collect();
                text.parse().ok().map(Json::Num)
            }
        }
    }

    fn hex4(&mut self) -> Option<u32> {
        let digits: String = self.chars.get(self.pos..self.pos + 4)?.iter().collect();
        self.pos += 4;
        u32::from_str_radix(&digits, 16).ok()
    }

    fn string(&mut self) -> Option<String> {
        if self.chars.get(self.pos) != Some(&'"') {
            return None;
        }
        self.pos += 1;
        let mut out = String::new();
        loop {
            let c = *self.chars.get(self.pos)?;
            self.pos += 1;
            match c {
                '"' => return Some(out),
                '\\' => {
                    let escape = *self.chars.get(self.pos)?;
                    self.pos += 1;
                    out.push(match escape {
                        'n' => '\n',
                        't' => '\t',
                        'r' => '\r',
                        'b' => '\u{8}',
                        'f' => '\u{c}',
                        'u' => {
                            let mut code = self.hex4()?;
                            // a surrogate pair is one character
                            if (0xD800..0xDC00).contains(&code)
                                && self.chars.get(self.pos..self.pos + 2) == Some(&['\\', 'u'])
                            {
                                self.pos += 2;
                                let low = self.hex4()?;
                                code = 0x10000
                                    + ((code - 0xD800) << 10)
                                    + (low.wrapping_sub(0xDC00) & 0x3FF);
                            }
                            char::from_u32(code).unwrap_or('\u{FFFD}')
                        }
                        other => other,
                    });
                }
                c => out.push(c),
            }
        }
    }
}

impl fmt::Display for Json {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Json::Null => write!(f, "null"),
            Json::Bool(b) => write!(f, "{b}"),
            Json::Num(n) if n.fract() == 0.0 && n.abs() < 1e15 => write!(f, "{}", *n as i64),
            Json::Num(n) => write!(f, "{n}"),
            Json::Str(s) => {
                write!(f, "\"")?;
                for c in s.chars() {
                    match c {
                        '"' => write!(f, "\\\"")?,
                        '\\' => write!(f, "\\\\")?,
                        '\n' => write!(f, "\\n")?,
                        '\r' => write!(f, "\\r")?,
                        '\t' => write!(f, "\\t")?,
                        c if (c as u32) < 0x20 => write!(f, "\\u{:04x}", c as u32)?,
                        c => write!(f, "{c}")?,
                    }
                }
                write!(f, "\"")
            }
            Json::Arr(items) => {
                write!(f, "[")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, ",")?;
                    }
                    write!(f, "{item}")?;
                }
                write!(f, "]")
            }
            Json::Obj(members) => {
                write!(f, "{{")?;
                for (i, (key, value)) in members.iter().enumerate() {
                    if i > 0 {
                        write!(f, ",")?;
                    }
                    write!(f, "{}:{value}", Json::Str(key.clone()))?;
                }
                write!(f, "}}")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        let text = r#"{"jsonrpc":"2.0","id":1,"params":{"a":[1,2.5,-3],"t":true,"n":null,"s":"x\"y\\z\n↑"}}"#;
        let value = Json::parse(text).unwrap();
        assert_eq!(value.get("id").as_u32(), Some(1));
        assert_eq!(value.get("params").get("a").at(1), &Json::Num(2.5));
        assert_eq!(value.get("params").get("s").as_str(), Some("x\"y\\z\n↑"));
        assert_eq!(value.get("missing").get("deeper"), &Json::Null);
        assert_eq!(value.to_string(), text);
    }

    #[test]
    fn escapes_and_rubbish() {
        assert_eq!(Json::parse(r#""↑ 😀""#), Some(Json::str("↑ 😀")));
        assert_eq!(Json::parse(" [ ] "), Some(Json::Arr(vec![])));
        assert_eq!(Json::parse("{\"a\":}"), None);
        assert_eq!(Json::parse("[1,2"), None);
        assert_eq!(Json::parse("1 2"), None);
    }
}
