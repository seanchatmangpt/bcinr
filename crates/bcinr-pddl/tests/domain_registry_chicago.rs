//! Chicago TDD coverage for `DomainRegistry` version compatibility and
//! duplicate-source-root refusal.

#![cfg(feature = "mfw-planner")]

use bcinr_pddl::prelude::*;

fn versioned_domain(major: u16, minor: u16, patch: u16) -> VersionedDomain<&'static str> {
    VersionedDomain::new(
        DomainSourceRoot::hash(b"fulfillment-domain-v1"),
        CompiledDomainRoot::hash(b"fulfillment-compiled-v1"),
        DomainVersion::new(major, minor, patch),
        [
            "reserve-inventory".to_string(),
            "notify-customer".to_string(),
        ],
        "fulfillment",
    )
}

chicago_tdd_tools::test!(
    registry_requires_compatible_version_and_refuses_a_stale_requirement,
    {
        let mut registry = DomainRegistry::default();
        registry
            .insert(versioned_domain(1, 3, 0))
            .expect("first insertion should succeed");
        let root = DomainSourceRoot::hash(b"fulfillment-domain-v1");

        let compatible = registry
            .require_compatible(root, DomainVersion::new(1, 2, 0))
            .expect("newer minor/patch should satisfy an older requirement");
        assert_eq!(compatible.version(), DomainVersion::new(1, 3, 0));

        let incompatible = registry.require_compatible(root, DomainVersion::new(2, 0, 0));
        assert_eq!(
            incompatible,
            Err(DomainRegistryError::IncompatibleVersion {
                available: DomainVersion::new(1, 3, 0),
                required: DomainVersion::new(2, 0, 0),
            })
        );
    }
);

chicago_tdd_tools::test!(registry_refuses_a_duplicate_source_root, {
    let mut registry = DomainRegistry::default();
    registry
        .insert(versioned_domain(1, 0, 0))
        .expect("first insertion should succeed");
    let duplicate = registry.insert(versioned_domain(1, 0, 1));
    assert_eq!(
        duplicate,
        Err(DomainRegistryError::DuplicateSourceRoot(
            DomainSourceRoot::hash(b"fulfillment-domain-v1")
        ))
    );
    assert_eq!(registry.len(), 1);
});

chicago_tdd_tools::test!(registry_refuses_lookup_of_a_missing_domain, {
    let registry: DomainRegistry<&'static str> = DomainRegistry::default();
    let root = DomainSourceRoot::hash(b"never-inserted");
    assert_eq!(
        registry.get(root),
        Err(DomainRegistryError::MissingDomain(root))
    );
});
