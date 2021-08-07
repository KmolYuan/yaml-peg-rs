use super::*;

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

/// A PEG parser with YAML grammar, support UTF-8 characters.
///
/// A simple example for parsing YAML only:
///
/// ```
/// use yaml_peg::{parser::Parser, node};
///
/// let n = Parser::new(b"true").parse().unwrap();
/// assert_eq!(n, vec![node!(true)]);
/// ```
///
/// For matching partial grammar, each methods are the sub-parser.
/// The methods have some behaviers:
///
/// + They will move the current cursor if matched.
/// + Returned value:
///     + `Result<(), PError>` represents the sub-parser can be matched and mismatched.
///     + [`PError`] represents the sub-parser can be totally breaked when mismatched.
/// + Use `?` to match a condition.
/// + Use [`Result::unwrap_or_default`] to match an optional condition.
/// + Method [`Parser::forward`] is used to move on.
/// + Method [`Parser::text`] is used to get the matched string.
/// + Method [`Parser::backward`] is used to get back if mismatched.
pub struct Parser<'a, R: repr::Repr> {
    doc: &'a [u8],
    indent: usize,
    consumed: u64,
    /// Current position.
    pub pos: usize,
    /// Read position.
    pub eaten: usize,
    /// A visitor of anchors.
    pub anchors: AnchorBase<R>,
}

/// The implementation of string pointer.
impl<'a, R: repr::Repr> Parser<'a, R> {
    /// Create a PEG parser with the string.
    pub fn new(doc: &'a [u8]) -> Self {
        Self {
            doc,
            indent: 2,
            consumed: 0,
            pos: 0,
            eaten: 0,
            anchors: AnchorBase::new(),
        }
    }

    /// Show the right hand side string after the current cursor.
    pub fn food(&self) -> &'a [u8] {
        &self.doc[self.pos..]
    }

    /// Get the text from the eaten cursor to the current position.
    pub fn text(&mut self) -> String {
        if self.eaten < self.pos {
            String::from(String::from_utf8_lossy(&self.doc[self.eaten..self.pos]))
        } else {
            String::new()
        }
    }
}

/// The low level grammar implementation.
///
/// These sub-parser returns `Result<(), PError>`, and calling [`Parser::backward`] if mismatched.
impl<R: repr::Repr> Parser<'_, R> {
    /// Builder method for setting indent.
    pub fn indent(mut self, indent: usize) -> Self {
        self.indent = indent;
        self
    }

    /// Set the starting point if character boundary is valid.
    pub fn pos(mut self, pos: usize) -> Self {
        self.pos = pos;
        self.eaten = pos;
        self
    }

    /// Get the indicator.
    pub fn indicator(&self) -> u64 {
        self.consumed + self.pos as u64
    }

    /// A short function to raise error.
    pub fn err<Ret>(&self, msg: &'static str) -> Result<Ret, PError> {
        Err(PError::Terminate(self.indicator(), msg))
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
    pub fn sym(&mut self, s: u8) -> Result<(), PError> {
        self.sym_set(&[s])
    }

    /// Match symbol from a set.
    pub fn sym_set(&mut self, s: &[u8]) -> Result<(), PError> {
        self.take_while(Self::is_in(s), TakeOpt::One)
    }

    /// Match sequence.
    pub fn seq(&mut self, s: &[u8]) -> Result<(), PError> {
        for s in s {
            self.sym(*s)?;
        }
        Ok(())
    }

    /// Match until the condition failed.
    ///
    /// The argument `opt` matches different terminate requirement.
    pub fn take_while<F>(&mut self, f: F, opt: TakeOpt) -> Result<(), PError>
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

    /// A wrapper for saving local checkpoint.
    pub fn context<F, Ret>(&mut self, f: F) -> Ret
    where
        F: Fn(&mut Self) -> Ret,
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
    pub fn ind(&mut self, level: usize) -> Result<(), PError> {
        self.seq(&b" ".repeat(self.indent * level))
    }
}
