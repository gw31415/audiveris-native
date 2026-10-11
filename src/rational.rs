//! Minimal exact fraction arithmetic for durations (in whole notes).

use std::cmp::Ordering;
use std::ops::{Add, Mul, Sub};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize)]
pub struct Q {
    pub n: i64,
    pub d: i64,
}

pub fn gcd(a: i64, b: i64) -> i64 {
    if b == 0 { a.abs() } else { gcd(b, a % b) }
}

pub fn lcm(a: i64, b: i64) -> i64 {
    a / gcd(a, b) * b
}

impl Q {
    pub const ZERO: Q = Q { n: 0, d: 1 };
    pub fn new(n: i64, d: i64) -> Q {
        let g = gcd(n, d).max(1);
        let s = if d < 0 { -1 } else { 1 };
        Q {
            n: s * n / g,
            d: s * d / g,
        }
    }
}

impl Add for Q {
    type Output = Q;
    fn add(self, o: Q) -> Q {
        Q::new(self.n * o.d + o.n * self.d, self.d * o.d)
    }
}
impl Sub for Q {
    type Output = Q;
    fn sub(self, o: Q) -> Q {
        Q::new(self.n * o.d - o.n * self.d, self.d * o.d)
    }
}
impl Mul for Q {
    type Output = Q;
    fn mul(self, o: Q) -> Q {
        Q::new(self.n * o.n, self.d * o.d)
    }
}
impl PartialOrd for Q {
    fn partial_cmp(&self, o: &Q) -> Option<Ordering> {
        Some(self.cmp(o))
    }
}
impl Ord for Q {
    fn cmp(&self, o: &Q) -> Ordering {
        (self.n * o.d).cmp(&(o.n * self.d))
    }
}
