pub fn collatz(mut n: u32) -> Option<u32> {
    let mut i = 0;
    if n == 0 {
        return None;
    }
    while n != 1 {
        n = if n % 2 == 0 {
            n / 2
        } else {
            // Overflow? (i.e. 3*n + 1 > 0xffff_ffff)
            if n >= 0x5555_5555 {
                return None;
            }
            // TODO: Use this instead when/if checked add/mul can work: n.checked_mul(3)?.checked_add(1)?
            3 * n + 1
        };
        i += 1;
    }
    Some(i)
}

// Work around https://github.com/EmbarkStudios/rust-gpu/issues/677
pub fn unwrap_or_max(option: Option<u32>) -> u32 {
    match option {
        Some(inner) => inner,
        None => u32::MAX,
    }
}
