use super::*;

/// The option of [`Parser::take_while`].
pub enum TakeOpt {
    /// Match once.
    One,
    /// Match until not matched, allow mismatched.
    Any,
    /// Match until not matched, at least one.
    All,
}

/// The grammar implementation.
impl<'a> Parser<'a> {
    /// Match symbol.
    pub fn sym(&mut self, s: u8) -> Result<(), ()> {
        if self.food().as_bytes()[0] == s {
            self.pos += 1;
            Ok(())
        } else {
            Err(())
        }
    }

    /// Match sequence.
    pub fn seq(&mut self, s: &[u8]) -> Result<(), ()> {
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
            Err(())
        }
    }

    /// Match until the condition failed.
    ///
    /// The argument `opt` matches different terminate requirement.
    pub fn take_while<F>(&mut self, f: F, opt: TakeOpt) -> Result<(), ()>
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
            if let TakeOpt::One = opt {
                break;
            }
        }
        if pos == self.pos {
            if let TakeOpt::Any = opt {
                Ok(())
            } else {
                Err(())
            }
        } else {
            self.pos = pos;
            Ok(())
        }
    }

    /// Take only matched string from parser `f`.
    ///
    /// This function will move the eaten cursor to the front if matched,
    /// so [`Parser::eat`] method will skip the suffix.
    pub fn take_only<F, E>(&mut self, f: F) -> Result<(), ()>
    where
        F: Fn(&mut Self) -> Result<(), E>,
    {
        let pos = self.pos;
        if f(self).is_ok() {
            self.eaten = pos;
            Ok(())
        } else {
            Err(())
        }
    }

    /// Match integer.
    pub fn int(&mut self) -> Result<(), ()> {
        self.sym(b'-').unwrap_or_default();
        self.take_while(|c| c.is_ascii_digit(), TakeOpt::All)?;
        Ok(())
    }

    /// Match float.
    pub fn float(&mut self) -> Result<(), ()> {
        self.int()?;
        self.sym(b'.')?;
        self.take_while(|c| c.is_ascii_digit(), TakeOpt::All)?;
        Ok(())
    }

    /// Match quoted string.
    pub fn quoted_string(&mut self) -> Result<(), ()> {
        self.sym(b'\"')?;
        self.take_while(|c| c.is_ascii_digit(), TakeOpt::All)?;
        self.sym(b'\"')?;
        Ok(())
    }

    /// Match valid YAML identifier.
    pub fn identifier(&mut self) -> Result<(), ()> {
        self.take_while(char::is_alphanumeric, TakeOpt::One)?;
        self.take_while(|c| c.is_alphanumeric() || c == '-', TakeOpt::Any)
    }

    /// Match type assertion.
    pub fn ty(&mut self) -> Result<(), ()> {
        self.sym(b'!')?;
        self.sym(b'!').unwrap_or_default();
        self.take_only(Self::identifier)
    }

    /// Match anchor definition.
    pub fn anchor(&mut self) -> Result<(), ()> {
        self.sym(b'&')?;
        self.take_only(Self::identifier)
    }

    /// Match anchor used.
    pub fn anchor_use(&mut self) -> Result<(), ()> {
        self.sym(b'*')?;
        self.take_only(Self::identifier)
    }

    /// Match a white space.
    pub fn ws(&mut self) -> Result<(), ()> {
        self.take_while(|c| c == ' ', TakeOpt::All)
    }

    /// Match any invisible characters.
    pub fn ws_any(&mut self) -> Result<(), ()> {
        self.take_while(char::is_whitespace, TakeOpt::Any)
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
