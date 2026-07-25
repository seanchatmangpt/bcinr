import math

def to_q16(val):
    return int(round(val * 65536))

log2_255 = math.log2(255)

print("pub const LOG2_TABLE: [i64; 256] = [")
for x in range(256):
    if x == 0:
        val = -200000000 # Enough to make mean_log < -523918 even with w=1
    else:
        val = to_q16(math.log2(x) - log2_255)
    print(f"    {val},")
print("];")

print("pub const EXP2_FRAC_TABLE: [u32; 256] = [")
for i in range(256):
    x = i / 256.0
    val = int(round((2**x) * 65536))
    print(f"    {val},")
print("];")

