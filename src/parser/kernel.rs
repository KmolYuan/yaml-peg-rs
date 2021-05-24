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
        let s = if self.eaten < self.pos {
            &self.doc[self.eaten..self.pos]
        } else {
            ""
        };
        self.eaten = self.pos;
        s
    }

    /// Move the current position back.
    pub fn backward(&mut self) {
        self.pos = self.eaten;
    }

    /// Move back current cursor.
    pub fn back(&mut self, n: usize) {
        self.pos -= n;
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

    /// A wrapper for saving local checkpoint.
    pub fn context<F, R>(&mut self, f: F) -> R
    where
        F: Fn(&mut Self) -> R,
    {
        let eaten = self.eaten;
        self.eat();
        let r = f(self);
        self.eaten = eaten;
        r
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
            let mut buff = [0; 4];
            s += match c {
                '\\' if b => "\\",
                'n' if b => "\\n",
                'r' if b => "\\r",
                't' if b => "\\t",
                'b' if b => "\x08",
                'f' if b => "\x0C",
                c => c.encode_utf8(&mut buff),
            };
            b = false;
        }
        s
    }
}
