use super::*;
use alloc::string::ToString;

/// The implementation of the directives.
impl<R: repr::Repr> Parser<'_, R> {
    /// Match directives.
    pub fn directive(&mut self) -> Result<(), PError> {
        self.sym(b'%')?;
        self.ws(TakeOpt::More(0))?;
        self.context(|p| {
            if p.seq(b"YAML").is_ok() {
                p.directive_yaml()
            } else if p.seq(b"TAG").is_ok() {
                p.directive_tag()
            } else {
                // Unknown
                p.take_while(Self::not_in(b"\n\r"), TakeOpt::More(0))
            }
        })?;
        self.gap(true).map(|_| ())
    }

    fn directive_yaml(&mut self) -> Result<(), PError> {
        self.ws(TakeOpt::More(1))?;
        self.context(|p| {
            p.float()?;
            if p.version_checked || p.text() != "1.2" {
                Err(PError::Terminate(p.indicator(), "wrong version"))
            } else {
                p.version_checked = true;
                Ok(())
            }
        })
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
