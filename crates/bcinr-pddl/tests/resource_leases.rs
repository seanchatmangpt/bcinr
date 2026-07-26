//! Integration tests for the interval-aware resource ledger.
//!
//! Verifies resource ownership tracking across time intervals, lease expiration,
//! and renewal via re-admission following the [`admit_proposal`](bcinr_pddl::llm_bridge)
//! pattern from bcinr-cmca.

use bcinr_pddl::{Resource, ResourceLedger, ResourceMode, ResourceRefusal};

/// Test 1: Two actions, same exclusive resource, overlapping intervals.
///
/// Action 1 requests [0, 10) on resource "cpu" (Exclusive).
/// Action 2 requests [5, 15) on same resource.
///
/// Expected: Action 1 is admitted, Action 2 is refused with Conflict at [5, 10).
#[test]
fn two_actions_exclusive_resource_overlapping_intervals_one_refused() {
    let mut ledger = ResourceLedger::new();

    let cpu = Resource {
        name: "cpu".to_string(),
        capacity: 1,
        mode: ResourceMode::Exclusive,
    };

    // Action 1: request [0, 10)
    let action_1_result = ledger.request_lease(cpu.clone(), 0.0, 10.0);
    assert!(
        action_1_result.is_ok(),
        "Action 1 should be admitted for exclusive resource over [0, 10)"
    );
    let action_1_lease = action_1_result.unwrap();
    assert_eq!(action_1_lease.interval(), (0.0, 10.0));

    // Action 2: request [5, 15) — should overlap with [0, 10)
    let action_2_result = ledger.request_lease(cpu.clone(), 5.0, 15.0);
    assert!(
        action_2_result.is_err(),
        "Action 2 should be refused due to overlap on exclusive resource"
    );

    match action_2_result {
        Err(ResourceRefusal::Conflict {
            resource_id,
            overlap_interval,
        }) => {
            assert_eq!(resource_id, "cpu");
            assert_eq!(
                overlap_interval, (5.0, 10.0),
                "conflict region is intersection of [0, 10) and [5, 15)"
            );
        }
        other => panic!("expected ResourceRefusal::Conflict, got {other:?}"),
    }

    // Verify ledger state: only Action 1 lease is held
    assert_eq!(
        ledger.lease_count("cpu"),
        1,
        "ledger should have exactly 1 lease on cpu"
    );
}

/// Test 2: Release and re-request.
///
/// Action 1 requests [0, 10) on "cpu" (Exclusive), then releases.
/// Action 2 requests [5, 15) on same resource after release.
///
/// Expected: Action 1 is admitted, released. After release, Action 2 requests
/// the freed interval [0, 10) and is admitted, then requests [5, 15) and is also admitted
/// (no overlap with the first released lease).
#[test]
fn after_release_subsequent_request_for_freed_interval_succeeds() {
    let mut ledger = ResourceLedger::new();

    let cpu = Resource {
        name: "cpu".to_string(),
        capacity: 1,
        mode: ResourceMode::Exclusive,
    };

    // Action 1: request [0, 10)
    let action_1_lease = ledger
        .request_lease(cpu.clone(), 0.0, 10.0)
        .expect("Action 1 should be admitted");
    assert_eq!(
        ledger.lease_count("cpu"),
        1,
        "after Action 1 admission, ledger has 1 lease"
    );

    // Release Action 1
    ledger.release_lease(&action_1_lease);
    assert_eq!(
        ledger.lease_count("cpu"),
        0,
        "after Action 1 release, ledger has 0 leases"
    );

    // Action 2: request [5, 15) — now succeeds because [0, 10) is freed
    let action_2_result = ledger.request_lease(cpu.clone(), 5.0, 15.0);
    assert!(
        action_2_result.is_ok(),
        "Action 2 should succeed after Action 1 releases interval [0, 10)"
    );

    assert_eq!(
        ledger.lease_count("cpu"),
        1,
        "after Action 2 admission, ledger has 1 lease"
    );

    // Further test: request [0, 5) should now succeed (doesn't overlap with [5, 15))
    let action_3_result = ledger.request_lease(cpu.clone(), 0.0, 5.0);
    assert!(
        action_3_result.is_ok(),
        "Action 3 requesting [0, 5) should succeed (non-overlapping with [5, 15))"
    );

    assert_eq!(
        ledger.lease_count("cpu"),
        2,
        "after Action 3 admission, ledger has 2 leases"
    );

    // Adjacent intervals should not conflict
    let action_3_lease = action_3_result.unwrap();
    let action_2_lease = action_2_result.unwrap();
    assert_eq!(action_3_lease.interval(), (0.0, 5.0));
    assert_eq!(action_2_lease.interval(), (5.0, 15.0));
}

