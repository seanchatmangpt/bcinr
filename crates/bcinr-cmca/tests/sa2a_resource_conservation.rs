use bcinr_cmca::sa2a::{Allocation, BudgetLedger, ResourceEnvelope};

fn root() -> BudgetLedger {
    BudgetLedger::new(ResourceEnvelope {
        cpu: 100,
        memory: 1_000,
        io: 50,
    })
}

#[test]
fn parent_allocation_conserves_each_resource_dimension() {
    let ledger = root();
    let receipt = ledger.propose(Allocation {
        cpu: 25,
        memory: 400,
        io: 10,
    }).expect("bounded allocation");

    assert!(receipt.conserves());
    assert_eq!(receipt.before.cpu, receipt.after.cpu + receipt.granted.cpu);
    assert_eq!(receipt.before.memory, receipt.after.memory + receipt.granted.memory);
    assert_eq!(receipt.before.io, receipt.after.io + receipt.granted.io);
}

#[test]
fn oversized_allocation_is_refused_without_state() {
    let ledger = root();
    assert_eq!(
        ledger.propose(Allocation {
            cpu: 101,
            memory: 1,
            io: 1,
        }),
        None
    );
}

#[test]
fn recursive_child_cannot_amplify_parent_grant() {
    let ledger = root();
    let parent = ledger.propose(Allocation {
        cpu: 20,
        memory: 200,
        io: 5,
    }).expect("parent grant");

    let child = parent.child_ledger();
    assert_eq!(
        child.propose(Allocation {
            cpu: 21,
            memory: 100,
            io: 1,
        }),
        None
    );
}

#[test]
fn sequential_siblings_cannot_exceed_root_budget() {
    let ledger = root();
    let first = ledger.propose(Allocation {
        cpu: 60,
        memory: 500,
        io: 30,
    }).expect("first allocation");
    let ledger = ledger.advance(first).expect("conserving receipt");

    assert_eq!(
        ledger.propose(Allocation {
            cpu: 41,
            memory: 100,
            io: 1,
        }),
        None
    );

    let second = ledger.propose(Allocation {
        cpu: 40,
        memory: 500,
        io: 20,
    }).expect("exact residual");
    let ledger = ledger.advance(second).expect("second conserving receipt");

    assert_eq!(ledger.remaining, ResourceEnvelope::ZERO);
}

#[test]
fn forged_nonconserving_receipt_cannot_advance_ledger() {
    let ledger = root();
    let mut receipt = ledger.propose(Allocation {
        cpu: 1,
        memory: 1,
        io: 1,
    }).expect("bounded allocation");
    receipt.after.cpu += 1;

    assert_eq!(ledger.advance(receipt), None);
}
