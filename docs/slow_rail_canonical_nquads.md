# Slow Rail `mfw-meaning` Canonical N-Quads in BCINR

In the `bcinr` architecture, the Authoritative Runtime (Hot Path) is governed by absolute deterministic laws (the Radon Law $CC=1$, `#![no_std]`, and zero heap allocation). Because dynamic Semantic Web data (`.ttl`/RDF) parsing fundamentally conflicts with these constraints, the semantic ontology is ingested and quarantined offline via the **Slow Rail** pipeline using the `mfw-meaning` tool.

Before generating any deterministic, branchless code for the hot path, `mfw-meaning` must mathematically normalize the ontology into an absolute, byte-stable representation. This ensures its cryptographic identity cannot be altered by insignificant formatting differences. 

This canonicalization process transforms the `.ttl` data into **Canonical N-Quads** and follows strict rules before computing a final **BLAKE3** hash:

## 1. Lexicographical Sorting
Instead of sorting term-by-term, all quads are strictly sorted lexicographically by their **full quad string** text representation (subject, predicate, object, graph).

## 2. UTF-8 and Line Endings
The entire payload must be strictly **UTF-8 encoded**. All line endings must be exclusively Unix style (`LF` / `\n`). Any Windows-style carriage returns (`\r\n`) must be aggressively normalized to `\n` prior to processing.

## 3. Stripping Insignificant Whitespace
Insignificant whitespace is strictly prohibited to guarantee byte-for-byte structural determinism:
*   There must be **no trailing whitespace** on any individual line.
*   There must be **no trailing blank lines** at the absolute end of the file.

## 4. BLAKE3 Cryptographic Binding (`rdf_digest`)
Once the ontology is fully canonicalized into this deterministic N-Quads byte stream, it is hashed using the **BLAKE3** algorithm. 

This yields the **`RDF_INPUT_DIGEST`** (`rdf_digest`), formatted as a lowercase hex string prefixed with `blake3:`. This cryptographic seal acts as the foundational root in the Slow Rail's `cmca_generation_receipt.json`. It securely binds the execution environment and guarantees that identical semantic input will produce the identical digest across any machine, time, or environment.
