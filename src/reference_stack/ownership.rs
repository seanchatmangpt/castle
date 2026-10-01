#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReferenceOwner {
    pub layer: &'static str,
    pub owner: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReferenceStage {
    SemanticState,
    GraphLaw,
    KnowledgeHook,
    Sa2aIntent,
    IndependentAuthorityEvidence,
    ReactorCommandBus,
    CastleBrce,
    ExternalOrSyntheticDo,
    IndependentPostcondition,
    ReceiptOcel,
    Beam4PmFeedback,
    XaasRuntimeComposition,
}

pub const GRAPHLAW_REFERENCE_HEAD: &str = "48a7bbd801b8df1d7ffab879b10d58d7f14ef7bc";
pub const ECOSYSTEM_REFERENCE_HEAD: &str = "bebffdeb2ea4dd6eace31d69298bfb51a073b265";

pub const REFERENCE_STAGES: &[ReferenceStage] = &[
    ReferenceStage::SemanticState,
    ReferenceStage::GraphLaw,
    ReferenceStage::KnowledgeHook,
    ReferenceStage::Sa2aIntent,
    ReferenceStage::IndependentAuthorityEvidence,
    ReferenceStage::ReactorCommandBus,
    ReferenceStage::CastleBrce,
    ReferenceStage::ExternalOrSyntheticDo,
    ReferenceStage::IndependentPostcondition,
    ReferenceStage::ReceiptOcel,
    ReferenceStage::Beam4PmFeedback,
    ReferenceStage::XaasRuntimeComposition,
];

pub const INHERITED_COURTS: &[&str] = &[
    "graphlaw:knowledge_hook:authority-none",
    "graphlaw:knowledge_hook:exact-subject",
    "graphlaw:knowledge_hook:source-bound",
    "graphlaw:knowledge_hook:replay-bound",
    "ash_a2a:prepared-effect:exact-identity",
    "castle:sa2a-security:prepared-effect-digest",
    "castle:brce:zero-unreceipted-do",
    "castle:receipt:outcome-bound",
    "beam4pm:receipt-ocel:feedback",
];

#[must_use]
pub const fn canonical_owners() -> &'static [ReferenceOwner] {
    &[
        ReferenceOwner { layer: "semantic-law", owner: "seanchatmangpt/graphlaw" },
        ReferenceOwner { layer: "application-projection", owner: "Ash/Ashler" },
        ReferenceOwner { layer: "observation-vkg", owner: "seanchatmangpt/ash_r2rml" },
        ReferenceOwner { layer: "consequence-thin-waist", owner: "seanchatmangpt/ash_a2a" },
        ReferenceOwner { layer: "trust-evidence", owner: "seanchatmangpt/affidavit" },
        ReferenceOwner { layer: "operational-realization", owner: "Reactor/CommandBus" },
        ReferenceOwner { layer: "process-evidence", owner: "seanchatmangpt/beam4pm" },
        ReferenceOwner { layer: "runtime-composition", owner: "seanchatmangpt/xaas" },
        ReferenceOwner { layer: "manufacture", owner: "seanchatmangpt/ggen-marketplace" },
        ReferenceOwner { layer: "product-constitution", owner: "seanchatmangpt/castle" },
    ]
}
