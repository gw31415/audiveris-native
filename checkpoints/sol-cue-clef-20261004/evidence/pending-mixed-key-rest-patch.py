from pathlib import Path
r=Path('/workspace/onboarding/artifacts/audiveris-source')
p=r/'src/key_preparation.rs';s=p.read_text();s=s.replace('''        if specific != sheet_interline as u64 {''','''        if specific != sheet_interline as u64
            && !(specific > 0 && specific < sheet_interline as u64 && staff["small"] == true) {''',1)
s=s.replace('''        }));
    }
    Ok(''','''        }));
        // Carry staff scale only for mixed rows; preserve uniform-page JSON.
        if input.specific_interline != sheet_interline {
            output.last_mut().unwrap()["specific_interline"] = json!(input.specific_interline);
        }
    }
    Ok(''',1);p.write_text(s)
p=r/'src/key_peaks.rs';s=p.read_text();s=s.replace('''        let parameters = Parameters::for_interline(interline)?;''','''        let specific = match row.get("specific_interline") {
            None => interline,
            Some(value) => value.as_u64().filter(|n| (1..=128).contains(n))
                .ok_or("unsupported_key_peak_interline")? as usize,
        };
        let parameters = Parameters::for_interline(specific)?;''',1);p.write_text(s)
p=r/'src/native_rest_mapping.rs';s=p.read_text();needle='''    fn from_native_grid(
'''
helper='''    /// Native page adapter with the actual sheet main scale. Staff selection
    /// uses page-scale areas; pitch/serif geometry retains each staff scale.
    /// The legacy from_grid contract still refuses unknown mixed-scale pages.
    pub fn from_grid_with_main(
        grid: &Value, width: usize, height: usize, system: u64, main: usize,
    ) -> Result<Self, String> {
        if !(1..=128).contains(&main) { return Err("invalid_rest_native_interline".into()); }
        if grid["schema"] != "audiveris-native.grid-ir.v1" {
            return Self::from_grid(grid, width, height, system);
        }
        let staffs = grid["staffs"].as_array().filter(|s| !s.is_empty())
            .ok_or("unsupported_rest_staff_count")?;
        for s in staffs {
            let specific = s["specific_interline"].as_u64()
                .filter(|n| (1..=128).contains(n)).ok_or("invalid_rest_native_interline")?;
            if specific != main as u64
                && !(specific < main as u64 && s["small"] == true) {
                return Err("unsupported_rest_native_mixed_interline".into());
            }
        }
        Self::from_native_grid_at_scale(grid, width, height, system, main)
    }
'''
assert needle in s;s=s.replace(needle,helper+needle,1)
old='''        if interline != Some((main as f64 * 0.9).round_ties_even()) {'''
new='''        Self::from_native_grid_at_scale(grid, width, height, system, main)
    }
    fn from_native_grid_at_scale(
        grid: &Value, width: usize, height: usize, system: u64, main: usize,
    ) -> Result<Self, String> {
        let interline = grid["systems"].as_array().and_then(|s| s.first())
            .and_then(|s| s["area"]["vertical_margin"].as_f64());
        if interline != Some((main as f64 * 0.9).round_ties_even()) {'''
assert old in s;s=s.replace(old,new,1)
# Move margin extraction to the shared explicit-scale adapter.
first='''        let interline = grid["systems"]
            .as_array()
            .and_then(|s| s.first())
            .and_then(|s| s["area"]["vertical_margin"].as_f64());
        // Native GRID writes the page scale's main interline on every staff.'''
assert first in s;s=s.replace(first,'''        // Legacy uniform-scale contract infers main from the first staff.''',1);p.write_text(s)
p=r/'src/beams_pipeline.rs';s=p.read_text();s=s.replace('SystemMapping::from_grid(', 'SystemMapping::from_grid_with_main(',1).replace('''                owner.system_id,
            )?;''','''                owner.system_id,
                scale.interline.main,
            )?;''',1);p.write_text(s)
