//! Physical-memory-equivalent preferences translated into exact provider integers.
//! This display convention does not establish a physical quota or reservation.
use hyper_gpu_support::model::Allocation;

const BYTES_PER_GIB: u64 = 1 << 30;
const STEPS_PER_GIB: u64 = 10;
const MAX_EXACT_STEP: u64 = 1 << 24;

pub(super) struct MemorySlider {
    pub minimum: i32,
    pub maximum: i32,
    pub position: i32,
    pub label: String,
    pub range_start: String,
    pub range_end: String,
    bounds: Allocation,
    limits: Allocation,
    physical_bytes: u64,
    provider_total: u64,
}

/// Floating point is used only for labels; selection and persisted values are integers.
pub(super) fn format_gib(bytes: u64) -> String {
    let value = bytes as f64 / BYTES_PER_GIB as f64;
    let digits = format!("{value:.2}");
    format!("{} GiB", digits.trim_end_matches('0').trim_end_matches('.'))
}

/// A proportional display equivalent, not a claim about driver memory enforcement.
pub(super) fn equivalent(raw: u64, physical: u64, total: u64) -> Option<u64> {
    if total == 0 || physical == 0 || raw > total {
        return None;
    }
    u64::try_from(u128::from(raw) * u128::from(physical) / u128::from(total)).ok()
}

/// Incomplete/invalid raw edits are never replaced by moving the slider.
pub(super) fn request(values: &[String]) -> Result<Option<Allocation>, String> {
    if values.len() != 3 {
        return Err("VRAM fields are unavailable.".into());
    }
    if values.iter().all(|v| v.trim().is_empty()) {
        return Ok(None);
    }
    let parsed = values
        .iter()
        .map(|v| v.trim().parse::<u64>())
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| "Complete or clear all three VRAM fields to use the slider.")?;
    Ok(Some(Allocation {
        minimum: parsed[0],
        optimal: parsed[1],
        maximum: parsed[2],
    }))
}

impl MemorySlider {
    /// Uses positive independent physical/provider totals and validated provider bounds.
    pub fn scaled(
        limits: &Allocation,
        current: Option<&Allocation>,
        physical_bytes: u64,
        provider_total: u64,
    ) -> Result<Self, String> {
        limits.validate(limits)?;
        if physical_bytes == 0 || provider_total == 0 || limits.maximum > provider_total {
            return Err("Physical memory and provider allocation scale are unavailable or inconsistent. Use Advanced Allocation.".into());
        }
        // Bound precision before the products below, which also keeps u128 products safe.
        if (u128::from(physical_bytes) * u128::from(STEPS_PER_GIB))
            .div_ceil(u128::from(BYTES_PER_GIB))
            > u128::from(MAX_EXACT_STEP)
        {
            return Err(
                "VRAM bounds exceed exact slider precision. Use Advanced Allocation.".into(),
            );
        }
        let bounds = current.unwrap_or(limits);
        bounds
            .validate(limits)
            .map_err(|_| "Correct the VRAM range in Advanced Allocation to use the slider.")?;
        let numerator =
            |raw: u64| u128::from(raw) * u128::from(physical_bytes) * u128::from(STEPS_PER_GIB);
        let denominator = u128::from(provider_total) * u128::from(BYTES_PER_GIB);
        let lower = numerator(limits.minimum).div_ceil(denominator) as u64;
        let upper = numerator(limits.maximum).div_ceil(denominator) as u64;
        if lower >= upper {
            return Err(
                "This VRAM range has no usable slider interval. Use Advanced Allocation.".into(),
            );
        }
        let display =
            |raw| format_gib(equivalent(raw, physical_bytes, provider_total).unwrap_or(0));
        Ok(Self {
            minimum: lower as i32,
            maximum: upper as i32,
            position: if bounds.optimal == limits.maximum {
                upper
            } else {
                ((numerator(bounds.optimal) / denominator) as u64).clamp(lower, upper)
            } as i32,
            label: current.map_or_else(
                || format!("{} (default preference)", display(limits.optimal)),
                |v| display(v.optimal),
            ),
            range_start: display(limits.minimum),
            range_end: display(limits.maximum),
            bounds: bounds.clone(),
            limits: limits.clone(),
            physical_bytes,
            provider_total,
        })
    }

