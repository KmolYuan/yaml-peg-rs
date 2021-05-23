use super::*;

/// The option of [`Parser::take_while`].
pub enum TakeOpt {
    /// Match once.
    One,
    /// Match in range. Same as regex `{a,b}`.
    Range(usize, usize),
    /// Match until mismatched.
    /// `More(0)` is same as regex `*`, and `More(1)` is same as regex `?`.
    More(usize),
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

    /// Match symbol.
    pub fn sym(&mut self, s: u8) -> Result<(), ()> {
        self.take_while(Self::is_in(&[s]), TakeOpt::One)
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
        let mut counter = 0;
        for (i, c) in self.food().char_indices() {
            pos = self.pos + i;
            if !f(c) {
                break;
            }
            pos += 1;
            counter += 1;
            if let TakeOpt::One = opt {
                break;
            }
            if let TakeOpt::Range(_, c) = opt {
                if counter == c {
                    break;
                }
            }
        }
        if pos == self.pos {
            match opt {
                TakeOpt::More(c) | TakeOpt::Range(c, _) if c == 0 => Ok(()),
                _ => {
                    self.backward();
                    Err(())
                }
            }
        } else {
            match opt {
                TakeOpt::Range(c, _) | TakeOpt::More(c) if counter < c => {
                    self.backward();
                    Err(())
                }
                _ => {
                    while !self.doc.is_char_boundary(pos) {
                        pos += 1;
                    }
                    self.pos = pos;
                    Ok(())
                }
            }
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
            Err(())
        }
    }

    /// Match invisible boundaries and keep the gaps. (must matched once)
    pub fn bound(&mut self) -> Result<(), ()> {
        self.inv(TakeOpt::One)?;
        self.pos -= 1;
        self.ws(TakeOpt::More(0))?;
        Ok(())
    }

    /// Match complex mapping indicator (`?`).
    pub fn complex_mapping(&mut self) -> Result<(), ()> {
        self.sym(b'?')?;
        self.bound()
    }

    /// Match integer.
    pub fn int(&mut self) -> Result<(), ()> {
        self.sym(b'-').unwrap_or_default();
        self.take_while(|c| c.is_ascii_digit(), TakeOpt::More(1))
    }

    /// Match float.
    pub fn float(&mut self) -> Result<(), ()> {
        self.int()?;
        self.sym(b'.')?;
        self.take_while(|c| c.is_ascii_digit(), TakeOpt::More(0))
    }

    /// Match float with scientific notation.
    pub fn sci_float(&mut self) -> Result<(), ()> {
        self.int()?;
        self.take_while(Self::is_in(b"eE"), TakeOpt::One)?;
        self.take_while(Self::is_in(b"+-"), TakeOpt::Range(0, 1))?;
        self.take_while(|c| c.is_ascii_digit(), TakeOpt::More(1))
    }

    /// Match NaN.
    pub fn nan(&mut self) -> Result<(), ()> {
        self.sym(b'.')?;
        for &s in &[b"nan", b"NaN", b"NAN"] {
            if self.seq(s).is_ok() {
                return Ok(());
            }
        }
        Err(())
    }

    /// Match inf, return true if the value is positive.
    pub fn inf(&mut self) -> Result<bool, ()> {
        let b = self.sym(b'-').is_err();
        self.sym(b'.')?;
        for &s in &[b"inf", b"Inf", b"INF"] {
            if self.seq(s).is_ok() {
                return Ok(b);
            }
        }
        Err(())
    }

