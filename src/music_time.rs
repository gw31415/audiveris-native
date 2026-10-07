// SPDX-License-Identifier: AGPL-3.0-or-later
//! Reduced nonnegative Rational duration/getter arithmetic. Rhythm clocks
//! still use sixteenth ticks; full signed Rational/tuplet processing is pending.
#[derive(Clone, Copy)]
pub(crate) struct Fraction {
    numerator: u32,
    denominator: u32,
}
impl Fraction {
    pub(crate) fn new(mut numerator: u32, mut denominator: u32) -> Result<Self, String> {
        if denominator == 0 {
            return Err("music_time_positive_rational_required".into());
        }
        let (mut a, mut b) = (numerator, denominator);
        while b != 0 {
            (a, b) = (b, a % b);
        }
        numerator /= a;
        denominator /= a;
        Ok(Self {
            numerator,
            denominator,
        })
    }
    pub(crate) fn parse(text: &str) -> Result<Self, String> {
        let (n, d) = text.split_once('/').unwrap_or((text, "1"));
        let parse = |s: &str| {
            s.parse::<u32>()
                .map_err(|_| "music_time_positive_rational_required".to_owned())
        };
        Self::new(parse(n)?, parse(d)?)
    }
    pub(crate) fn times(self, other: Self) -> Result<Self, String> {
        Self::new(
            self.numerator
                .checked_mul(other.numerator)
                .ok_or("music_time_rational_overflow")?,
            self.denominator
                .checked_mul(other.denominator)
                .ok_or("music_time_rational_overflow")?,
        )
    }
    pub(crate) fn plus(self, other: Self) -> Result<Self, String> {
        Self::new(
            self.numerator
                .checked_mul(other.denominator)
                .and_then(|a| {
                    self.denominator
                        .checked_mul(other.numerator)
                        .and_then(|b| a.checked_add(b))
                })
                .ok_or("music_time_rational_overflow")?,
            self.denominator
                .checked_mul(other.denominator)
                .ok_or("music_time_rational_overflow")?,
        )
    }
    pub(crate) fn ticks(self) -> Result<u32, String> {
        if !16_u32.is_multiple_of(self.denominator) {
            return Err("music_time_general_rational_grid_pending".into());
        }
        self.numerator
            .checked_mul(16 / self.denominator)
            .ok_or_else(|| "music_time_tick_overflow".into())
    }
}
impl std::fmt::Display for Fraction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.denominator == 1 {
            write!(f, "{}", self.numerator)
        } else {
            write!(f, "{}/{}", self.numerator, self.denominator)
        }
    }
}
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
    Fraction::parse(text)?.ticks()
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
