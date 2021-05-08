use super::*;

impl<'a> Parser<'a> {
    /// Match symbol.
    pub fn sym(&mut self, s: u8) -> Result<(), PError> {
        if self.doc[self.pos..].as_bytes()[0] == s {
            self.pos += 1;
            Ok(())
        } else {
            Err(PError::new(
                self.pos,
                &format!("expect {}", String::from_utf8_lossy(&[s])),
            ))
        }
    }

    /// Match sequence.
    pub fn seq(&mut self, s: &[u8]) -> Result<(), PError> {
        let len = s.len();
        if self.pos + len <= self.doc.len() && {
            let mut b = true;
            for (i, c) in self.doc[self.pos..].char_indices() {
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
                &format!("expect {}", String::from_utf8_lossy(s)),
            ))
        }
    }

    /// Match until the condition failed.
    ///
    /// This function will not move the cursor.
    pub fn take_while<F>(&mut self, f: F, msg: &str) -> Result<usize, PError>
    where
        F: Fn(usize, char) -> bool,
    {
        let mut pos = self.pos;
        for (i, c) in self.doc[self.pos..].char_indices() {
            pos = self.pos + i + 1;
            if !f(i, c) {
                pos -= 1;
                break;
            }
        }
        if pos == self.pos {
            Err(PError::new(self.pos, msg))
        } else {
            Ok(pos)
        }
    }

    /// Slice string, `pos` must bigger then current position.
    pub fn slice(&mut self, pos: usize) -> &'a str {
        assert!(pos > self.pos);
        let old_pos = self.pos;
        self.pos = pos;
        &self.doc[old_pos..pos]
    }

    /// Match integer and return the end position.
    ///
    /// This function will not move the cursor.
    pub fn int(&mut self) -> Result<usize, PError> {
        self.take_while(
            |i, c| {
                if i == 0 {
                    c == '-' || c.is_ascii_digit()
                } else {
                    c.is_ascii_digit()
                }
            },
            "invalid integer",
        )
    }

    /// Match front floating point and return the end position.
    ///
    /// This function will not move the cursor.
    pub fn float(&mut self) -> Result<usize, PError> {
        let pos = self.pos;
        // dbg!(&self.doc[self.pos..]);
        self.pos = match self.int() {
            Ok(v) => v,
            Err(_) => return Err(PError::new(self.pos, "invalid float")),
        };
        // dbg!(&self.doc[self.pos..]);
        if let Err(e) = self.sym(b'.') {
            self.pos = pos;
            return Err(e);
        }
        // dbg!(&self.doc[self.pos..]);
        let ret = self.take_while(|_, c| c.is_ascii_digit(), "invalid float");
        // dbg!(ret.is_ok());
        self.pos = pos;
        ret
    }

    /// Match valid YAML identifier.
    pub fn identifier(&mut self) -> Result<usize, PError> {
        self.take_while(|_, c| c.is_alphanumeric() || c == '-', "invalid identifier")
    }
}
