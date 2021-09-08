use super::*;
use alloc::string::ToString;

/// The implementation of the directives.
impl<R: repr::Repr> Parser<'_, R> {
    /// Match directives.
    pub fn directive(&mut self) -> Result<(), PError> {
        self.sym(b'%')?;
        self.context(|p| {
            if p.seq(b"YAML").is_ok() {
                p.directive_yaml()
            } else if p.seq(b"TAG").is_ok() {
                p.directive_tag()
            } else {
                // Unknown - ignore
                p.take_while(Self::not_in(b"\n\r"), TakeOpt::More(0))
            }
        })?;
        self.gap(true).map(|_| ())
    }

    fn directive_yaml(&mut self) -> Result<(), PError> {
        self.ws(TakeOpt::More(1))?;
        if self.version_checked {
            self.err("checked version")
        } else if self.seq(b"1.2").is_err() {
            self.err("version")
        } else {
            self.version_checked = true;
            Ok(())
        }
    }

    fn directive_tag(&mut self) -> Result<(), PError> {
        self.ws(TakeOpt::More(1))?;
        self.sym(b'!')?;
        self.context(|p| {
            let tag = if p.identifier().is_ok() {
                let tag = p.text();
                p.sym(b'!')?;
                tag
            } else if p.sym(b'!').is_ok() {
                "!!".to_string()
            } else {
                "!".to_string()
            };
            p.ws(TakeOpt::More(1))?;
            let doc = p.context(|p| {
                p.take_while(Self::not_in(b" \n\r"), TakeOpt::More(1))?;
                Ok(p.text())
            })?;
            p.tag.insert(tag, doc);
            Ok(())
        })
    }
}
