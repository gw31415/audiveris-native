// Copyright (c) 1998, 2023, Oracle and/or its affiliates. All rights reserved.
// Rust translation: 2026 audiveris-native contributors.
// SPDX-License-Identifier: GPL-2.0-only WITH Classpath-exception-2.0
// Derived from java.lang.FdLibm.Hypot in the already-local OpenJDK source.
// The Classpath exception is retained for this modified independent library.
// Distributed WITHOUT ANY WARRANTY; see LICENSE, NOTICE, and upstream/FdLibm.java.

fn high(value: f64) -> i32 {
    (value.to_bits() >> 32) as i32
}
fn from_high(value: i32) -> f64 {
    f64::from_bits((value as u32 as u64) << 32)
}
fn power_of_two(exponent: i32) -> f64 {
    // All callers use normal powers of two, with exponents in [-1022, 1022].
    f64::from_bits(((exponent + 1023) as u64) << 52)
}

/// OpenJDK Math.hypot/StrictMath.hypot operation order, including fdlibm
/// high-word splitting. Platform hypot can differ by one ULP; preserving this
/// result matters when subsequent original geometry uses strict thresholds.
pub fn hypot(x: f64, y: f64) -> f64 {
    let mut a = x.abs();
    let mut b = y.abs();
    if !a.is_finite() || !b.is_finite() {
        return if a == f64::INFINITY || b == f64::INFINITY {
            f64::INFINITY
        } else {
            a + b
        };
    }
    if b > a {
        std::mem::swap(&mut a, &mut b);
    }
    let mut ha = high(a);
    let mut hb = high(b);
    if ha - hb > 0x03c0_0000 {
        return a + b;
    }
    let mut k = 0;
    if a > f64::from_bits(0x5f30_0000_ffff_ffff) {
        ha -= 0x2580_0000;
        hb -= 0x2580_0000;
        a *= power_of_two(-600);
        b *= power_of_two(-600);
        k += 600;
    }
    if b < power_of_two(-500) {
        if b < f64::MIN_POSITIVE {
            if b == 0.0 {
                return a;
            }
            let factor = power_of_two(1022);
            b *= factor;
            a *= factor;
            k -= 1022;
        } else {
            ha += 0x2580_0000;
            hb += 0x2580_0000;
            a *= power_of_two(600);
            b *= power_of_two(600);
            k -= 600;
        }
    }
    let w = a - b;
    let result = if w > b {
        let t1 = from_high(ha);
        let t2 = a - t1;
        (t1 * t1 - (b * (-b) - t2 * (a + t1))).sqrt()
    } else {
        a += a;
        let y1 = from_high(hb);
        let y2 = b - y1;
        let t1 = from_high(ha + 0x0010_0000);
        let t2 = a - t1;
        (t1 * y1 - (w * (-w) - (t1 * y2 + t2 * b))).sqrt()
    };
    if k != 0 {
        power_of_two(k) * result
    } else {
        result
    }
}

#[cfg(test)]
mod tests {
    use super::hypot;

    #[test]
    fn matches_original_java_hypot_vectors() {
        let input = include_str!("../upstream/hypot-vectors.txt");
        let mut count = 0;
        for line in input.lines() {
            let bits: Vec<u64> = line
                .split_whitespace()
                .map(|word| u64::from_str_radix(word, 16).unwrap())
                .collect();
            assert_eq!(bits.len(), 3);
            let actual = hypot(f64::from_bits(bits[0]), f64::from_bits(bits[1]));
            let expected = f64::from_bits(bits[2]);
            if expected.is_nan() {
                assert!(actual.is_nan(), "{line}");
            } else {
                assert_eq!(actual.to_bits(), expected.to_bits(), "{line}");
            }
            count += 1;
        }
        assert_eq!(count, 5321);
    }
}
