// Copyright © Audiveris 2026. SPDX-License-Identifier: AGPL-3.0-or-later
// KeyBuilder.retrieveHiLoPeaks and bounded HiLoPeakFinder derivative hysteresis.
// Pinned upstream 9e1e55cd2746037d059345881c53e6a6754bffbd.
use crate::key_preparation::Projection;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Peak {
    pub min: usize,
    pub main: usize,
    pub max: usize,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Parameters {
    pub min_value: i32,
    pub min_derivative: i32,
    pub min_gain_ratio: f64,
    pub quorum: i32,
}
impl Parameters {
    pub fn for_interline(interline: usize) -> Result<Self, String> {
        if !(1..=128).contains(&interline) {
            return Err("unsupported_key_peak_interline".into());
        }
        let il = interline as f64;
        Ok(Self {
            min_value: (il * 0.4).round_ties_even() as i32 + 1,
            min_derivative: (il * 0.35).round_ties_even() as i32,
            min_gain_ratio: 0.2,
            quorum: (il * 1.25).round_ties_even() as i32,
        })
    }
}
#[derive(Clone, Copy)]
struct DerivativePeak {
    min: usize,
    max: usize,
    finished: bool,
}

/// Values have an absolute abscissa `function_start`; domain is inclusive.
/// Mirrors original terminal behavior: an unfinished terminal HiLo is not flushed.
/// Returns peaks by decreasing main value; equal values retain original HiLo order.
pub fn find(
    values: &[i32],
    function_start: usize,
    domain: [usize; 2],
    params: Parameters,
) -> Result<Vec<Peak>, String> {
    if values.is_empty()
        || values.len() > 8192
        || values.iter().any(|v| !(0..=32767).contains(v))
        || function_start > 4_000_000
        || domain[0] > domain[1]
        || domain[0] < function_start
        || domain[1] >= function_start + values.len()
    {
        return Err("unsupported_key_peak_domain".into());
    }
    if params.min_value < 0
        || params.min_derivative < 0
        || params.quorum < 0
        || !params.min_gain_ratio.is_finite()
        || params.min_gain_ratio <= 0.
        || params.min_gain_ratio > 1.
    {
        return Err("unsupported_key_peak_parameters".into());
    }
    let value = |x: usize| values[x - function_start];
    let mut hilos = Vec::new();
    let (mut hi, mut lo): (Option<DerivativePeak>, Option<DerivativePeak>) = (None, None);
    let make = |hi: DerivativePeak, lo: DerivativePeak| {
        let mut main = hi.min;
        for x in hi.min..=lo.max {
            if value(x) > value(main) {
                main = x;
            }
        }
        Peak {
            min: hi.min,
            main,
            max: lo.max,
        }
    };
    for x in domain[0] + 1..=domain[1] {
        let der = value(x) - value(x - 1);
        if der >= params.min_derivative {
            if let Some(l) = lo.take() {
                hilos.push(make(hi.take().unwrap(), l));
            }
            if hi.is_none_or(|h| h.finished) {
                hi = Some(DerivativePeak {
                    min: x,
                    max: x,
                    finished: false,
                });
            } else {
                hi.as_mut().unwrap().max = x;
            }
        } else if der <= -params.min_derivative {
            if let Some(l) = lo.as_mut() {
                l.max = x;
            } else if hi.is_some() {
                lo = Some(DerivativePeak {
                    min: x,
                    max: x,
                    finished: false,
                });
            }
        } else if let Some(l) = lo.take() {
            hilos.push(make(hi.take().unwrap(), l));
        } else if let Some(h) = hi.as_mut() {
            if value(x) < params.min_value {
                hi = None;
            } else {
                h.finished = true;
            }
        }
    }
    let mut order = (0..hilos.len()).collect::<Vec<_>>();
    order.sort_by_key(|i| std::cmp::Reverse(value(hilos[*i].main)));
    let mut associated = vec![None::<Peak>; hilos.len()];
    let mut peaks = Vec::new();
    for i in order {
        let hilo = hilos[i];
        let mut pmin = (hilo.min - 1).max(1);
        if i > 0 {
            pmin = pmin.max(associated[i - 1].map_or(hilos[i - 1].max, |p| p.max) + 1);
        }
        let (mut lower, mut upper, mut total) = (hilo.main, hilo.main, value(hilo.main));
        loop {
            let before = if lower == pmin { 0 } else { value(lower - 1) };
            let after = if upper == hilo.max {
                0
            } else {
                value(upper + 1)
            };
            let gain = before.max(after);
            // Java's 0/0 is NaN and the >= comparison is false.
            if total + gain == 0 || gain as f64 / ((total + gain) as f64) < params.min_gain_ratio {
                break;
            }
            if before > after {
                lower -= 1;
            } else {
                upper += 1;
            }
            total += gain;
        }
        let peak = Peak {
            min: lower,
            main: hilo.main,
            max: upper,
        };
        associated[i] = Some(peak);
        peaks.push(peak);
    }
    peaks.sort_by_key(|p| std::cmp::Reverse(value(p.main)));
    peaks.retain(|p| value(p.main) >= params.quorum);
    Ok(peaks)
}

pub fn retrieve(projection: &Projection, interline: usize) -> Result<Vec<Peak>, String> {
    let params = Parameters::for_interline(interline)?;
    let start =
        usize::try_from(projection.projection_start).map_err(|_| "invalid_key_peak_start")?;
    let domain = [
        usize::try_from(projection.browse_start).map_err(|_| "invalid_key_peak_start")?,
        usize::try_from(projection.browse_stop).map_err(|_| "invalid_key_peak_stop")?,
    ];
    let mut peaks = find(&projection.values, start, domain, params)?;
    peaks.sort_by_key(|p| p.main);
    Ok(peaks)
}

/// Native projection JSON adapter. This stage does not infer sharp/flat slices or a key signature.
pub fn analyze(preparation: &Value, interline: usize) -> Result<Value, String> {
    let mut staffs = Vec::new();
    for row in preparation["staffs"]
        .as_array()
        .ok_or("missing_key_projections")?
    {
        let values = row["projection"]["values"]
            .as_array()
            .ok_or("missing_key_projection_values")?
            .iter()
            .map(|v| {
                v.as_i64()
                    .and_then(|v| i32::try_from(v).ok())
                    .ok_or("invalid_key_projection_value")
            })
            .collect::<Result<Vec<_>, _>>()?;
        let start = row["projection"]["x_min"]
            .as_u64()
            .and_then(|v| usize::try_from(v).ok())
            .ok_or("invalid_key_peak_start")?;
        let domain = [
            row["browse_start"]
                .as_u64()
                .and_then(|v| usize::try_from(v).ok())
                .ok_or("invalid_key_peak_start")?,
            row["browse_stop"]
                .as_u64()
                .and_then(|v| usize::try_from(v).ok())
                .ok_or("invalid_key_peak_stop")?,
        ];
        let specific = match row.get("specific_interline") {
            None => interline,
            Some(value) => value
                .as_u64()
                .filter(|n| (1..=128).contains(n))
                .ok_or("unsupported_key_peak_interline")? as usize,
        };
        let parameters = Parameters::for_interline(specific)?;
        let mut peaks = find(&values, start, domain, parameters)?;
        peaks.sort_by_key(|p| p.main);
        staffs.push(json!({"staff_id":row["staff_id"],"parameters":parameters,"peaks":peaks}));
    }
    Ok(
        json!({"stage":"HEADERS.key_hilo_peaks","staffs":staffs,"key_recognition_complete":false,"headers_complete":false,"pending":["sharp/flat peak filtering and slice construction","alteration glyph extraction and classification","clef/key consistency and cross-staff key selection"]}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    fn p() -> Parameters {
        Parameters {
            min_value: 1,
            min_derivative: 2,
            min_gain_ratio: 0.2,
            quorum: 0,
        }
    }
    #[test]
    fn terminal_hilo_is_not_flushed_and_absolute_domain_is_preserved() {
        assert!(
            find(&[0, 0, 8, 0], 100, [100, 103], p())
                .unwrap()
                .is_empty()
        );
        assert_eq!(
            find(&[0, 0, 8, 0, 0], 100, [100, 104], p()).unwrap(),
            vec![Peak {
                min: 102,
                main: 102,
                max: 102
            }]
        );
        assert!(
            find(&[0, 0, 8, 0, 0], 100, [102, 104], p())
                .unwrap()
                .is_empty()
        );
    }
    #[test]
    fn gain_equal_threshold_expands_and_quorum_is_inclusive() {
        let v = [0, 0, 8, 2, 0, 0];
        let mut params = p();
        params.quorum = 8;
        assert_eq!(
            find(&v, 0, [0, 5], params).unwrap(),
            vec![Peak {
                min: 2,
                main: 2,
                max: 3
            }]
        );
        params.quorum = 9;
        assert!(find(&v, 0, [0, 5], params).unwrap().is_empty());
    }
    #[test]
    fn tied_peak_heights_keep_abscissa_order_and_bad_domains_are_rejected() {
        let v = [0, 0, 8, 0, 0, 8, 0, 0];
        let peaks = find(&v, 0, [0, 7], p()).unwrap();
        assert_eq!(peaks.iter().map(|p| p.main).collect::<Vec<_>>(), vec![2, 5]);
        assert!(find(&v, 0, [0, 8], p()).is_err());
        let mut params = p();
        params.min_gain_ratio = 0.;
        assert!(find(&v, 0, [0, 7], params).is_err());
    }
}
