use super::*;
use alloc::string::ToString;
use core::cmp::Ordering;

/// The low level grammar implementation for YAML.
///
/// These sub-parser returns `Result<R, ()>`, and calling [`Parser::backward`] if mismatched.
impl<R: repr::Repr> Parser<'_, R> {
    /// Match invisible boundaries and keep the gaps. (must matched once)
    pub fn bound(&mut self) -> Result<(), PError> {
        self.sym_set(b":{}[] ,\n\r")?;
        self.back(1);
        self.ws(TakeOpt::More(0))?;
        Ok(())
    }

    /// Match complex mapping indicator (`?`).
    pub fn complex_mapping(&mut self) -> Result<(), PError> {
        self.sym(b'?')?;
        self.bound()
    }

    fn num_prefix(&mut self) -> Result<(), PError> {
        self.sym(b'-').unwrap_or_default();
        self.take_while(u8::is_ascii_digit, TakeOpt::More(1))
    }

    /// Match integer.
    pub fn int(&mut self) -> Result<String, PError> {
        self.num_prefix()?;
        let s = self.text();
        self.bound()?;
        Ok(s)
    }

    /// Match float.
    pub fn float(&mut self) -> Result<String, PError> {
        self.num_prefix()?;
        self.sym(b'.')?;
        self.take_while(u8::is_ascii_digit, TakeOpt::More(0))?;
        let s = self.text();
        self.bound()?;
        Ok(s)
    }

    /// Match float with scientific notation.
    pub fn sci_float(&mut self) -> Result<String, PError> {
        self.num_prefix()?;
        self.sym_set(b"eE")?;
        self.take_while(Self::is_in(b"+-"), TakeOpt::Range(0, 1))?;
        self.take_while(u8::is_ascii_digit, TakeOpt::More(1))?;
        let s = self.text();
        self.bound()?;
        Ok(s)
    }

    /// Match NaN.
    pub fn nan(&mut self) -> Result<(), PError> {
        self.sym(b'.')?;
        for s in [b"nan", b"NaN", b"NAN"] {
            if self.seq(s).is_ok() {
                self.bound()?;
                return Ok(());
            }
        }
        Err(PError::Mismatch)
    }

    /// Match inf, return true if the value is positive.
    pub fn inf(&mut self) -> Result<bool, PError> {
        let b = self.sym(b'-').is_err();
        self.sym(b'.')?;
        for s in [b"inf", b"Inf", b"INF"] {
            if self.seq(s).is_ok() {
                self.bound()?;
                return Ok(b);
            }
        }
        Err(PError::Mismatch)
    }

    /// Match quoted string.
    pub fn string_quoted(&mut self, sym: u8, ignore: &[u8]) -> Result<String, PError> {
        self.context(|p| {
            p.sym(sym)?;
            p.forward();
            let mut v = String::new();
            p.ws(TakeOpt::More(0))?;
            v.push_str(&p.text());
            loop {
                p.forward();
                p.take_while(Self::not_in(&[b'\n', b'\r', b'\\', sym]), TakeOpt::More(0))?;
                v.push_str(&p.text());
                p.forward();
                if p.seq(ignore).is_ok() {
                    v.push(char::from(sym));
                } else if let Ok(mut t) = p.gap(false) {
                    if v.ends_with('\\') {
                        t -= 1;
                    }
                    match t.cmp(&1) {
                        Ordering::Less => {}
                        Ordering::Equal => {
                            v.truncate(v.trim_end().len());
                            // Manual wrapping
                            if !v.ends_with("\\n") {
                                v.push(' ');
                            }
                        }
                        Ordering::Greater => {
                            for _ in 0..t - 1 {
                                v.push('\n');
                            }
                        }
                    }
                    // Remove leading space
                    p.ws(TakeOpt::More(0))?;
                } else if p.sym(b'\\').is_ok() {
                    v.push('\\');
                } else if p.sym(sym).is_ok() {
                    break;
                }
            }
            Ok(v)
        })
    }

    /// Match plain string.
    pub fn string_plain(&mut self, level: usize, inner: bool) -> Result<String, PError> {
        let mut patt = b"[]{}: \n\r".to_vec();
        if inner {
            patt.push(b',');
        }
        self.context(|p| {
            let mut v = String::new();
            loop {
                p.forward();
                p.take_while(Self::not_in(&patt), TakeOpt::More(0))?;
                v.push_str(&p.text());
                p.forward();
                if p.seq(b": ").is_ok()
                    || (p.sym(b':').is_ok() && p.nl().is_ok())
                    || p.seq(b" #").is_ok()
                {
                    p.backward();
                    break;
                }
                p.forward();
                if p.sym_set(b": ").is_ok() {
                    // Remove leading space
                    if p.text() == " " {
                        v.truncate(v.trim_end().len());
                    }
                    v.push_str(&p.text());
                } else if !inner && !v.is_empty() && p.sym_set(b"{}[]").is_ok() {
                    v.push_str(&p.text());
                } else if p.ind(level).is_err() {
                    if let Ok(t) = p.gap(true) {
                        if t == 1 {
                            v.push(' ');
                        }
                        for _ in 0..t - 1 {
                            v.push('\n');
                        }
                        if p.ind(level).is_err() {
                            break;
                        }
                    } else {
                        break;
                    }
                }
            }
            v.truncate(v.trim_end().len());
            if v.is_empty() {
                Err(PError::Mismatch)
            } else {
                Ok(v)
            }
        })
    }

    /// Match flow string and return the content.
    pub fn string_flow(&mut self, level: usize, inner: bool) -> Result<String, PError> {
        if let Ok(s) = self.string_quoted(b'\'', b"''") {
            Ok(s)
        } else if let Ok(s) = self.string_quoted(b'"', b"\\\"") {
            Ok(Self::escape(&s))
        } else if let Ok(s) = self.string_plain(level, inner) {
            Ok(s)
        } else {
            Err(PError::Mismatch)
        }
    }

    /// Match literal string.
    pub fn string_literal(&mut self, level: usize) -> Result<String, PError> {
        self.sym(b'|')?;
        let chomp = self.chomp();
        self.ws(TakeOpt::More(0))?;
        let s = self.string_wrapped(level, b'\n', true)?;
        Ok(chomp(s))
    }

    /// Match folded string.
    pub fn string_folded(&mut self, level: usize) -> Result<String, PError> {
        self.sym(b'>')?;
        let chomp = self.chomp();
        self.ws(TakeOpt::More(0))?;
        let s = self.string_wrapped(level, b' ', false)?;
        Ok(chomp(s))
    }

    /// Match chomping option.
    pub fn chomp(&mut self) -> impl Fn(String) -> String {
        self.context(|p| {
            if p.sym(b'-').is_ok() {
                |s: String| s.trim_end().to_string()
            } else if p.sym(b'+').is_ok() {
                |s| s
            } else {
                |s: String| s.trim_end().to_string() + "\n"
            }
        })
    }

    /// Match wrapped string.
    pub fn string_wrapped(
        &mut self,
        level: usize,
        sep: u8,
        leading: bool,
    ) -> Result<String, PError> {
        self.context(|p| {
            let mut v = String::new();
            loop {
                p.nl()?;
                p.forward();
                if p.ind(level).is_err() {
                    if let Ok(t) = p.gap(false) {
                        for _ in 0..t {
                            v.push('\n');
                        }
                        if p.ind(level).is_err() {
                            break;
                        }
                    } else {
                        break;
                    }
                }
                p.forward();
                p.take_while(Self::not_in(b"\n\r"), TakeOpt::More(0))?;
                let s = p.text();
                if leading {
                    if !v.is_empty() {
                        v.push(char::from(sep));
                    }
                    v.push_str(&s);
                } else {
                    let s = s.trim_start();
                    if !v.is_empty() && !v.ends_with(char::is_whitespace) {
                        v.push(char::from(sep));
                    }
                    v.push_str(s);
                }
            }
            // Keep the last wrap
            p.back(1);
            Ok(v + "\n")
        })
    }

    /// String escaping, return a new string.
    pub fn escape(doc: &str) -> String {
        let mut s = String::new();
        let mut b = false;
        for c in doc.chars() {
            if c == '\\' && !b {
                b = true;
                continue;
            }
            s.push(match c {
                '\\' if b => '\\',
                'n' if b => '\n',
                'r' if b => '\r',
                't' if b => '\t',
                'b' if b => '\x08',
                'f' if b => '\x0C',
                c => c,
            });
            b = false;
        }
        s
    }

    /// Match valid YAML identifier.
    pub fn identifier(&mut self) -> Result<(), PError> {
        self.take_while(u8::is_ascii_alphanumeric, TakeOpt::One)?;
        self.take_while(
            |c| c.is_ascii_alphanumeric() || *c == b'-',
            TakeOpt::More(0),
        )
    }

    /// Match type assertion.
    pub fn ty(&mut self) -> Result<String, PError> {
        self.take_while(Self::is_in(b"!"), TakeOpt::Range(1, 2))?;
        self.context(|p| {
            p.identifier()?;
            Ok(p.text())
        })
    }

    /// Match anchor definition.
    pub fn anchor(&mut self) -> Result<String, PError> {
        self.sym(b'&')?;
        self.context(|p| {
            p.identifier()?;
            Ok(p.text())
        })
    }

    /// Match anchor used.
    pub fn anchor_use(&mut self) -> Result<String, PError> {
        self.sym(b'*')?;
        self.context(|p| {
            p.identifier()?;
            Ok(p.text())
        })
    }

    /// Match any invisible characters except newline.
    pub fn ws(&mut self, opt: TakeOpt) -> Result<(), PError> {
        self.take_while(
            |c| c.is_ascii_whitespace() && *c != b'\n' && *c != b'\r',
            opt,
        )
    }

    /// Match newline characters.
    pub fn nl(&mut self) -> Result<(), PError> {
        self.context(|p| {
            (p.seq(b"\r\n").is_ok()
                || p.seq(b"\n\r").is_ok()
                || p.sym(b'\n').is_ok()
                || p.sym(b'\r').is_ok())
            .then(|| ())
            .ok_or(PError::Mismatch)
        })
    }

    /// Match any invisible characters.
    pub fn inv(&mut self, opt: TakeOpt) -> Result<(), PError> {
        self.take_while(u8::is_ascii_whitespace, opt)
    }

    /// Match indent with previous level.
    ///
    /// Return `true` if downgrading indent is allowed.
    pub fn unind(&mut self, level: usize) -> Result<bool, PError> {
        if level > 0 {
            self.ind(level - 1)?;
            self.context(|p| Ok(p.ind(1).is_err()))
        } else {
            self.ind(level)?;
            Ok(false)
        }
    }

    /// Match any optional invisible characters between two lines.
    ///
    /// Set `cmt` to `true` to ignore comments at the line end.
    pub fn gap(&mut self, cmt: bool) -> Result<usize, PError> {
        self.context(|p| {
            if cmt {
                p.comment().unwrap_or_default();
            }
            p.nl()?;
            let mut t = 1;
            loop {
                // Check point
                p.forward();
                p.ws(TakeOpt::More(0))?;
                if cmt {
                    p.comment().unwrap_or_default();
                }
                if p.nl().is_err() {
                    p.backward();
                    return Ok(t);
                }
                t += 1;
            }
        })
    }

    /// Match comment.
    pub fn comment(&mut self) -> Result<(), PError> {
        self.ws(TakeOpt::More(0))?;
        self.sym(b'#')?;
        self.take_while(Self::not_in(b"\n\r"), TakeOpt::More(0))
    }
}
