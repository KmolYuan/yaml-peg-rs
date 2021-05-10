use super::*;

/// The option of [`Parser::take_while`].
pub enum TakeOpt {
    /// Match once.
    One,
    /// Match until mismatched, allow mismatched. Same as regex `*`.
    If,
    /// Match until mismatched, at least one. Same as regex `+`.
    Any,
    /// Match optional once. Same as regex `?`.
    OneOpt,
}

/// The low level grammar implementation.
///
/// These sub-parser returns `Result<(), ()>`, and calling [`Parser::backward`] if mismatched.
impl<'a> Parser<'a> {
    /// Move the eaten cursor to the current position and return the string.
    pub fn eat(&mut self) -> &'a str {
        if self.eaten < self.pos {
            let s = &self.doc[self.eaten..self.pos];
            self.eaten = self.pos;
            s
        } else {
            self.eaten = self.pos;
            ""
        }
    }

    /// Move the current position back.
    pub fn backward(&mut self) {
        self.pos = self.eaten;
    }

    /// Show the right hand side string after the current cursor.
    pub fn food(&self) -> &'a str {
        &self.doc[self.pos..]
    }

    /// Match optional symbol.
    pub fn opt(&mut self, s: u8) {
        self.take_while(|c| c == char::from(s), TakeOpt::OneOpt)
            .unwrap_or_default()
    }

    /// Match symbol.
    pub fn sym(&mut self, s: u8) -> Result<(), ()> {
        self.take_while(|c| c == char::from(s), TakeOpt::One)
    }

    /// Match sequence.
    pub fn seq(&mut self, s: &[u8]) -> Result<(), ()> {
        for s in s {
            self.sym(*s)?;
        }
        Ok(())
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
            if let TakeOpt::One | TakeOpt::OneOpt = opt {
                break;
            }
        }
        if pos == self.pos {
            if let TakeOpt::If | TakeOpt::OneOpt = opt {
                Ok(())
            } else {
                self.backward();
                Err(())
            }
        } else {
            self.pos = pos;
            Ok(())
        }
    }

    /// Select matched string from parser `f`.
    ///
    /// This function will move the eaten cursor to the front if matched,
    /// so [`Parser::eat`] method will skip the suffix.
    pub fn select<F>(&mut self, f: F) -> Result<(), ()>
    where
        F: Fn(&mut Self) -> Result<(), ()>,
    {
        let pos = self.pos;
        if f(self).is_ok() {
            self.eaten = pos;
            Ok(())
        } else {
            self.backward();
            Err(())
        }
    }

    /// Eat the matched string but require back boundary (spacing) for the matched parser.
    pub fn token<F>(&mut self, f: F) -> Result<&'a str, ()>
    where
        F: Fn(&mut Self) -> Result<(), ()>,
    {
        let eaten = self.eaten;
        f(self)?;
        let s = self.eat();
        if self.ws(TakeOpt::Any).is_ok() {
            self.eat();
            Ok(s)
        } else {
            // backward
            self.eaten = eaten;
            self.pos = eaten;
            Err(())
        }
    }

    /// Match integer.
    pub fn int(&mut self) -> Result<(), ()> {
        self.sym(b'-').unwrap_or_default();
        self.take_while(|c| c.is_ascii_digit(), TakeOpt::Any)?;
        Ok(())
    }

    /// Match float.
    pub fn float(&mut self) -> Result<(), ()> {
        self.int()?;
        self.sym(b'.')?;
        self.take_while(|c| c.is_ascii_digit(), TakeOpt::Any)?;
        Ok(())
    }

    /// Match quoted string.
    pub fn quoted_string(&mut self) -> Result<(), ()> {
        self.sym(b'\"')?;
        self.take_while(|c| c.is_ascii_digit(), TakeOpt::Any)?;
        self.sym(b'\"')?;
        Ok(())
    }

    /// Match valid YAML identifier.
    pub fn identifier(&mut self) -> Result<(), ()> {
        self.take_while(char::is_alphanumeric, TakeOpt::One)?;
        self.take_while(|c| c.is_alphanumeric() || c == '-', TakeOpt::If)
    }

    /// Match type assertion.
    pub fn ty(&mut self) -> Result<(), ()> {
        self.sym(b'!')?;
        self.sym(b'!').unwrap_or_default();
        self.select(Self::identifier)
    }

    /// Match anchor definition.
    pub fn anchor(&mut self) -> Result<(), ()> {
        self.sym(b'&')?;
        self.select(Self::identifier)
    }

    /// Match anchor used.
    pub fn anchor_use(&mut self) -> Result<(), ()> {
        self.sym(b'*')?;
        self.select(Self::identifier)
    }

    /// Match a white space.
    pub fn ws(&mut self, opt: TakeOpt) -> Result<(), ()> {
        self.take_while(|c| c == ' ', opt)
    }

    /// Match any optional invisible characters.
    pub fn inv(&mut self, opt: TakeOpt) -> Result<(), ()> {
        self.take_while(char::is_whitespace, opt)
    }

    /// Match any optional invisible characters between two lines.
    pub fn gap(&mut self) -> Result<(), ()> {
        let eaten = self.eaten;
        self.sym(b'\n')?;
        loop {
            // Check point
            self.eat();
            self.take_while(|c| c.is_whitespace() && c != '\n', TakeOpt::If)
                .unwrap_or_default();
            if let Err(()) = self.sym(b'\n') {
                self.eaten = eaten;
                return Ok(());
            }
        }
    }

    /// String escaping, return a new string.
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
