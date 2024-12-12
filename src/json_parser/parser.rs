use std::iter::Peekable;
use std::str::Chars;

use super::value::JsonValue;

struct Parser<'a> {
    chars: Peekable<Chars<'a>>,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        Parser {
            chars: input.chars().peekable(),
        }
    }

    fn parse_value(&mut self) -> Result<JsonValue, String> {
        self.skip_whitespace();
        match self.chars.peek() {
            Some(&'{') => self.parse_object(),
            Some(&'[') => self.parse_array(),
            Some(&'"') => self.parse_string().map(JsonValue::String),
            Some(&'-') | Some(&('0'..='9')) => self.parse_number(),
            Some(&'t') | Some(&'f') => self.parse_boolean(),
            Some(&'n') => self.parse_null(),
            Some(&c) => Err(format!("Unexpected character: {}", c)),
            None => Err("Unexpected end of input".to_string()),
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(&c) = self.chars.peek() {
            if !c.is_whitespace() {
                break;
            }
            self.chars.next();
        }
    }

    fn parse_object(&mut self) -> Result<JsonValue, String> {
        self.chars.next(); // Consume '{'
        let mut object = Vec::new();

        loop {
            self.skip_whitespace();
            if let Some(&'}') = self.chars.peek() {
                self.chars.next();
                return Ok(JsonValue::Object(object));
            }

            let key = self.parse_string()?;
            self.skip_whitespace();

            if self.chars.next() != Some(':') {
                return Err("Expected ':' in object".to_string());
            }

            let value = self.parse_value()?;
            object.push((key, value));

            self.skip_whitespace();
            match self.chars.next() {
                Some(',') => continue,
                Some('}') => return Ok(JsonValue::Object(object)),
                _ => return Err("Expected ',' or '}' in object".to_string()),
            }
        }
    }

    fn parse_array(&mut self) -> Result<JsonValue, String> {
        self.chars.next(); // Consume '['
        let mut array = Vec::new();

        loop {
            self.skip_whitespace();
            if let Some(&']') = self.chars.peek() {
                self.chars.next();
                return Ok(JsonValue::Array(array));
            }

            let value = self.parse_value()?;
            array.push(value);

            self.skip_whitespace();
            match self.chars.next() {
                Some(',') => continue,
                Some(']') => return Ok(JsonValue::Array(array)),
                _ => return Err("Expected ',' or ']' in array".to_string()),
            }
        }
    }

    fn parse_string(&mut self) -> Result<String, String> {
        self.chars.next(); // Consume opening '"'
        let mut string = String::new();

        while let Some(c) = self.chars.next() {
            match c {
                '"' => return Ok(string),
                '\\' => {
                    match self.chars.next() {
                        Some('"') => string.push('"'),
                        Some('\\') => string.push('\\'),
                        Some('/') => string.push('/'),
                        Some('b') => string.push('\u{0008}'),
                        Some('f') => string.push('\u{000C}'),
                        Some('n') => string.push('\n'),
                        Some('r') => string.push('\r'),
                        Some('t') => string.push('\t'),
                        Some('u') => {
                            // Parse 4-digit hex
                            let hex: String = self.chars.by_ref().take(4).collect();
                            if hex.len() != 4 {
                                return Err("Invalid unicode escape".to_string());
                            }
                            let code = u32::from_str_radix(&hex, 16)
                                .map_err(|_| "Invalid unicode escape".to_string())?;
                            string.push(
                                char::from_u32(code).ok_or("Invalid unicode escape".to_string())?,
                            );
                        }
                        _ => return Err("Invalid escape character".to_string()),
                    }
                }
                _ => string.push(c),
            }
        }
        Err("Unterminated string".to_string())
    }

    fn parse_number(&mut self) -> Result<JsonValue, String> {
        let mut number = String::new();

        if let Some(&'-') = self.chars.peek() {
            number.push(self.chars.next().unwrap());
        }

        while let Some(&c) = self.chars.peek() {
            if c.is_digit(10) || c == '.' || c == 'e' || c == 'E' || c == '+' || c == '-' {
                number.push(self.chars.next().unwrap());
            } else {
                break;
            }
        }

        number
            .parse::<f64>()
            .map(JsonValue::Number)
            .map_err(|_| "Invalid number".to_string())
    }

    fn parse_boolean(&mut self) -> Result<JsonValue, String> {
        match self.chars.peek() {
            Some(&'t') => {
                if self.consume_if_match("true") {
                    Ok(JsonValue::Boolean(true))
                } else {
                    Err("Expected 'true'".to_string())
                }
            }
            Some(&'f') => {
                if self.consume_if_match("false") {
                    Ok(JsonValue::Boolean(false))
                } else {
                    Err("Expected 'false'".to_string())
                }
            }
            _ => Err("Expected boolean".to_string()),
        }
    }

    fn parse_null(&mut self) -> Result<JsonValue, String> {
        if self.consume_if_match("null") {
            Ok(JsonValue::Null)
        } else {
            Err("Expected 'null'".to_string())
        }
    }

    fn consume_if_match(&mut self, expected: &str) -> bool {
        let mut chars = self.chars.clone();
        for exp_char in expected.chars() {
            if chars.next() != Some(exp_char) {
                return false;
            }
        }
        for _ in 0..expected.len() {
            self.chars.next();
        }
        true
    }
}

pub fn parse_json(input: &str) -> Result<JsonValue, String> {
    let mut parser = Parser::new(input);
    let value = parser.parse_value()?;
    parser.skip_whitespace();
    if parser.chars.next().is_some() {
        Err("Unexpected characters after JSON value".to_string())
    } else {
        Ok(value)
    }
}
