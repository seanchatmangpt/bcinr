import re
with open("crates/bcinr-logic/src/autonomic/receipt_integration.rs", "r") as f:
    content = f.read()

content = content.replace(
"""        let refusal_code = ((receipt_rejected_mask & 1) as u8)
            * (ReceiptIntegrationRefusal::LearningFrozen as u8)
            | ((learning_frozen_mask & 1) as u8)
                * (ReceiptIntegrationRefusal::ReceiptRejected as u8);""",
"""        let refusal_code = (((receipt_rejected_mask & 1) as u8)
            * (ReceiptIntegrationRefusal::LearningFrozen as u8))
            | (((learning_frozen_mask & 1) as u8)
                * (ReceiptIntegrationRefusal::ReceiptRejected as u8));"""
)

with open("crates/bcinr-logic/src/autonomic/receipt_integration.rs", "w") as f:
    f.write(content)
