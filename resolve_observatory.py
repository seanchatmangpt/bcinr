import re

with open('crates/bcinr-cmca/src/observatory.rs', 'r') as f:
    content = f.read()

# Conflict 1
c1 = """<<<<<<< HEAD
        unroll_4_static!(K_IDX, {
            let matches = const_eq_u32(k_masked as u32, K_IDX as u32);
            log_m = const_select_u32(matches, node_masses[K_IDX & 3][i & 7].log2().0 as u32, log_m);
=======
        unroll_4_static!(k_idx, {
            let matches = const_eq_u32(k_masked as u32, k_idx as u32);
            log_m = const_select_u32(matches, node_masses[k_idx & 3][i & 7].log2().val as u32, log_m);
>>>>>>> subagent-Numeric-Law-Architect-self-3795f08e"""
r1 = """        unroll_4_static!(K_IDX, {
            let matches = const_eq_u32(k_masked as u32, K_IDX as u32);
            log_m = const_select_u32(matches, node_masses[K_IDX & 3][i & 7].log2().val as u32, log_m);"""
content = content.replace(c1, r1)

# Conflict 2
c2 = """<<<<<<< HEAD
    unroll_8_static!(J, {
        let is_child = const_eq_u32(parent[J & 7] as u32, v as u32);
        let a_prime = x[J & 7].wrapping_sub(x_max_meas);
        let exp_val = SignedFixed(a_prime).exp2();
        sum_exp_meas += NonNegativeFixed(const_select_u32(is_child, exp_val.0, 0));
=======
    unroll_8_static!(j, {
        let is_child = const_eq_u32(parent[j & 7] as u32, v as u32);
        let a_prime = x[j & 7].wrapping_sub(x_max_meas);
        let exp_val = SignedFixed::from_bits(a_prime).exp2();
        sum_exp_meas += NonNegativeFixed::from_bits(const_select_u32(is_child, exp_val.val, 0));
>>>>>>> subagent-Numeric-Law-Architect-self-3795f08e"""
r2 = """    unroll_8_static!(J, {
        let is_child = const_eq_u32(parent[J & 7] as u32, v as u32);
        let a_prime = x[J & 7].wrapping_sub(x_max_meas);
        let exp_val = SignedFixed::from_bits(a_prime).exp2();
        sum_exp_meas += NonNegativeFixed::from_bits(const_select_u32(is_child, exp_val.val, 0));"""
content = content.replace(c2, r2)

# Conflict 3
c3 = """<<<<<<< HEAD
    unroll_8_static!(X_IDX, {
        let is_sub = is_subtree_leaf_v[X_IDX & 7];
        let a_prime = x[X_IDX & 7].wrapping_sub(x_max_leaf);
        let exp_val = SignedFixed(a_prime).exp2();
        sum_exp_leaf += NonNegativeFixed(const_select_u32(is_sub as u32, exp_val.0, 0));
=======
    unroll_8_static!(x_idx, {
        let is_sub = is_subtree_leaf_v[x_idx & 7];
        let a_prime = x[x_idx & 7].wrapping_sub(x_max_leaf);
        let exp_val = SignedFixed::from_bits(a_prime).exp2();
        sum_exp_leaf += NonNegativeFixed::from_bits(const_select_u32(is_sub as u32, exp_val.val, 0));
>>>>>>> subagent-Numeric-Law-Architect-self-3795f08e"""
r3 = """    unroll_8_static!(X_IDX, {
        let is_sub = is_subtree_leaf_v[X_IDX & 7];
        let a_prime = x[X_IDX & 7].wrapping_sub(x_max_leaf);
        let exp_val = SignedFixed::from_bits(a_prime).exp2();
        sum_exp_leaf += NonNegativeFixed::from_bits(const_select_u32(is_sub as u32, exp_val.val, 0));"""
content = content.replace(c3, r3)

# Conflict 4
c4 = """<<<<<<< HEAD
        unroll_8_static!(X_IDX, {
            let is_sub_c = is_subtree_leaf[c & 7][X_IDX & 7];
            let a_prime = x[X_IDX & 7].wrapping_sub(x_max_c);
            let exp_val = SignedFixed(a_prime).exp2();
            sum_exp_c += NonNegativeFixed(const_select_u32(is_sub_c as u32, exp_val.0, 0));
=======
        unroll_8_static!(x_idx, {
            let is_sub_c = is_subtree_leaf[c & 7][x_idx & 7];
            let a_prime = x[x_idx & 7].wrapping_sub(x_max_c);
            let exp_val = SignedFixed::from_bits(a_prime).exp2();
            sum_exp_c += NonNegativeFixed::from_bits(const_select_u32(is_sub_c as u32, exp_val.val, 0));
>>>>>>> subagent-Numeric-Law-Architect-self-3795f08e"""
r4 = """        unroll_8_static!(X_IDX, {
            let is_sub_c = is_subtree_leaf[c & 7][X_IDX & 7];
            let a_prime = x[X_IDX & 7].wrapping_sub(x_max_c);
            let exp_val = SignedFixed::from_bits(a_prime).exp2();
            sum_exp_c += NonNegativeFixed::from_bits(const_select_u32(is_sub_c as u32, exp_val.val, 0));"""
content = content.replace(c4, r4)

with open('crates/bcinr-cmca/src/observatory.rs', 'w') as f:
    f.write(content)

