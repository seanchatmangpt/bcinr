# Interaction between Mask-Based Execution (Rule 9) and No Runtime Theorem Discovery (Rule 12)

The `bcinr` framework fundamentally relies on the synergistic interaction between **Rule 9 (Mask-Based Execution)** and **Rule 12 (No Runtime Theorem Discovery)** to guarantee its mandated $CC=1$ branchless deterministic execution. This interaction cleanly separates the complex mathematics of finding truth from the strict mechanics of acting upon it.

Here is how the substrate uses strict bitwise polynomials to branchlessly verify pre-calculated stability theorems rather than dynamically discovering them:

## 1. Upstream Discovery vs. Downstream Verification (Rule 12)
Rule 12 strictly forbids the authoritative runtime (the "hot path") from discovering mathematical bounds. Operations like spectral-radius estimation, adaptive threshold discovery, and Lyapunov searches are completely banned at runtime. 

Instead, all complex stability theorems are resolved on the **non-authoritative "slow rail"**. The slow rail searches for and identifies fixed stability constraints—such as a positive witness vector $d$, comparison matrix $G$, and margin $\delta$. It then provides these statically fixed values as a "supplied witness" to the hot path. The hot path's only responsibility is to mechanically verify the static domination law ($G d \leq (1-\delta)d$).

## 2. Compiling Verification to Bitwise Polynomials (Rule 9)
Because the runtime is only evaluating a pre-calculated constraint, it can do so using straightforward fixed-width arithmetic. Under Rule 9, the outcome of verifying this pre-calculated witness does not translate into a `bool` evaluated inside an `if / else` condition. 

Instead, the predicate check algebraically resolves to a **full-width bitwise mask** ($m \in \{0, 2^w-1\}$, representing all 1s or all 0s). 

## 3. The Synthesis: Branchless State Admission
The substrate unites these two rules when committing persistent state mutations. The mathematical proof of stability generates an admission mask which directly drives state selection through a strict bitwise polynomial:

$$ x_{t+1} = \text{select}(m_{\text{admitted}}, x_{\text{candidate}}, x_t) $$
$$ x_{t+1} = (m_{\text{admitted}} \land x_{\text{candidate}}) \lor (\neg m_{\text{admitted}} \land x_t) $$

If the supplied theorem parameters fail the static domination check, the algebraic result is a zero mask. Without executing a single branching jump instruction, the bitwise selection polynomial mathematically discards the candidate state and perfectly retains the current state bit-for-bit.

### Summary
By pairing **Rule 12** (which ensures the hot path only has to verify simple algebraic laws) with **Rule 9** (which mandates those verifications resolve strictly to selection masks), `bcinr` successfully forces "Rich semantics upstream, fixed deterministic mechanics downstream." The heavy lifting of theorem discovery is kept entirely off the authoritative call graph, allowing the runtime to branchlessly guarantee system stability via constant-time bitwise operations.
