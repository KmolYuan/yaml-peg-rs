use super::*;
use alloc::borrow::Cow;

mod directive;
mod grammar;

/// The option of [`Parser::take_while`].
pub enum TakeOpt {
    /// Match once.
    One,
    /// Match in range. Same as regex `{a,b}`.
    /// `Range(0, 1)` is same as regex `?`.
    Range(usize, usize),
    /// Match until mismatched.
    /// `More(0)` is same as regex `*`, and `More(1)` is same as regex `+`.
    More(usize),
}

/// Basic greedy parser with YAML syntax.
///
/// Its methods are actually the sub-parser of the syntax.
pub struct Parser<'a> {
    doc: &'a [u8],
    indent: Vec<usize>,
    consumed: u64,
    pub(crate) version_checked: bool,
    pub(crate) tag: BTreeMap<String, String>,
    /// Current position.
    pub pos: usize,
    /// Read position.
    pub eaten: usize,
}

impl Default for Parser<'_> {
    fn default() -> Self {
        let mut tag = BTreeMap::new();
        tag.insert("!".to_string(), String::new());
        tag.insert("!!".to_string(), tag_prefix!().to_string());
        Self {
            doc: b"",
            indent: vec![0],
            consumed: 0,
            version_checked: false,
            tag,
            pos: 0,
            eaten: 0,
        }
    }
}

/// The implementation of string pointer.
impl<'a> Parser<'a> {
    /// Create a parser with the string.
    pub fn new(doc: &'a [u8]) -> Self {
        Self::default().with_doc(doc)
    }

    /// Attach document on the parser.
    pub fn with_doc(self, doc: &'a [u8]) -> Self {
        Self { doc, ..self }
    }

    /// Show the right hand side string after the current cursor.
    pub fn food(&self) -> &'a [u8] {
        &self.doc[self.pos..]
    }

    /// Encoded version of the left characters.
    pub fn food_str(&self) -> Cow<str> {
        String::from_utf8_lossy(&self.doc[self.pos..])
    }

    /// Get the text from the eaten cursor to the current position.
    pub fn text(&mut self) -> String {
        if self.eaten < self.pos {
            String::from_utf8_lossy(&self.doc[self.eaten..self.pos]).into()
        } else {
            String::new()
        }
    }
}

/// The low level grammar implementation.
///
/// These sub-parser returns `Result<(), PError>`, and calling [`Parser::backward`] if mismatched.
impl Parser<'_> {
    /// Set the starting point if character boundary is valid.
    pub fn pos(self, pos: usize) -> Self {
        Self { pos, eaten: pos, ..self }
    }

    /// Get the indicator.
    pub fn indicator(&self) -> u64 {
        self.consumed + self.pos as u64
    }

    /// A short function to raise error.
    pub fn err<R>(&self, name: &'static str) -> PResult<R> {
        Err(PError::Terminate {
            name,
            msg: indicated_msg(self.doc, self.indicator()),
        })
    }

    /// Consume and move the pointer.
    pub fn consume(&mut self) {
        self.forward();
        self.consumed += self.eaten as u64;
        self.eaten = 0;
        self.backward();
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

    /// Match symbol.
    pub fn sym(&mut self, s: u8) -> PResult<()> {
        self.sym_set(&[s])
    }

    /// Match symbol set.
    pub fn sym_set(&mut self, s: &[u8]) -> PResult<()> {
        self.take_while(Self::is_in(s), TakeOpt::One)
    }

    /// Match symbol sequence.
    pub fn sym_seq(&mut self, s: &[u8]) -> PResult<()> {
        for s in s {
            self.sym(*s)?;
        }
        Ok(())
    }

    /// Match until the condition failed.
    ///
    /// The argument `opt` matches different terminate requirement.
    pub fn take_while<F>(&mut self, f: F, opt: TakeOpt) -> PResult<()>
    where
        F: Fn(&u8) -> bool,
    {
        let pos = self.pos;
        let mut counter = 0;
        for c in self.food() {
            if !f(c) {
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
            Err(PError::Mismatch)
        } else {
            if let TakeOpt::More(c) | TakeOpt::Range(c, _) = opt {
                if counter < c {
                    self.backward();
                    return Err(PError::Mismatch);
                }
            }
            Ok(())
        }
    }

    /// Count the position that parser goes, expect error.
    pub fn count<F, R>(&mut self, f: F) -> PResult<usize>
    where
        F: FnOnce(&mut Self) -> PResult<R>,
    {
        let pos = self.pos;
        let _ = f(self)?;
        Ok(self.pos - pos)
    }

    /// A wrapper for saving checkpoint locally.
    pub fn context<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut Self) -> R,
    {
        let eaten = self.eaten;
        self.forward();
        let r = f(self);
        self.eaten = eaten;
        r
    }

    /// A SET detector.
    pub fn is_in(s: &[u8]) -> impl Fn(&u8) -> bool + '_ {
        move |c| !Self::not_in(s)(c)
    }

    /// A NOT detector.
    pub fn not_in(s: &[u8]) -> impl Fn(&u8) -> bool + '_ {
        move |c| {
            for s in s {
                if c == s {
                    return false;
                }
            }
            true
        }
    }

    /// Match indent.
    pub fn ind(&mut self, level: usize) -> PResult<()> {
        if level >= self.indent.len() {
            for _ in 0..level - self.indent.len() + 1 {
                self.indent.push(2);
            }
        } else {
            // Clear the old indent settings
            self.indent.drain(level + 1..);
        }
        for _ in 0..self.indent[..=level].iter().sum() {
            self.sym(b' ')?;
        }
        Ok(())
    }
}
