//! Runtime-readable CASTLE product capability intake.
//!
//! These donors terminate at the CASTLE product crown. The registry is
//! side-effect free and preserves XaaS as the runtime core.

/// Exact ggen-ecosystem projection that selected these donors.
pub const PROJECTION_SOURCE: &str =
    "seanchatmangpt/ggen-ecosystem@50fdfa20c84205a80c6eb94e916cffbedc4b816e";

/// CASTLE's irreducible product capability.
pub const OWNER_CAPABILITY: &str = "CONSEQUENTIAL_ADMISSIBILITY";

/// Runtime core remains external to CASTLE.
pub const RUNTIME_CORE: &str = "seanchatmangpt/xaas";

/// Maximum authority this projection may manufacture.
pub const AUTHORITY_CEILING: &str = "CONSTRUCT";

/// One projected product donor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CapabilityDonor {
    /// Source repository.
    pub repository: &'static str,
    /// Exact source revision.
    pub sha: &'static str,
    /// Projected capability.
    pub capability: &'static str,
    /// Projection disposition.
    pub disposition: &'static str,
}

/// Product-crown donor set.
pub const DONORS: &[CapabilityDonor] = &[
    CapabilityDonor {
        repository: "seanchatmangpt/ash_surface",
        sha: "c066a55f452f23b348df375c00aae7811f6f72be",
        capability: "HUMAN_BOARD_SURFACE",
        disposition: "WRAP",
    },
    CapabilityDonor {
        repository: "seanchatmangpt/zoela",
        sha: "cae4a862744380f6a78b962e3f345ede8bd860dd",
        capability: "DOMAIN_APPLICATION_SURFACE",
        disposition: "WRAP",
    },
    CapabilityDonor {
        repository: "seanchatmangpt/cargo-cicd",
        sha: "59214d6a1293d794b7a27095276047cdff73a9ec",
        capability: "SOFTWARE_CONSEQUENCE_ADAPTER",
        disposition: "WRAP",
    },
    CapabilityDonor {
        repository: "seanchatmangpt/chatman-ecosystem",
        sha: "92cb17cda899a8d85abdaa04db10a3d2334e116d",
        capability: "STRATEGY_DOCTRINE_KNOWLEDGE",
        disposition: "KEEP_KNOWLEDGE_PLANE",
    },
    CapabilityDonor {
        repository: "seanchatmangpt/chatman-nano-stack",
        sha: "de7eb619310a522e44e9d6cf20e9460806b314ea",
        capability: "APPLICATION_CONSTITUTION_RESEARCH",
        disposition: "CANDIDATE_ABSORB",
    },
];

/// Look up a projected donor by repository identity.
#[must_use]
pub fn donor(repository: &str) -> Option<&'static CapabilityDonor> {
    DONORS.iter().find(|donor| donor.repository == repository)
}

/// No projection entry grants protected DO authority.
#[must_use]
pub const fn do_authority(_repository: &str) -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn product_registry_preserves_xaas_runtime_crown() {
        assert_eq!(OWNER_CAPABILITY, "CONSEQUENTIAL_ADMISSIBILITY");
        assert_eq!(RUNTIME_CORE, "seanchatmangpt/xaas");
        assert_eq!(AUTHORITY_CEILING, "CONSTRUCT");
        assert_eq!(DONORS.len(), 5);

        let cicd = donor("seanchatmangpt/cargo-cicd").expect("cargo-cicd donor");
        assert_eq!(cicd.capability, "SOFTWARE_CONSEQUENCE_ADAPTER");
        assert!(!do_authority(cicd.repository));
    }
}
