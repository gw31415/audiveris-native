// SPDX-License-Identifier: AGPL-3.0-or-later
//! Exact duration codec for the currently admitted binary rhythm grid. This
//! retains the existing sixteenth tick policy, not a full Rational/tuplet port.
// Exact sixteenth-of-whole ticks; floats are only outward report quarter values.
pub(crate) fn format_ticks(ticks: u32) -> String {
    let mut gcd = 16;
    while !ticks.is_multiple_of(gcd) {
        gcd /= 2;
    }
    if gcd == 16 {
        format!("{}", ticks / 16)
    } else {
        format!("{}/{}", ticks / gcd, 16 / gcd)
    }
}

pub(crate) fn parse_ticks(text: &str) -> Result<u32, String> {
    let (numerator, denominator) = text.split_once('/').unwrap_or((text, "1"));
    let mut n = numerator
        .parse::<u32>()
        .map_err(|_| "music_time_positive_rational_required")?;
    let mut d = denominator
        .parse::<u32>()
        .map_err(|_| "music_time_positive_rational_required")?;
    if d == 0 {
        return Err("music_time_positive_rational_required".into());
    }
    let mut a = n;
    let mut b = d;
    while b != 0 {
        let r = a % b;
        a = b;
        b = r;
    }
    n /= a;
    d /= a;
    if !16_u32.is_multiple_of(d) {
        return Err("music_time_general_rational_grid_pending".into());
    }
    n.checked_mul(16 / d)
        .ok_or_else(|| "music_time_tick_overflow".into())
}

/// Difference of two u32 tick clocks is signed and has u32 magnitude.
pub(crate) fn format_delta_ticks(delta: i64) -> String {
    let value =
        format_ticks(u32::try_from(delta.unsigned_abs()).expect("difference of u32 clocks"));
    if delta < 0 {
        format!("-{value}")
    } else {
        value
    }
}
