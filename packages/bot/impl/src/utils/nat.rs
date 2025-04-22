use candid::Nat;

pub fn nat_to_u128(
    value: Nat
) -> u128 {
    value.0.iter_u64_digits()
        .rev()
        .take(2)
        .fold(0u128, |acc, n| (acc << 64) | n as u128)
}