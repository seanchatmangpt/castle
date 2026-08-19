#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DdUiRefusal {
    IrreversiblePresentationSelection,
    RenderAuthorityEscalation,
    DirectDoFromUi,
    UnadmittedConstruct,
    NonBrceDo,
    UnreplayablePresentation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PresentationAuthority<'a> {
    pub irreversible_selections: u32,
    pub render_actuation_authority: bool,
    pub output_kind: &'a str,
    pub construct_admitted: bool,
    pub do_route: &'a str,
    pub grammar_digest: Option<&'a str>,
    pub world_digest: Option<&'a str>,
    pub frontier_digest: Option<&'a str>,
    pub screen_digest: Option<&'a str>,
}

impl PresentationAuthority<'_> {
    pub fn admit(&self) -> Result<(), DdUiRefusal> {
        if self.irreversible_selections != 0 {
            return Err(DdUiRefusal::IrreversiblePresentationSelection);
        }
        if self.render_actuation_authority {
            return Err(DdUiRefusal::RenderAuthorityEscalation);
        }
        if self.output_kind != "intent" {
            return Err(DdUiRefusal::DirectDoFromUi);
        }
        if !self.construct_admitted {
            return Err(DdUiRefusal::UnadmittedConstruct);
        }
        if self.do_route != "BRCE" {
            return Err(DdUiRefusal::NonBrceDo);
        }
        if [
            self.grammar_digest,
            self.world_digest,
            self.frontier_digest,
            self.screen_digest,
        ]
        .iter()
        .any(Option::is_none)
        {
            return Err(DdUiRefusal::UnreplayablePresentation);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{DdUiRefusal, PresentationAuthority};

    fn admitted() -> PresentationAuthority<'static> {
        PresentationAuthority {
            irreversible_selections: 0,
            render_actuation_authority: false,
            output_kind: "intent",
            construct_admitted: true,
            do_route: "BRCE",
            grammar_digest: Some("g"),
            world_digest: Some("w"),
            frontier_digest: Some("f"),
            screen_digest: Some("s"),
        }
    }

    #[test]
    fn admits_replayable_intent_only_ui() {
        assert_eq!(admitted().admit(), Ok(()));
    }

    #[test]
    fn refuses_render_authority_escalation() {
        let mut subject = admitted();
        subject.render_actuation_authority = true;
        assert_eq!(subject.admit(), Err(DdUiRefusal::RenderAuthorityEscalation));
    }

    #[test]
    fn refuses_non_brce_do() {
        let mut subject = admitted();
        subject.do_route = "direct";
        assert_eq!(subject.admit(), Err(DdUiRefusal::NonBrceDo));
    }

    #[test]
    fn refuses_missing_replay_binding() {
        let mut subject = admitted();
        subject.world_digest = None;
        assert_eq!(subject.admit(), Err(DdUiRefusal::UnreplayablePresentation));
    }
}
