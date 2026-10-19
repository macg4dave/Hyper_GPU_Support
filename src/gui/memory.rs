//! Exact display conversion for the native Hyper-V VRAM byte contract.
//! This requests values; it makes no physical reservation/enforcement claim.
use hyper_gpu_support::model::Allocation;

const BYTES_PER_GB: u64 = 1_000_000_000;
const BYTES_PER_STEP: u64 = BYTES_PER_GB / 10;
// Every selectable integer must survive Slint's f32 Slider representation.
const MAX_EXACT_STEP: u64 = 1 << 24;

/// Permit incomplete decimal edits, but reject invalid keystrokes/pastes as a whole.
pub(super) fn acceptable_input(text: slint::SharedString) -> bool {
    let (whole, fraction) = text.split_once('.').unwrap_or((&text, ""));
    whole.bytes().all(|b| b.is_ascii_digit())
        && fraction.len() <= 9
        && fraction.bytes().all(|b| b.is_ascii_digit())
}

pub(super) struct MemorySlider {
    pub minimum: i32,
    pub maximum: i32,
    pub position: i32,
    pub label: String,
    pub range_start: String,
    pub range_end: String,
    pub range_note: Option<String>,
    bounds: Allocation,
    pub physical_capacity: u64,
}

pub(super) fn format_gb(bytes: u64) -> String {
    hyper_gpu_support::ui_text::resources::units::gb(&decimal_gb(bytes)).into_string()
}

pub(super) fn decimal_gb(bytes: u64) -> String {
    let whole = bytes / BYTES_PER_GB;
    let fraction = bytes % BYTES_PER_GB;
    if fraction == 0 {
        format!("{whole}")
    } else {
        let digits = format!("{fraction:09}");
        format!("{whole}.{}", digits.trim_end_matches('0'))
    }
}

// Presentation only. Quotient/remainder avoids overflow when rounding near u64::MAX.
fn rounded_capacity(bytes: u64) -> String {
    let whole = bytes / BYTES_PER_GB + u64::from(bytes % BYTES_PER_GB >= BYTES_PER_GB / 2);
    hyper_gpu_support::ui_text::resources::units::gb(&whole.to_string()).into_string()
}

/// Physical capacity is displayed separately from the provider's byte request.
pub(super) fn format_gib(bytes: u64) -> String {
    hyper_gpu_support::ui_text::resources::units::gib(&format!(
        "{:.2}",
        bytes as f64 / (1u64 << 30) as f64
    ))
    .into_string()
}

/// Incomplete/invalid raw edits are never replaced by moving the slider.
pub(super) fn request(values: &[String]) -> Result<Option<Allocation>, String> {
    if values.len() != 3 {
        return Err(hyper_gpu_support::ui_text::resources::memory::fields_unavailable().into());
    }
    if values.iter().all(|v| v.trim().is_empty()) {
        return Ok(None);
    }
    let parsed = values
        .iter()
        .map(|v| v.trim().parse::<u64>())
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| hyper_gpu_support::ui_text::resources::memory::fields_incomplete())?;
    Ok(Some(Allocation {
        minimum: parsed[0],
        optimal: parsed[1],
        maximum: parsed[2],
    }))
}

fn parse_gb(text: &str) -> Result<u64, String> {
    let text = text.trim();
    let (whole, fraction) = text.split_once('.').unwrap_or((text, ""));
    if whole.is_empty()
        || !whole.bytes().all(|b| b.is_ascii_digit())
        || fraction.len() > 9
        || !fraction.bytes().all(|b| b.is_ascii_digit())
    {
        return Err(hyper_gpu_support::ui_text::resources::memory::decimal_required().into());
    }
    let whole = whole
        .parse::<u64>()
        .map_err(|_| hyper_gpu_support::ui_text::resources::memory::too_large())?;
    let fraction = format!("{fraction:0<9}")
        .parse::<u64>()
        .map_err(|_| hyper_gpu_support::ui_text::resources::memory::invalid_decimal())?;
    let bytes = whole
        .checked_mul(BYTES_PER_GB)
        .and_then(|b| b.checked_add(fraction))
        .ok_or(hyper_gpu_support::ui_text::resources::memory::too_large())?;
    Ok(bytes)
}

