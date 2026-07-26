#!/bin/bash
echo "=== Gate G9 — Terminal Release: Artifact Verification ==="
echo ""
echo "Verifying mandatory release artifacts..."
echo ""

ARTIFACTS=(
    "CONTRACT.md"
    "HOARE_TRIPLES.md"
    "COMMAND_TRANSCRIPT.md"
    "MUTANT_KILL_MATRIX.md"
    "OBJECT_CODE_AUDIT.md"
    "RELEASE_NOTES.md"
    "docs/contracts/HOARE_TRIPLES.md"
    "docs/gates/ORACLE_INDEPENDENCE.md"
)

MISSING=0
for artifact in "${ARTIFACTS[@]}"; do
    if [ -f "$artifact" ]; then
        echo "✓ $artifact"
    else
        echo "✗ $artifact (MISSING)"
        MISSING=$((MISSING + 1))
    fi
done

echo ""
if [ $MISSING -eq 0 ]; then
    echo "All mandatory artifacts verified: 8/8 ✓"
    exit 0
else
    echo "Missing artifacts: $MISSING"
    exit 1
fi
