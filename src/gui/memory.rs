//! Exact display conversion for the native Hyper-V VRAM byte contract.
//! This requests values; it makes no physical reservation/enforcement claim.
use hyper_gpu_support::model::Allocation;

const BYTES_PER_GB: u64 = 1_000_000_000;
const BYTES_PER_STEP: u64 = BYTES_PER_GB / 10;
// Every selectable integer must survive Slint's f32 Slider representation.
const MAX_EXACT_STEP: u64 = 1 << 24;

pub(super) struct MemorySlider {
    pub minimum: i32,
    pub maximum: i32,
    pub position: i32,
    pub label: String,
    pub range_start: String,
    pub range_end: String,
    pub range_note: Option<String>,
    bounds: Allocation,
    limits: Allocation,
    physical_capacity: u64,
}

pub(super) fn format_gb(bytes: u64) -> String {
    let whole = bytes / BYTES_PER_GB;
    let fraction = bytes % BYTES_PER_GB;
    if fraction == 0 {
        format!("{whole} GB")
    } else {
        let digits = format!("{fraction:09}");
        format!("{whole}.{} GB", digits.trim_end_matches('0'))
    }
}

/// Physical capacity is displayed separately from the provider's byte request.
pub(super) fn format_gib(bytes: u64) -> String {
    format!("{:.2} GiB", bytes as f64 / (1u64 << 30) as f64)
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
    /// Call only for native VRAM bytes, never arbitrary historical provider units.
    pub fn new(
        limits: &Allocation,
        current: Option<&Allocation>,
        physical_capacity: u64,
    ) -> Result<Self, String> {
        limits.validate(limits)?;
        if physical_capacity < BYTES_PER_GB {
            return Err("Detected physical VRAM is below the slider's 1 GB minimum. GPU memory selection is unavailable.".into());
        }
        let bounds = current.unwrap_or(limits);
        bounds
            .validate(limits)
            .map_err(|_| "Correct the VRAM range in Advanced Allocation to use the slider.")?;
        // Product range and provider admission are separate. The driver can reject
        // points in the displayed physical range; never scale its limits to the card.
        let lower = BYTES_PER_GB / BYTES_PER_STEP;
        let upper = physical_capacity.div_ceil(BYTES_PER_STEP);
        if bounds.optimal > physical_capacity {
            return Err("The current request exceeds detected physical VRAM. Use Advanced Allocation; it has not been rewritten.".into());
        }
        if upper > MAX_EXACT_STEP {
            return Err(
                "VRAM bounds exceed exact slider precision. Use Advanced Allocation.".into(),
            );
        }
        let first_supported = limits.minimum.max(BYTES_PER_GB).div_ceil(BYTES_PER_STEP);
        let last_supported = limits.maximum.min(physical_capacity) / BYTES_PER_STEP;
        let endpoint_supported = (limits.minimum..=limits.maximum).contains(&physical_capacity);
        if first_supported > last_supported && !endpoint_supported {
            return Err(
                "The driver has no supported request in the slider's 1 GB to physical-capacity range. Exact saved values remain available in Advanced Allocation."
                    .into(),
            );
        }
        Ok(Self {
            minimum: lower as i32,
            maximum: upper as i32,
            position: if bounds.optimal == physical_capacity {
                upper as i32
            } else {
                (bounds.optimal / BYTES_PER_STEP).clamp(lower, upper) as i32
            },
            label: current.map_or_else(
                || format!("{} (default request)", format_gb(limits.optimal)),
                |v| format_gb(v.optimal),
            ),
            range_start: format_gb(lower * BYTES_PER_STEP),
            range_end: format_gb(physical_capacity),
            range_note: (bounds.optimal < BYTES_PER_GB).then(|| {
                "The existing request is below 1 GB and has been retained exactly. The slider starts at 1 GB; edit it or use Advanced Allocation to change the request.".into()
            }),
            bounds: bounds.clone(),
            limits: limits.clone(),
            physical_capacity,
        })
    }

    pub fn select(&self, position: i32) -> Result<Allocation, String> {
        if !(self.minimum..=self.maximum).contains(&position) {
            return Err("The selected memory value is outside the current VRAM range.".into());
        }
        let optimal = u64::try_from(position)
            .map_err(|_| "Invalid memory value")?
            .checked_mul(BYTES_PER_STEP)
            .ok_or("Memory conversion overflow")?
            .min(self.physical_capacity);
        if optimal < self.limits.minimum || optimal > self.limits.maximum {
            return Err(format!(
                "The driver supports VRAM requests from {} to {}. The selected physical-capacity value has no supported provider mapping; the draft was retained.",
                format_gb(self.limits.minimum), format_gb(self.limits.maximum),
            ));
        }
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
            minimum: 1,
            optimal: 4_000_000_000,
            maximum: 8_000_000_000,
        }
    }

    #[test]
    fn slider_preserves_bounds_and_writes_exact_decimal_bytes() {
        let current = Allocation {
            minimum: 2_000_000_001,
            optimal: 4_000_000_001,
            maximum: 6_099_999_999,
        };
        let slider = MemorySlider::new(&limits(), Some(&current), 8_000_000_000).unwrap();
        assert_eq!((slider.minimum, slider.maximum), (10, 80));
        assert_eq!(slider.range_start, "1 GB");
        assert_eq!(slider.label, "4.000000001 GB");
        let selected = slider.select(55).unwrap();
        assert_eq!(selected.minimum, current.minimum);
        assert_eq!(selected.maximum, current.maximum);
        assert_eq!(selected.optimal, 5_500_000_000);
        let expanded = slider.select(70).unwrap();
        assert_eq!(expanded.maximum, 7_000_000_000);
        assert_eq!(expanded.minimum, current.minimum);
        assert!(slider.select(81).is_err());
        assert!(slider.select(-1).is_err());
    }

    #[test]
    fn unspecified_request_is_not_invented_until_selection() {
        let limits = limits();
        let slider = MemorySlider::new(&limits, None, 8_000_000_000).unwrap();
        assert_eq!(slider.label, "4 GB (default request)");
        assert_eq!(
            slider.select(80).unwrap(),
            Allocation {
                optimal: 8_000_000_000,
                ..limits
            }
        );
        assert_eq!(
            request(&[String::new(), String::new(), String::new()]).unwrap(),
            None
        );
        assert!(request(&["1".into(), "".into(), "8".into()]).is_err());
    }

    #[test]
    fn one_gb_provider_limit_is_not_rescaled_to_physical_card_capacity() {
        let limits = Allocation {
            minimum: 0,
            optimal: 1_000_000_000,
            maximum: 1_000_000_000,
        };
        let slider = MemorySlider::new(&limits, None, 8_589_934_592).unwrap();
        assert_eq!(slider.range_end, "8.589934592 GB");
        assert_eq!(slider.maximum, 86);
        assert_eq!(slider.minimum, 10);
        assert_eq!(slider.select(10).unwrap().optimal, 1_000_000_000);
        assert!(slider.select(5).is_err());
        assert!(slider.select(0).is_err());
        assert!(slider.select(40).is_err());
    }

    #[test]
    fn physical_endpoint_retains_exact_bytes_without_exceeding_capacity() {
        let capacity = 8_589_934_592;
        let limits = Allocation { minimum: 0, optimal: 1_000_000_000, maximum: capacity };
        let slider = MemorySlider::new(&limits, None, capacity).unwrap();
        let selected = slider.select(slider.maximum).unwrap();
        assert_eq!(selected.optimal, capacity);
        let refreshed = MemorySlider::new(&limits, Some(&selected), capacity).unwrap();
        assert_eq!(refreshed.position, refreshed.maximum);
        assert_eq!(refreshed.label, "8.589934592 GB");
    }

    #[test]
    fn below_minimum_saved_request_is_displayed_exactly_until_an_explicit_edit() {
        let limits = Allocation {
            minimum: 0,
            ..limits()
        };
        let saved = Allocation {
            minimum: 0,
            optimal: 0,
            maximum: 0,
        };
        let slider = MemorySlider::new(&limits, Some(&saved), 8_000_000_000).unwrap();
        assert_eq!(slider.label, "0 GB");
        assert_eq!(slider.position, 10);
        assert_eq!(slider.range_start, "1 GB");
        assert!(
            slider
                .range_note
                .as_ref()
                .unwrap()
                .contains("retained exactly")
        );
        assert_eq!(saved.optimal, 0);
        let edited = slider.select(10).unwrap();
        assert_eq!(edited.optimal, BYTES_PER_GB);
        assert_eq!(edited.minimum, 0);
        assert_eq!(edited.maximum, BYTES_PER_GB);
        assert!(slider.select(9).is_err());
    }

    #[test]
    fn higher_driver_minimum_does_not_replace_the_one_gb_displayed_minimum() {
        let limits = Allocation {
            minimum: 2_000_000_001,
            ..limits()
        };
        let slider = MemorySlider::new(&limits, None, 8_000_000_000).unwrap();
        assert_eq!(slider.range_start, "1 GB");
        assert_eq!(slider.range_end, "8 GB");
        assert!(slider.select(10).is_err());
        assert!(slider.select(20).is_err());
        assert_eq!(slider.select(21).unwrap().optimal, 2_100_000_000);
    }

    #[test]
    fn unsupported_product_ranges_are_unavailable_but_exact_endpoints_are_usable() {
        let limits = Allocation {
            minimum: 0,
            ..limits()
        };
        for capacity in [0, BYTES_PER_GB - 1] {
            assert!(MemorySlider::new(&limits, None, capacity).is_err());
        }
        let fixed = Allocation {
            minimum: 0,
            optimal: BYTES_PER_GB,
            maximum: BYTES_PER_GB,
        };
        let slider = MemorySlider::new(&fixed, None, BYTES_PER_GB).unwrap();
        assert_eq!((slider.minimum, slider.maximum), (10, 10));
        assert_eq!(slider.select(10).unwrap().optimal, BYTES_PER_GB);

        let capacity = 1_050_000_001;
        let narrow = Allocation {
            minimum: 1_040_000_000,
            optimal: capacity,
            maximum: capacity,
        };
        let endpoint = MemorySlider::new(&narrow, None, capacity).unwrap();
        assert_eq!(endpoint.select(11).unwrap().optimal, capacity);
        assert!(endpoint.select(10).is_err());
        // A narrow range between ticks cannot offer a guessed or rounded request.
        assert!(MemorySlider::new(&narrow, None, 2_000_000_000).is_err());
    }

    #[test]
    fn invalid_narrow_and_imprecise_ranges_stay_unavailable() {
        assert!(
            MemorySlider::new(
                &Allocation {
                    minimum: 9,
                    optimal: 8,
                    maximum: 10
                },
                None, 8_000_000_000
            )
            .is_err()
        );
        assert!(
            MemorySlider::new(
                &Allocation {
                    minimum: 0,
                    optimal: 1,
                    maximum: 99_999_999
                },
                None, 8_000_000_000
            )
            .is_err()
        );
        assert!(
            MemorySlider::new(
                &Allocation {
                    minimum: 0,
                    optimal: 1,
                    maximum: u64::MAX
                },
                None, u64::MAX
            )
            .is_err()
        );
        assert!(
            MemorySlider::new(
                &limits(),
                Some(&Allocation {
                    minimum: 1,
                    optimal: 9_000_000_000,
                    maximum: 9_000_000_000
                }), 8_000_000_000
            )
            .is_err()
        );
        assert_eq!(format_gb(u64::MAX), "18446744073.709551615 GB");
    }
}