    pub fn select(&self, position: i32) -> Result<Allocation, String> {
        if !(self.minimum..=self.maximum).contains(&position) {
            return Err("The selected memory value is outside the current VRAM range.".into());
        }
        // The last tick is the exact provider endpoint, including fractional capacity.
        let optimal = if position == self.maximum {
            self.limits.maximum
        } else {
            let numerator = u128::from(position as u32)
                * u128::from(BYTES_PER_GIB)
                * u128::from(self.provider_total);
            let denominator = u128::from(STEPS_PER_GIB) * u128::from(self.physical_bytes);
            u64::try_from(numerator.div_ceil(denominator))
                .map_err(|_| "Memory conversion overflow")?
                .clamp(self.limits.minimum, self.limits.maximum)
        };
        let result = Allocation {
            minimum: self.bounds.minimum.min(optimal),
            optimal,
            maximum: self.bounds.maximum.max(optimal),
        };
        result.validate(&self.limits)?;
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn limits() -> Allocation {
        Allocation {
            minimum: 0,
            optimal: 1_000_000_000,
            maximum: 1_000_000_000,
        }
    }
    #[test]
    fn eight_and_sixteen_gib_cards_use_their_own_capacity_and_exact_raw_values() {
        for gib in [8, 16] {
            let slider = MemorySlider::scaled(&limits(), None, gib << 30, 1_000_000_000).unwrap();
            assert_eq!((slider.minimum, slider.maximum), (0, gib as i32 * 10));
            assert_eq!(slider.range_end, format!("{gib} GiB"));
            assert_eq!(
                slider.select(slider.maximum).unwrap().optimal,
                1_000_000_000
            );
            assert_eq!(
                slider.select(slider.maximum / 2).unwrap().optimal,
                500_000_000
            );
            assert_eq!(slider.select(0).unwrap().optimal, 0);
            assert!(slider.select(-1).is_err());
            assert!(slider.select(slider.maximum + 1).is_err());
        }
    }
    #[test]
    fn raw_bounds_survive_and_expand_only_for_explicit_selections() {
        let current = Allocation {
            minimum: 250_000_001,
            optimal: 500_000_001,
            maximum: 750_000_001,
        };
        let slider =
            MemorySlider::scaled(&limits(), Some(&current), 8 << 30, 1_000_000_000).unwrap();
        assert_eq!(slider.label, "4 GiB");
        let selected = slider.select(50).unwrap();
        assert_eq!(selected.optimal, 625_000_000);
        assert_eq!(selected.minimum, current.minimum);
        assert_eq!(selected.maximum, current.maximum);
        assert_eq!(slider.select(80).unwrap().maximum, limits().maximum);
        assert_eq!(slider.select(0).unwrap().minimum, 0);
        let reread =
            MemorySlider::scaled(&limits(), Some(&selected), 8 << 30, 1_000_000_000).unwrap();
        assert_eq!(reread.position, 50);
    }
    #[test]
    fn fractional_capacity_has_exact_maximum_endpoint() {
        let physical = (8 << 30) - 40_000_000;
        let slider = MemorySlider::scaled(&limits(), None, physical, 1_000_000_000).unwrap();
        assert_eq!(slider.maximum, 80);
        assert_eq!(slider.select(80).unwrap().optimal, limits().maximum);
        assert_eq!(
            equivalent(1_000_000_000, physical, 1_000_000_000),
            Some(physical)
        );
    }
    #[test]
    fn invalid_unknown_and_imprecise_scales_are_rejected_without_inventing_intent() {
        assert!(MemorySlider::scaled(&limits(), None, 0, 1_000_000_000).is_err());
        assert!(MemorySlider::scaled(&limits(), None, 8 << 30, 0).is_err());
        assert!(MemorySlider::scaled(&limits(), None, 8 << 30, 999_999_999).is_err());
        assert!(MemorySlider::scaled(&limits(), None, u64::MAX, 1_000_000_000).is_err());
        let invalid = Allocation {
            optimal: 1_000_000_001,
            ..limits()
        };
        assert!(MemorySlider::scaled(&limits(), Some(&invalid), 8 << 30, 1_000_000_000).is_err());
        assert_eq!(request(&vec![String::new(); 3]).unwrap(), None);
        assert!(request(&["1".into(), "".into(), "8".into()]).is_err());
        assert_eq!(equivalent(u64::MAX, 8 << 30, 1), None);
    }
}
