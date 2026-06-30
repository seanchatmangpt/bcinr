use bcinr_core::logic::dfa::{dfa_is_accepting, dfa_run};
use bcinr_core::logic::swar_str::{count_byte_in_slice, find_first_byte_in_slice, is_all_ascii};

/// 2-state DFA over a 4-symbol alphabet: stays in state 0 unless it sees
/// symbol 3, which moves it to (accepting) state 1.
const ALPHABET_SIZE: usize = 4;
const TABLE: [usize; 8] = [0, 0, 0, 1, 1, 1, 1, 1];
const ACCEPT_STATES: [usize; 1] = [1];

fn sample_input(n: usize) -> Vec<u8> {
    (0..n).map(|i| (i % ALPHABET_SIZE) as u8).collect()
}

#[divan::bench]
fn dfa_run_avg() {
    let input = sample_input(256);
    divan::black_box(dfa_run(
        divan::black_box(&TABLE),
        divan::black_box(ALPHABET_SIZE),
        divan::black_box(0),
        divan::black_box(&input),
    ));
}

#[divan::bench]
fn dfa_is_accepting_avg() {
    divan::black_box(dfa_is_accepting(divan::black_box(1), divan::black_box(&ACCEPT_STATES)));
}

#[divan::bench]
fn count_byte_in_slice_avg() {
    let text = "the quick brown fox jumps over the lazy dog ".repeat(8);
    divan::black_box(count_byte_in_slice(divan::black_box(text.as_bytes()), divan::black_box(b'o')));
}

#[divan::bench]
fn find_first_byte_in_slice_avg() {
    let text = "the quick brown fox jumps over the lazy dog ".repeat(8);
    divan::black_box(find_first_byte_in_slice(divan::black_box(text.as_bytes()), divan::black_box(b'z')));
}

#[divan::bench]
fn is_all_ascii_avg() {
    let text = "the quick brown fox jumps over the lazy dog ".repeat(8);
    divan::black_box(is_all_ascii(divan::black_box(text.as_bytes())));
}

fn main() {
    divan::main();
}