    /// Match quoted string.
    pub fn string_quoted(&mut self, sym: u8) -> Result<&'a str, ()> {
        let eaten = self.eaten;
        self.sym(sym)?;
        self.select(|p| p.take_while(Self::not_in(&[sym]), TakeOpt::More(1)))?;
        let s = self.eat();
        self.eaten = eaten;
        self.sym(sym)?;
        Ok(s)
    }

    /// Match plain string.
    pub fn string_plain(&mut self, use_sep: bool) -> Result<&'a str, ()> {
        let eaten = self.eaten;
        let mut p = b"[]{}: \n\r".iter().cloned().collect::<Vec<_>>();
        if use_sep {
            p.push(b',');
        }
        loop {
            self.take_while(Self::not_in(&p), TakeOpt::More(0))?;
            self.eat();
            if self.seq(b": ").is_ok() || self.seq(b":\n").is_ok() || self.seq(b" #").is_ok() {
                self.pos -= 2;
            } else if self.take_while(Self::is_in(b": "), TakeOpt::One).is_ok() {
                continue;
            }
            break;
        }
        self.eaten = eaten;
        let s = self.eat().trim_end();
        if s.is_empty() {
            Err(())
        } else {
            Ok(s)
        }
    }

    /// Match flow string and return the content.
    pub fn string_flow(&mut self, use_sep: bool) -> Result<&'a str, ()> {
        if let Ok(s) = self.string_quoted(b'\'') {
            Ok(s)
        } else if let Ok(s) = self.string_quoted(b'"') {
            Ok(s)
        } else if let Ok(s) = self.string_plain(use_sep) {
            Ok(s)
        } else {
            Err(())
        }
    }

    /// Match literal string.
    pub fn string_literal(&mut self, level: usize) -> Result<String, ()> {
        self.sym(b'|')?;
        let s = self.string_wrapped(level, "\n")?;
        Ok(s)
    }

    /// Match folded string.
    pub fn string_folded(&mut self, level: usize) -> Result<String, ()> {
        self.sym(b'>')?;
        let s = self.string_wrapped(level, " ")?;
        Ok(s)
    }

    /// Match wrapped string.
    pub fn string_wrapped(&mut self, level: usize, sep: &str) -> Result<String, ()> {
        let eaten = self.eaten;
        let mut v = vec![];
        loop {
            self.bound()?;
            self.inv(TakeOpt::One)?;
            self.eat();
            if self.indent(level).is_err() {
                break;
            }
            self.eat();
            self.take_while(Self::not_in(b"\n\r"), TakeOpt::More(0))?;
            v.push(self.eat());
        }
        self.pos -= 1;
        self.eaten = eaten;
        Ok(v.join(sep))
    }

    /// Match valid YAML identifier.
    pub fn identifier(&mut self) -> Result<(), ()> {
        self.take_while(char::is_alphanumeric, TakeOpt::One)?;
        self.take_while(|c| c.is_alphanumeric() || c == '-', TakeOpt::More(0))
    }

    /// Match type assertion.
    pub fn ty(&mut self) -> Result<&'a str, ()> {
        self.take_while(Self::is_in(b"!"), TakeOpt::Range(1, 2))?;
        self.select(Self::identifier)?;
        Ok(self.eat())
    }

    /// Match anchor definition.
    pub fn anchor(&mut self) -> Result<&'a str, ()> {
        self.sym(b'&')?;
        self.select(Self::identifier)?;
        Ok(self.eat())
    }

    /// Match anchor used.
    pub fn anchor_use(&mut self) -> Result<(), ()> {
        self.sym(b'*')?;
        self.select(Self::identifier)
    }

    /// Match any optional invisible characters except newline.
    pub fn ws(&mut self, opt: TakeOpt) -> Result<(), ()> {
        self.take_while(|c| c.is_whitespace() && !"\n\r".contains(c), opt)
    }

    /// Match any optional invisible characters.
    pub fn inv(&mut self, opt: TakeOpt) -> Result<(), ()> {
        self.take_while(char::is_whitespace, opt)
    }

    /// Match indent.
    pub fn indent(&mut self, level: usize) -> Result<(), ()> {
        self.seq(&b"  ".repeat(level))
    }

    /// Match any optional invisible characters between two lines.
    pub fn gap(&mut self) -> Result<(), ()> {
        let eaten = self.eaten;
        self.eat();
        self.comment().unwrap_or_default();
        self.sym(b'\n')?;
        loop {
            // Check point
            self.eat();
            self.ws(TakeOpt::More(0))?;
            if self.sym(b'\n').is_err() {
                self.eaten = eaten;
                return Ok(());
            }
        }
    }

    /// Match comment.
    pub fn comment(&mut self) -> Result<(), ()> {
        self.ws(TakeOpt::More(0))?;
        self.sym(b'#')?;
        self.take_while(Self::not_in(b"\n\r"), TakeOpt::More(0))
    }

    /// A SET detector for `char`.
    pub fn is_in<'b>(s: &'b [u8]) -> impl Fn(char) -> bool + 'b {
        move |c| !Self::not_in(s)(c)
    }

    /// A NOT detector for `char`.
    pub fn not_in<'b>(s: &'b [u8]) -> impl Fn(char) -> bool + 'b {
        move |c| {
            for s in s {
                if c == char::from(*s) {
                    return false;
                }
            }
            true
        }
    }

    /// String escaping, return a new string.
    pub fn escape(doc: &str) -> String {
        let mut s = String::new();
        let mut b = false;
        for c in doc.chars() {
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

    /// A short function to raise error.
    pub fn err<R>(&self, msg: &str) -> Result<R, PError> {
        Err(PError::Terminate(self.pos, msg.into()))
    }
}