pub(super) fn valid_input(text: &str, capacity: &str) -> bool {
    acceptable_input(text.into())
        && parse_gb(text)
            .ok()
            .zip(parse_gb(capacity).ok())
            .is_some_and(|(value, maximum)| (BYTES_PER_GB..=maximum).contains(&value))
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
            return Err(hyper_gpu_support::ui_text::resources::memory::below_minimum().into());
        }
        let bounds = current.unwrap_or(limits);
        bounds
            .validate(bounds)
            .map_err(|_| hyper_gpu_support::ui_text::resources::memory::range_invalid())?;
        // Physical capacity bounds the slider. Reported partition metadata must
        // not impose a second (for example 1 GB) ceiling on byte requests.
        let lower = BYTES_PER_GB / BYTES_PER_STEP;
        let upper = physical_capacity.div_ceil(BYTES_PER_STEP);
        if bounds.optimal > physical_capacity {
            return Err(hyper_gpu_support::ui_text::resources::memory::exceeds_physical().into());
        }
        if upper > MAX_EXACT_STEP {
            return Err(hyper_gpu_support::ui_text::resources::memory::precision_exceeded().into());
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
                || hyper_gpu_support::ui_text::resources::request_absent().into(),
                |v| {
                    if v.optimal == physical_capacity {
                        rounded_capacity(physical_capacity)
                    } else {
                        format_gb(v.optimal)
                    }
                },
            ),
            range_start: format_gb(lower * BYTES_PER_STEP),
            range_end: rounded_capacity(physical_capacity),
            range_note: {
                let mut notes = Vec::new();
                if !physical_capacity.is_multiple_of(BYTES_PER_GB) {
                    notes.push(hyper_gpu_support::ui_text::resources::endpoint_rounding_note());
                }
                if bounds.optimal < BYTES_PER_GB {
                    notes
                        .push(hyper_gpu_support::ui_text::resources::retained_below_minimum_note());
                }
                (!notes.is_empty()).then(|| notes.join("\n"))
            },
            bounds: bounds.clone(),
            physical_capacity,
        })
    }

    pub fn select(&self, position: i32) -> Result<Allocation, String> {
        if !(self.minimum..=self.maximum).contains(&position) {
            return Err(hyper_gpu_support::ui_text::resources::memory::outside_range().into());
        }
        let optimal = u64::try_from(position)
            .map_err(|_| hyper_gpu_support::ui_text::resources::memory::invalid_value())?
            .checked_mul(BYTES_PER_STEP)
            .ok_or(hyper_gpu_support::ui_text::resources::memory::overflow())?
            .min(self.physical_capacity);
        self.select_bytes(optimal)
    }

    /// Parse decimal GB without floating-point rounding or silently clamping input.
    pub fn select_gb(&self, text: &str) -> Result<Allocation, String> {
        let bytes = parse_gb(text)?;
        if bytes < BYTES_PER_GB || bytes > self.physical_capacity {
            return Err(
                hyper_gpu_support::ui_text::resources::memory::request_range(&format_gb(
                    self.physical_capacity,
                ))
                .into_string(),
            );
        }
        self.select_bytes(bytes)
    }

    fn select_bytes(&self, optimal: u64) -> Result<Allocation, String> {
        let result = Allocation {
            minimum: self.bounds.minimum.min(optimal),
            optimal,
            // Only an explicit edit adopts the selected card's exact maximum.
            maximum: self.physical_capacity,
        };
        result.validate(&result)?;
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn selected_request_formats_exact_capacity_and_custom_bytes() {
        for (optimal, expected) in [
            (8_278_507_520_u64, "8.27850752 GB"),
            (5_900_000_000, "5.9 GB"),
            (4_000_000_001, "4.000000001 GB"),
        ] {
            let values = ["1".to_owned(), optimal.to_string(), "8278507520".to_owned()];
            assert_eq!(
                super::format_gb(super::request(&values).unwrap().unwrap().optimal),
                expected
            );
        }
    }

    #[test]
    fn request_distinguishes_default_from_incomplete_allocation() {
        assert!(
            super::request(&[String::new(), String::new(), String::new()])
                .unwrap()
                .is_none()
        );
        assert!(super::request(&["1".into(), String::new(), "8000000000".into()]).is_err());
        assert!(super::request(&[]).is_err());
    }

    #[test]
    fn typing_and_paste_reject_non_decimal_text_and_bounds_disable_submission() {
        for text in ["", ".", "7.", "7.0", "8.278507520"] {
            assert!(super::acceptable_input(text.into()), "{text}");
        }
        for text in [
            "7 GB",
            "7x",
            "1e1",
            "-7",
            "+7",
            "7,0",
            "7..0",
            " 7",
            "7\n",
            "７",
            "7.1234567890",
        ] {
            assert!(!super::acceptable_input(text.into()), "{text}");
            assert!(!super::valid_input(text, "8.27850752"));
        }
        for text in ["", ".", "0.9", "8.278507521", "999999999999999999999"] {
            assert!(!super::valid_input(text, "8.27850752"), "{text}");
        }
        for text in ["1", "7.0", "7.1", "8.278507520"] {
            assert!(super::valid_input(text, "8.27850752"), "{text}");
        }
        assert!(
            !super::valid_input("7.1", "4"),
            "selected GPU capacity changes validation"
        );
        assert!(!super::valid_input("1", ""));
    }

    #[test]
    fn numeric_editor_input_roundtrips_exact_bytes_without_a_display_unit() {
        for bytes in [0, 1, 6_000_000_001, 8_278_507_520, u64::MAX] {
            let number = super::decimal_gb(bytes);
            let (whole, fraction) = number.split_once('.').unwrap_or((&number, ""));
            let fraction = format!("{fraction:0<9}").parse::<u64>().unwrap();
            assert_eq!(
                whole.parse::<u64>().unwrap() * super::BYTES_PER_GB + fraction,
                bytes
            );
        }
    }

    use super::*;

    fn limits() -> Allocation {
        Allocation {
            minimum: 1,
            optimal: 4_000_000_000,
            maximum: 8_000_000_000,
        }
    }

    #[test]
    fn numeric_memory_input_is_byte_exact_and_rejects_invalid_values_without_clamping() {
        let slider = MemorySlider::new(&limits(), None, 8_278_507_520).unwrap();
        let selected = slider.select_gb("6.000000001").unwrap();
        assert_eq!(selected.optimal, 6_000_000_001);
        assert_eq!(selected.maximum, 8_278_507_520);
        assert_eq!(
            slider.select_gb("8.278507520").unwrap().optimal,
            8_278_507_520
        );
        for input in [
            "",
            "0.9",
            "8.278507521",
            "6.0000000001",
            "NaN",
            "1e9",
            "-1",
            "18446744074",
        ] {
            assert!(slider.select_gb(input).is_err(), "{input}");
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
        assert_eq!(selected.maximum, 8_000_000_000);
        assert_eq!(selected.optimal, 5_500_000_000);
        let expanded = slider.select(70).unwrap();
        assert_eq!(expanded.maximum, 8_000_000_000);
        assert_eq!(expanded.minimum, current.minimum);
        assert!(slider.select(81).is_err());
        assert!(slider.select(-1).is_err());
    }

    #[test]
    fn slider_edits_use_new_card_capacity_without_migrating_loaded_legacy_maximum() {
        let saved = Allocation {
            minimum: 1,
            optimal: 4_000_000_001,
            maximum: 16_000_000_000,
        };
        for capacity in [8_589_934_592, 24_000_000_000] {
            let slider = MemorySlider::new(&limits(), Some(&saved), capacity).unwrap();
            assert_eq!(slider.label, "4.000000001 GB");
            for position in [20, 60, slider.maximum] {
                let edited = slider.select(position).unwrap();
                assert_eq!(edited.maximum, capacity);
                assert!(edited.optimal <= edited.maximum);
            }
            assert_eq!(saved.maximum, 16_000_000_000);
        }
        let slider = MemorySlider::new(&limits(), None, 8_589_934_592).unwrap();
        assert_eq!(slider.select(20).unwrap().maximum, 8_589_934_592);
    }

    #[test]
    fn capacity_labels_round_half_up_without_changing_selection_bytes() {
        for (capacity, expected) in [
            (8_499_999_999, "8 GB"),
            (8_500_000_000, "9 GB"),
            (8_500_000_001, "9 GB"),
            (8_000_000_000, "8 GB"),
        ] {
            let slider = MemorySlider::new(&limits(), None, capacity).unwrap();
            assert_eq!(slider.range_end, expected);
            let selected = slider.select(slider.maximum).unwrap();
            assert_eq!(selected.optimal, capacity);
            assert_eq!(selected.maximum, capacity);
            let loaded = MemorySlider::new(&limits(), Some(&selected), capacity).unwrap();
            assert_eq!(loaded.label, expected);
            assert_eq!(loaded.position, loaded.maximum);
            assert_eq!(slider.select(40).unwrap().optimal, 4_000_000_000);
        }
        assert_eq!(rounded_capacity(u64::MAX), "18446744074 GB");
    }

    #[test]
    fn unspecified_request_is_not_invented_until_selection() {
        let limits = limits();
        let slider = MemorySlider::new(&limits, None, 8_000_000_000).unwrap();
        assert_eq!(slider.label, "Not set");
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
    fn raw_provider_metadata_does_not_establish_a_physical_gb_ceiling() {
        let limits = Allocation {
            minimum: 0,
            optimal: 1_000_000_000,
            maximum: 1_000_000_000,
        };
        let slider = MemorySlider::new(&limits, None, 8_589_934_592).unwrap();
        assert_eq!(slider.range_end, "9 GB");
        assert_eq!(slider.maximum, 86);
        assert_eq!(slider.minimum, 10);
        assert_eq!(slider.select(10).unwrap().optimal, 1_000_000_000);
        assert!(slider.select(5).is_err());
        assert!(slider.select(0).is_err());
        assert_eq!(slider.select(40).unwrap().optimal, 4_000_000_000);
        let selected = slider.select(slider.maximum).unwrap();
        assert_eq!(selected.maximum, 8_589_934_592);
        assert_eq!(
            MemorySlider::new(&limits, Some(&selected), 8_589_934_592)
                .unwrap()
                .label,
            "9 GB"
        );
    }

    #[test]
    fn physical_endpoint_retains_exact_bytes_without_exceeding_capacity() {
        let capacity = 8_589_934_592;
        let limits = Allocation {
            minimum: 0,
            optimal: 1_000_000_000,
            maximum: capacity,
        };
        let slider = MemorySlider::new(&limits, None, capacity).unwrap();
        let selected = slider.select(slider.maximum).unwrap();
        assert_eq!(selected.optimal, capacity);
        let refreshed = MemorySlider::new(&limits, Some(&selected), capacity).unwrap();
        assert_eq!(refreshed.position, refreshed.maximum);
        assert_eq!(refreshed.label, "9 GB");
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
        assert!(slider.range_note.as_ref().unwrap().contains("kept exactly"));
        assert_eq!(saved.optimal, 0);
        let edited = slider.select(10).unwrap();
        assert_eq!(edited.optimal, BYTES_PER_GB);
        assert_eq!(edited.minimum, 0);
        assert_eq!(edited.maximum, 8_000_000_000);
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
        assert_eq!(slider.select(10).unwrap().optimal, BYTES_PER_GB);
        assert_eq!(slider.select(20).unwrap().optimal, 2_000_000_000);
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
        assert_eq!(endpoint.select(10).unwrap().optimal, BYTES_PER_GB);
        // Partition metadata does not narrow the physical slider range.
        assert_eq!(
            MemorySlider::new(&narrow, None, 2_000_000_000)
                .unwrap()
                .select(20)
                .unwrap()
                .optimal,
            2_000_000_000
        );
    }

    #[test]
    fn invalid_and_imprecise_ranges_stay_unavailable() {
        assert!(
            MemorySlider::new(
                &Allocation {
                    minimum: 9,
                    optimal: 8,
                    maximum: 10
                },
                None,
                8_000_000_000
            )
            .is_err()
        );
        assert_eq!(
            MemorySlider::new(
                &Allocation {
                    minimum: 0,
                    optimal: 1,
                    maximum: 99_999_999
                },
                None,
                8_000_000_000
            )
            .unwrap()
            .select(80)
            .unwrap()
            .optimal,
            8_000_000_000
        );
        assert!(
            MemorySlider::new(
                &Allocation {
                    minimum: 0,
                    optimal: 1,
                    maximum: u64::MAX
                },
                None,
                u64::MAX
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
                }),
                8_000_000_000
            )
            .is_err()
        );
        assert_eq!(format_gb(u64::MAX), "18446744073.709551615 GB");
    }
}
