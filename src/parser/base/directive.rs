use super::*;

/// The implementation of the directives.
impl Parser<'_> {
    /// Match directives.
    pub fn directive(&mut self) -> PResult<()> {
        self.sym(b'%')?;
        self.context(|p| {
            if p.sym_seq(b"YAML").is_ok() {
                p.directive_yaml()
            } else if p.sym_seq(b"TAG").is_ok() {
                p.directive_tag()
            } else {
                // Unknown - ignore
                p.take_while(Self::not_in(b"\n\r"), TakeOpt::More(0))
            }
        })?;
        self.gap(true).map(|_| ())
    }

    fn directive_yaml(&mut self) -> PResult<()> {
        self.ws(TakeOpt::More(1))?;
        if self.version_checked {
            self.err("checked version")
        } else if !self.context(|p| p.sym_seq(b"1.1").is_ok() || p.sym_seq(b"1.2").is_ok()) {
            self.err("invalid version")
        } else {
            self.version_checked = true;
            Ok(())
        }
    }

    fn directive_tag(&mut self) -> PResult<()> {
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