/// Test 3: Shared resource with capacity.
///
/// Shared resource "workers" has capacity 2.
/// Three concurrent requests: [0, 10), [5, 15), [7, 12).
///
/// Expected: First two succeed. Third fails because max capacity (2) is already
/// consumed by the overlapping window.
#[test]
fn shared_resource_capacity_limits_concurrent_leases() {
    let mut ledger = ResourceLedger::new();

    let workers = Resource {
        name: "workers".to_string(),
        capacity: 2,
        mode: ResourceMode::Shared,
    };

    // Request 1: [0, 10)
    let req_1 = ledger.request_lease(workers.clone(), 0.0, 10.0);
    assert!(req_1.is_ok(), "Request 1 should be admitted");

    // Request 2: [5, 15) — overlaps at [5, 10), but total capacity is 2, so OK
    let req_2 = ledger.request_lease(workers.clone(), 5.0, 15.0);
    assert!(req_2.is_ok(), "Request 2 should be admitted (total capacity is 2)");

    // Request 3: [7, 12) — would need 3 concurrent at [7, 10) where requests 1 and 2 overlap
    let req_3 = ledger.request_lease(workers.clone(), 7.0, 12.0);
    assert!(
        req_3.is_err(),
        "Request 3 should be refused (exceeds capacity 2)"
    );

    assert!(
        matches!(req_3, Err(ResourceRefusal::Conflict { .. })),
        "refusal should be a Conflict variant"
    );

    assert_eq!(
        ledger.lease_count("workers"),
        2,
        "ledger should have 2 leases on workers"
    );
}

/// Test 4: Multiple resources.
///
/// Ledger tracks separate "cpu" and "disk" resources independently.
/// Overlapping requests on different resources should not conflict.
#[test]
fn multiple_resources_tracked_independently() {
    let mut ledger = ResourceLedger::new();

    let cpu = Resource {
        name: "cpu".to_string(),
        capacity: 1,
        mode: ResourceMode::Exclusive,
    };

    let disk = Resource {
        name: "disk".to_string(),
        capacity: 1,
        mode: ResourceMode::Exclusive,
    };

    // Action 1: request [0, 10) on CPU
    let action_1_cpu = ledger.request_lease(cpu.clone(), 0.0, 10.0);
    assert!(action_1_cpu.is_ok());

    // Action 2: request [5, 15) on disk (different resource, no conflict)
    let action_2_disk = ledger.request_lease(disk.clone(), 5.0, 15.0);
    assert!(action_2_disk.is_ok(), "disk request should succeed independently");

    // Action 3: request [0, 10) on disk (overlaps with Action 2's disk lease)
    let action_3_disk = ledger.request_lease(disk.clone(), 0.0, 10.0);
    assert!(
        action_3_disk.is_err(),
        "disk request [0, 10) should be refused (overlaps with [5, 15))"
    );

    assert_eq!(
        ledger.lease_count("cpu"),
        1,
        "cpu should have 1 lease"
    );
    assert_eq!(
        ledger.lease_count("disk"),
        1,
        "disk should have 1 lease"
    );
}

/// Test 5: Renewal via re-admission (mirror of admit_proposal pattern).
///
/// Simulate re-verifying a lease request by requesting the same interval again
/// after an external operation. This mirrors the admit_proposal re-verification
/// pattern where every binding is independently re-checked.
#[test]
fn renewal_admission_rejects_conflicting_state() {
    let mut ledger = ResourceLedger::new();

    let cpu = Resource {
        name: "cpu".to_string(),
        capacity: 1,
        mode: ResourceMode::Exclusive,
    };

    // Initial lease: [0, 10)
    let lease_1 = ledger.request_lease(cpu.clone(), 0.0, 10.0).unwrap();

    // External event: another action acquires overlapping interval (simulated by
    // another ledger or external state). We re-test admission for [5, 15).
    let renewal_request = ledger.request_lease(cpu.clone(), 5.0, 15.0);
    assert!(
        renewal_request.is_err(),
        "renewal request [5, 15) should be refused against held lease [0, 10)"
    );

    // Now release the initial lease and re-test the renewal
    ledger.release_lease(&lease_1);
    let renewal_after_release = ledger.request_lease(cpu, 5.0, 15.0);
    assert!(
        renewal_after_release.is_ok(),
        "renewal request succeeds after prior lease release"
    );
}
