import os
files = [
    "crates/bcinr-cmca/tests/case_studies.rs",
    "crates/bcinr-cmca/tests/differential.rs",
    "crates/bcinr-cmca/tests/hostile_mutants.rs"
]

for f in files:
    with open(f, "r") as fh:
        content = fh.read()
    
    content = content.replace("AdmittedControlState,", "AdmittedControlState::new(0),")
    content = content.replace("CertificateReceipt,", "CertificateReceipt::new(0),")
    content = content.replace("EnvelopeReceipt,", "EnvelopeReceipt::new(0),")
    content = content.replace("OutcomeReceipt,", "OutcomeReceipt::new(0),")
    content = content.replace("NonNegativeFixed::ONE,\n    )", "NonNegativeFixed::ONE,\n        CertifiedLearning::new(),\n    )")
    content = content.replace("NonNegativeFixed(65),\n    );", "NonNegativeFixed(65),\n        CertifiedLearning::new(),\n    );")
    content = content.replace("NonNegativeFixed(64),\n    );", "NonNegativeFixed(64),\n        CertifiedLearning::new(),\n    );")

    with open(f, "w") as fh:
        fh.write(content)
