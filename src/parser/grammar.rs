use super::*;

impl<'a> Parser<'a> {
    /// Match symbol.
    pub fn sym(&mut self, s: u8) -> Result<(), PError> {
        if self.food().as_bytes()[0] == s {
            self.pos += 1;
            Ok(())
        } else {
            Err(PError::new(
                self.pos,
                &format!(": expect {}", String::from_utf8_lossy(&[s])),
            ))
        }
    }

    /// Match sequence.
    pub fn seq(&mut self, s: &[u8]) -> Result<(), PError> {
        let len = s.len();
        if self.pos + len <= self.doc.len() && {
            let mut b = true;
            for (i, c) in self.food().char_indices() {
                if i >= len {
                    break;
                }
                if c != char::from(s[i]) {
                    b = false;
                    break;
                }
            }
            b
        } {
            self.pos += len;
            Ok(())
        } else {
            Err(PError::new(
                self.pos,
                &format!(": expect {}", String::from_utf8_lossy(s)),
            ))
        }
    }

    /// Match until the condition failed.
    ///
    /// The argument `msg` can be `Option::None` if the string are empty allowed,
    /// otherwise there must be match at least one character.
    pub fn take_while<F>(&mut self, f: F, name: Option<&str>) -> Result<(), PError>
    where
        F: Fn(char) -> bool,
    {
        let mut pos = self.pos;
        for (i, c) in self.food().char_indices() {
            pos = self.pos + i + 1;
            if !f(c) {
                pos -= 1;
                break;
            }
        }
        if pos == self.pos {
            if let Some(name) = name {
                Err(PError::new(self.pos, name))
            } else {
                Ok(())
            }
        } else {
            self.pos = pos;
            Ok(())
        }
    }

    /// Match integer.
    pub fn int(&mut self) -> Result<(), PError> {
        self.sym(b'-').unwrap_or_default();
        self.take_while(|c| c.is_ascii_digit(), Some("integer"))?;
        Ok(())
    }

    /// Match float.
    pub fn float(&mut self) -> Result<(), PError> {
        self.int()?;
        self.sym(b'.')?;
        self.take_while(|c| c.is_ascii_digit(), Some("float"))?;
        Ok(())
    }

    /// Match quoted string.
    pub fn quoted_string(&mut self) -> Result<(), PError> {
        self.sym(b'\"')?;
        self.take_while(|c| c.is_ascii_digit(), Some("float"))?;
        self.sym(b'\"')?;
        Ok(())
    }

    /// Match valid YAML identifier.
    pub fn identifier(&mut self) -> Result<(), PError> {
        self.take_while(|c| c.is_alphanumeric() || c == '-', Some("identifier"))
    }

    /// Match any invisible characters.
    pub fn ws_any(&mut self) -> Result<(), PError> {
        self.take_while(|c| c.is_whitespace(), None)
    }

    /// String escaping.
    pub fn escape(doc: &str) -> String {
        let mut s = String::new();
        let mut b = false;
        for (_, c) in doc.char_indices() {
            if c == '\\' {
                b = true;
                continue;
            }
            s += match &c {
                '\\' if b => "\\",
                'n' if b => "\\n",
                'r' if b => "\\r",
                't' if b => "\\t",
                'b' if b => "\x08",
                'f' if b => "\x0C",
                v => {
                    s += &v.to_string();
                    b = false;
                    continue;
                }
            };
            b = false;
        }
        s
    }
}
