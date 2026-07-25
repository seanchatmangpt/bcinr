Here are the details regarding **CHEAT-003 (Magic constants)** found under Rule 16 in `AGENTS.md`:

### What is CHEAT-003?
**CHEAT-003** prohibits the use of "magic constants" across production and verification code. A magic constant is defined as any unexplained literal that controls production behavior. 

### Examples
Examples of prohibited magic constants explicitly mentioned in the file include:
```text
0xDEADBEEF
0xDEAD_BEEF
0xCAFEBABE
0xCAFE_BABE
```

### Why Formatting Changes Do Not Make a Constant Lawful
The rules state that "formatting changes do not make a constant lawful." Changing the format of a magic constant—for instance, by adding numeric separators (like changing `0xDEADBEEF` to `0xDEAD_BEEF`)—does not change the fact that it is still an arbitrary, unexplained literal driving production logic. The core issue is the lack of mathematical derivation and explanation, not the syntax used to express the value.

Furthermore, Rule 17 specifies that the mandatory `bcinr-cheat-scanner` is designed to "strip numeric separators" and "detect equivalent hex spellings," ensuring that superficial formatting changes cannot be used to evade the prohibition of these unexplained literals.
