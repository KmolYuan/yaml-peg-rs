use super::*;
use std::str::Bytes;

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
        self.forward();
        s
    }

    /// Consume the eaten part.
    pub fn forward(&mut self) {
        self.eaten = self.pos;
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
    pub fn food(&self) -> Bytes<'a> {
        self.doc[self.pos..].bytes()
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
        F: Fn(&u8) -> bool,
    {
        let pos = self.pos;
        let mut counter = 0;
        for c in self.food() {
            if !f(&c) {
                break;
            }
            self.pos += 1;
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
            if let TakeOpt::More(c) | TakeOpt::Range(c, _) = opt {
                if c == 0 {
                    return Ok(());
                }
            }
            self.backward();
            Err(())
        } else {
            if let TakeOpt::More(c) | TakeOpt::Range(c, _) = opt {
                if counter < c {
                    self.backward();
                    return Err(());
                }
            }
            Ok(())
        }
    }

    /// A wrapper for saving local checkpoint.
    pub fn context<F, R>(&mut self, f: F) -> R
    where
        F: Fn(&mut Self) -> R,
    {
        let eaten = self.eaten;
        self.forward();
        let r = f(self);
        self.eaten = eaten;
        r
    }

    /// A SET detector.
    pub fn is_in<'b>(s: &'b [u8]) -> impl Fn(&u8) -> bool + 'b {
        move |c| !Self::not_in(s)(c)
    }

    /// A NOT detector.
    pub fn not_in<'b>(s: &'b [u8]) -> impl Fn(&u8) -> bool + 'b {
        move |c| {
            for s in s {
                if c == s {
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
