#![allow(
    dead_code,
    clippy::similar_names,
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::upper_case_acronyms,
    clippy::unwrap_used,
    clippy::large_stack_arrays,
    clippy::trivially_copy_pass_by_ref,
    clippy::recursive_format_impl,
    clippy::to_string_in_format_args,
    clippy::match_same_arms,
    clippy::default_trait_access,
    clippy::inline_always,
    clippy::unnecessary_wraps,
    clippy::unused_self,
    clippy::let_unit_value,
    clippy::absurd_extreme_comparisons,
    clippy::get_first,
    clippy::type_complexity,
    clippy::manual_range_contains,
    clippy::string_extend_chars,
    clippy::boxed_local,
    clippy::large_types_passed_by_value,
    clippy::wrong_self_convention,
    clippy::mutable_key_type,
    clippy::only_used_in_recursion,
    clippy::vec_init_then_push,
    clippy::needless_pass_by_value
)]

pub mod petri;
pub mod powl;
pub mod wasm;
pub mod yawl;
