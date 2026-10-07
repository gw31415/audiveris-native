// SPDX-License-Identifier: AGPL-3.0-or-later
//! Exact nonnegative ordinary clocks/durations and signed Voice termination.
//! The image mapper retains its sixteenth boundary; general tuplets are pending.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Fraction {
    numerator: u32,
    denominator: u32,
}
impl Fraction {
    pub(crate) const ZERO: Self = Self {
        numerator: 0,
        denominator: 1,
    };
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
    fn from_wide(mut numerator: u64, mut denominator: u64) -> Result<Self, String> {
        let (mut a, mut b) = (numerator, denominator);
        while b != 0 {
            (a, b) = (b, a % b);
        }
        numerator /= a;
        denominator /= a;
        Self::new(
            u32::try_from(numerator).map_err(|_| "music_time_rational_overflow")?,
            u32::try_from(denominator).map_err(|_| "music_time_rational_overflow")?,
        )
    }
    pub(crate) fn times(self, other: Self) -> Result<Self, String> {
        Self::from_wide(
            u64::from(self.numerator) * u64::from(other.numerator),
            u64::from(self.denominator) * u64::from(other.denominator),
        )
    }
    pub(crate) fn plus(self, other: Self) -> Result<Self, String> {
        Self::from_wide(
            (u64::from(self.numerator) * u64::from(other.denominator))
                .checked_add(u64::from(self.denominator) * u64::from(other.numerator))
                .ok_or("music_time_rational_overflow")?,
            u64::from(self.denominator) * u64::from(other.denominator),
        )
    }
    pub(crate) fn difference(self, other: Self) -> Result<Delta, String> {
        let a = u64::from(self.numerator) * u64::from(other.denominator);
        let b = u64::from(other.numerator) * u64::from(self.denominator);
        Ok(Delta {
            negative: a < b,
            magnitude: Self::from_wide(
                a.abs_diff(b),
                u64::from(self.denominator) * u64::from(other.denominator),
            )?,
        })
    }
    pub(crate) fn denominator(self) -> u32 {
        self.denominator
    }
    pub(crate) fn units(self, per_whole: u32) -> Result<u32, String> {
        let scaled = u64::from(self.numerator) * u64::from(per_whole);
        if per_whole == 0 || !scaled.is_multiple_of(u64::from(self.denominator)) {
            return Err("musicxml_exact_positive_division_required".into());
        }
        u32::try_from(scaled / u64::from(self.denominator))
            .map_err(|_| "music_time_rational_overflow".into())
    }
    pub(crate) fn reported_units(self, per_whole: u32) -> serde_json::Value {
        let scaled = u64::from(self.numerator) * u64::from(per_whole);
        if scaled.is_multiple_of(u64::from(self.denominator)) {
            serde_json::json!(scaled / u64::from(self.denominator))
        } else {
            serde_json::json!(scaled as f64 / f64::from(self.denominator))
        }
    }
    pub(crate) fn from_ticks(ticks: u32) -> Self {
        Self::new(ticks, 16).expect("nonzero constant denominator")
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
impl Ord for Fraction {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (u64::from(self.numerator) * u64::from(other.denominator))
            .cmp(&(u64::from(other.numerator) * u64::from(self.denominator)))
    }
}
impl PartialOrd for Fraction {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl<'de> serde::Deserialize<'de> for Fraction {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = <String as serde::Deserialize>::deserialize(deserializer)?;
        Self::parse(&text).map_err(serde::de::Error::custom)
    }
}
impl serde::Serialize for Fraction {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}
#[derive(Clone, Copy)]
pub(crate) struct Delta {
    negative: bool,
    magnitude: Fraction,
}
impl Delta {
    pub(crate) fn excess(self) -> Option<Fraction> {
        (!self.negative && self.magnitude != Fraction::ZERO).then_some(self.magnitude)
    }
}
impl std::fmt::Display for Delta {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.negative {
            f.write_str("-")?;
        }
        std::fmt::Display::fmt(&self.magnitude, f)
    }
}
pub(crate) fn format_fraction(value: Fraction) -> String {
    value.to_string()
}
pub(crate) fn format_delta(value: Delta) -> String {
    value.to_string()
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

#[cfg(test)]
pub(crate) fn parse_ticks(text: &str) -> Result<u32, String> {
    Fraction::parse(text)?.ticks()
}
