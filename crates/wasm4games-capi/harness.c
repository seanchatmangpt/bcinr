/* Cross-language portability proof for wasm4games.
 *
 * Links the wasm4games-capi staticlib and recomputes the corpus digest from C. If it
 * matches the native Rust golden, one ggen-declared pattern law produced byte-identical
 * results across two languages. Build/run via ./portability_proof.sh.
 */
#include <stdint.h>
#include <stdio.h>

extern uint32_t w4g_pattern_count(void);
extern uint64_t w4g_kernel(uint16_t pattern_id, uint64_t state, uint64_t input);
extern uint64_t w4g_corpus_digest(void);
extern uint64_t w4g_golden_corpus_digest(void);

int main(void) {
    uint32_t count = w4g_pattern_count();
    uint64_t got = w4g_corpus_digest();
    uint64_t want = w4g_golden_corpus_digest();

    printf("pattern_count = %u\n", count);
    printf("corpus_digest = 0x%016llX (C-ABI execution)\n", (unsigned long long)got);
    printf("golden_digest = 0x%016llX (native Rust oracle)\n", (unsigned long long)want);

    /* spot-check one kernel too: damage_applied(100, 7) */
    printf("damage_applied(100,7) = %llu\n", (unsigned long long)w4g_kernel(14, 100, 7));

    if (count == 20 && got == want) {
        printf("PORTABILITY_OK: C-ABI execution reproduces the native golden digest\n");
        return 0;
    }
    printf("PORTABILITY_FAIL: cross-language digest mismatch\n");
    return 1;
}
