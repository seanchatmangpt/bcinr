struct NNUE {
    l1_weights: array<i32, 12288>, // 768 * 16
    l1_biases: array<i32, 16>,
    l2_weights_value: array<i32, 16>,
    l2_weights_policy: array<i32, 1024>, // 16 hidden * 64 squares
    l2_bias_value: i32,
    _pad: array<i32, 3>,
};

struct Accumulator {
    hidden: array<i32, 16>,
};

@group(0) @binding(0) var<storage, read> nnue: NNUE;
@group(0) @binding(1) var<storage, read> accumulators: array<Accumulator>;
@group(0) @binding(2) var<storage, read_write> outputs_value: array<i32>;
@group(0) @binding(3) var<storage, read_write> outputs_policy: array<f32>;

fn branchless_relu(val: i32) -> i32 {
    let relu_mask = ~(val >> 31u);
    return val & bitcast<i32>(relu_mask);
}

// DeepMind PUCT requirement: fast branchless exponentiation (Base-2 approximation)
fn swar_exp2(x: f32) -> f32 {
    let bits = bitcast<i32>(x * 8388608.0 + 1065353216.0);
    return bitcast<f32>(bits);
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let idx = global_id.x;
    
    if (idx >= arrayLength(&outputs_value)) {
        return;
    }
    
    // 1. Fetch incrementally computed L1 from CPU
    var hidden: array<i32, 16>;
    for (var h = 0u; h < 16u; h = h + 1u) {
        hidden[h] = accumulators[idx].hidden[h];
    }
    
    // 2. AlphaZero Dual-Head Architecture
    var score_value: i32 = nnue.l2_bias_value;
    var policy_logits: array<f32, 64>;
    
    for (var h = 0u; h < 16u; h = h + 1u) {
        let activated = branchless_relu(hidden[h]);
        score_value += activated * nnue.l2_weights_value[h];
        
        for (var sq = 0u; sq < 64u; sq = sq + 1u) {
            policy_logits[sq] += f32(activated) * f32(nnue.l2_weights_policy[h * 64u + sq]) * 0.001;
        }
    }
    
    outputs_value[idx] = score_value;
    
    // 3. Branchless Base-2 Softmax (PUCT Priors)
    var max_logit = -100000.0;
    for (var sq = 0u; sq < 64u; sq = sq + 1u) {
        let mask = f32(policy_logits[sq] > max_logit);
        max_logit = policy_logits[sq] * mask + max_logit * (1.0 - mask);
    }
    
    var sum_exp = 0.0;
    var probs: array<f32, 64>;
    for (var sq = 0u; sq < 64u; sq = sq + 1u) {
        let exp_val = swar_exp2((policy_logits[sq] - max_logit) * 1.442695); // e^x approx
        probs[sq] = exp_val;
        sum_exp += exp_val;
    }
    
    // Store policy prior for best target square (compressing to 1 f32 per board for demo)
    outputs_policy[idx] = probs[0] / sum_exp; 
}
