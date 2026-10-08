// SPDX-License-Identifier: AGPL-3.0-or-later
/// Measured sheet geometry carried with the live image owner, independently
/// of its mutable observation report.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct SheetSkew {
    slope: f64,
}
impl SheetSkew {
    pub(crate) fn new(slope: f64) -> Result<Self, String> {
        if !slope.is_finite() {
            return Err("music_measured_sheet_skew_required".into());
        }
        Ok(Self { slope })
    }
    pub(crate) fn slope(self) -> f64 {
        self.slope
    }
    pub(crate) fn vertical(self, point: [f64; 2]) -> [f64; 4] {
        [
            point[0],
            point[1],
            point[0] - 1000. * self.slope,
            point[1] + 1000.,
        ]
    }
}
