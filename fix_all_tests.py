import re

for f in ['crates/bcinr-cmca/tests/differential.rs', 'crates/bcinr-cmca/tests/hostile_mutants.rs', 'crates/bcinr-cmca/tests/case_studies.rs', 'crates/bcinr-cmca/tests/calibration.rs']:
    with open(f, 'r') as file:
        content = file.read()
    
    # Authority Types replacements
    content = content.replace("AdmittedControlState::new(", "AdmittedControlState::admit_control_state(")
    content = content.replace("CertificateReceipt::new(", "CertificateReceipt::admit_certificate(")
    content = content.replace("EnvelopeReceipt::new(", "EnvelopeReceipt::admit_envelope(")
    content = content.replace("OutcomeReceipt::new(", "OutcomeReceipt::admit_outcome(")
    
    # CertifiedLearning::new() -> CertifiedLearning::admit_learning()
    content = content.replace("CertifiedLearning::new()", "CertifiedLearning::admit_learning()")
    
    # AdaptiveUpdate::new -> AdaptiveUpdate::admit_adaptive_update
    content = content.replace("AdaptiveUpdate::new", "AdaptiveUpdate::admit_adaptive_update")
    
    # Some of the above return Option<T> or Result<T, E>, we might need .unwrap() but let's see. 
    # Actually, in tests they were returning values. In C2, `admit_control_state` returns `AdmittedControlState` directly if it just wraps. Let's check allocator.rs for the signatures.
    
    # Fix NonNegativeFixed(X)
    content = re.sub(r'NonNegativeFixed\(([\d\-]+)\)', r'NonNegativeFixed::from_bits(\1)', content)
    content = re.sub(r'SignedFixed\(([\d\-]+)\)', r'SignedFixed::from_bits(\1)', content)
    
    # Fix .0
    content = content.replace("].0", "].val")
    content = content.replace("::ONE.0", "::ONE.val")
    content = content.replace("::ZERO.0", "::ZERO.val")
    content = content.replace(").0", ").val")
    
    with open(f, 'w') as file:
        file.write(content)
        
