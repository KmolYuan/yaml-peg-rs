use super::*;
use alloc::{format, string::ToString};
use core::cmp::Ordering;

/// The low level grammar implementation for YAML.
///
/// These sub-parser returns `Result<R, PError>`, and calling [`Parser::backward`] if mismatched.
impl Parser<'_> {
    /// Match invisible boundaries and keep the gaps. (must matched once)
    pub fn bound(&mut self) -> PResult<()> {
        self.sym_set(b":{}[] ,\n\r")?;
        self.back(1);
        self.ws(TakeOpt::More(0))
    }

    /// Match complex mapping indicator (`?`).
    pub fn complex_mapping(&mut self) -> PResult<()> {
        self.sym(b'?')?;
        self.bound()
    }

    fn num_prefix(&mut self) -> PResult<()> {
        self.sym(b'-').unwrap_or_default();
        self.take_while(u8::is_ascii_digit, TakeOpt::More(1))
    }

    /// Match integer.
    pub fn int(&mut self) -> PResult<String> {
        self.num_prefix()?;
        let s = self.text();
        self.bound()?;
        Ok(s)
    }

    /// Match float.
    pub fn float(&mut self) -> PResult<String> {
        self.num_prefix()?;
        self.sym(b'.')?;
        self.take_while(u8::is_ascii_digit, TakeOpt::More(0))?;
        let s = self.text();
        self.bound()?;
        Ok(s.trim_end_matches(|c| ".0".contains(c)).to_string())
    }

    /// Match float with scientific notation.
    pub fn sci_float(&mut self) -> PResult<String> {
        self.num_prefix()?;
        self.sym_set(b"eE")?;
        self.take_while(Self::is_in(b"+-"), TakeOpt::Range(0, 1))?;
        self.take_while(u8::is_ascii_digit, TakeOpt::More(1))?;
        let s = self.text();
        self.bound()?;
        Ok(s)
    }

    /// Match quoted string.
    pub fn string_quoted(&mut self, sym: u8, ignore: &[u8]) -> PResult<String> {
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
                if p.sym_seq(ignore).is_ok() {
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
    pub fn string_plain(&mut self, level: usize, inner: bool) -> PResult<String> {
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
                if p.food().is_empty()
                    || p.sym_seq(b": ").is_ok()
                    || (p.sym(b':').is_ok() && p.nl().is_ok())
                    || p.sym_seq(b" #").is_ok()
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

    /// Match literal string.
    pub fn string_literal(&mut self, level: usize) -> PResult<String> {
        self.sym(b'|')?;
        let chomp = self.chomp();
        self.ws(TakeOpt::More(0))?;
        let s = self.string_wrapped(level, b'\n', true)?;
        Ok(chomp(s))
    }

    /// Match folded string.
    pub fn string_folded(&mut self, level: usize) -> PResult<String> {
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
    pub fn string_wrapped(&mut self, level: usize, sep: u8, leading: bool) -> PResult<String> {
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
    pub fn identifier(&mut self) -> PResult<()> {
        self.take_while(u8::is_ascii_alphanumeric, TakeOpt::One)?;
        self.take_while(
            |c| c.is_ascii_alphanumeric() || *c == b'-',
            TakeOpt::More(0),
        )
    }

    /// Match tags.
    pub fn tag(&mut self) -> PResult<String> {
        self.sym(b'!')?;
        self.context(|p| {
            p.identifier().unwrap_or_default();
            let tag = p.text();
            let prefix = if !tag.is_empty() {
                if p.sym(b'!').is_ok() {
                    // Tag prefix variable
                    p.tag[&tag].clone()
                } else {
                    tag
                }
            } else if p.sym(b'<').is_ok() {
                // Full tag
                let tag = p.context(|p| {
                    p.take_while(Self::not_in(b" <>\n\r"), TakeOpt::More(1))?;
                    Ok(p.text())
                })?;
                p.sym(b'>')?;
                tag
            } else if p.sym(b'!').is_ok() {
                p.tag["!!"].clone()
            } else {
                p.tag["!"].clone()
            };
            let doc = p.context(|p| {
                if p.identifier().is_ok() {
                    p.text()
                } else {
                    String::new()
                }
            });
            Ok(format!("{}{}", prefix, doc))
        })
    }

    /// Match anchor definition.
    pub fn anchor(&mut self) -> PResult<String> {
        self.sym(b'&')?;
        self.context(|p| {
            p.identifier()?;
            Ok(p.text())
        })
    }

    /// Match anchor used.
    pub fn anchor_use(&mut self) -> PResult<String> {
        self.sym(b'*')?;
        self.context(|p| {
            p.identifier()?;
            Ok(p.text())
        })
    }

    /// Match any invisible characters except newline.
    pub fn ws(&mut self, opt: TakeOpt) -> PResult<()> {
        self.take_while(
            |c| c.is_ascii_whitespace() && *c != b'\n' && *c != b'\r',
            opt,
        )
    }

    /// Match newline characters.
    pub fn nl(&mut self) -> PResult<()> {
        self.context(|p| {
            (p.sym_seq(b"\r\n").is_ok()
                || p.sym_seq(b"\n\r").is_ok()
                || p.sym(b'\n').is_ok()
                || p.sym(b'\r').is_ok())
            .then(|| ())
            .ok_or(PError::Mismatch)
        })
    }

    /// Match any invisible characters.
    pub fn inv(&mut self, opt: TakeOpt) -> PResult<()> {
        self.take_while(u8::is_ascii_whitespace, opt)
    }

    /// Match and define new indent size.
    pub fn ind_define(&mut self, level: usize) -> PResult<()> {
        if level > 0 {
            self.ind(level - 1)?;
        }
        let ind = self.count(|p| p.take_while(Self::is_in(b" "), TakeOpt::More(0)))?;
        if level == self.indent.len() {
            self.indent.push(ind);
        } else {
            self.indent[level] = ind;
        }
        Ok(())
    }

    /// Match indent with previous level.
    ///
    /// This sub-parser returns `true` if downgrading indent is used.
    ///
    /// The downgrading can only use a level as unit, not a whitespace.
    pub fn unind(&mut self, level: usize) -> PResult<bool> {
        if level > 0 {
            self.ind(level - 1)?;
            self.context(|p| Ok(p.ind(1).is_err()))
        } else {
            self.ind(0)?;
            Ok(false)
        }
    }

    /// Match any optional invisible characters between two lines.
    ///
    /// Set `cmt` to `true` to ignore comments at the line end.
    pub fn gap(&mut self, cmt: bool) -> PResult<usize> {
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
    pub fn comment(&mut self) -> PResult<()> {
        self.ws(TakeOpt::More(0))?;
        self.sym(b'#')?;
        self.take_while(Self::not_in(b"\n\r"), TakeOpt::More(0))
    }
}
