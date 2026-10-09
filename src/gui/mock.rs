//! Deterministic fixtures. This module never discovers or changes host resources.
use super::VmRow;

pub fn inventory(many: bool) -> Vec<VmRow> {
    let fixtures = [
        (
            "Design studio",
            "Running",
            "Configured",
            "NVIDIA GeForce RTX 5060",
            "windows",
        ),
        (
            "Windows 11 · Development",
            "Stopped",
            "Setup required",
            "No GPU assigned",
            "windows",
        ),
        (
            "Video editing suite",
            "Stopped",
            "Configured",
            "NVIDIA GeForce RTX 4070",
            "windows",
        ),
        (
            "Legacy accounting · Windows 10",
            "Stopped",
            "Unsupported",
            "Generation 1 · ineligible",
            "windows",
        ),
        (
            "Research workstation",
            "Unknown",
            "Unknown",
            "Inventory access denied · sample",
            "unknown",
        ),
        (
            "Windows 11 · Architectural visualisation and rendering workstation",
            "Running",
            "Configured",
            "NVIDIA GeForce RTX 5060 · sharing unqualified",
            "windows",
        ),
        (
            "Ubuntu · Build server",
            "Running",
            "Unknown",
            "GPU information unavailable · sample",
            "linux",
        ),
    ];
    let mut rows: Vec<_> = fixtures
        .iter()
        .enumerate()
        .map(|(index, (name, power, status, gpu, os))| VmRow {
            id: format!("a5801e91-1083-4e79-a803-{:012}", index + 1).into(),
            name: (*name).into(),
            power: (*power).into(),
            status: (*status).into(),
            gpu: (*gpu).into(),
            os: (*os).into(),
        })
        .collect();
    if many {
        for index in 8..=35 {
            rows.push(VmRow {
                id: format!("a5801e91-1083-4e79-a803-{index:012}").into(),
                name: format!("Windows 11 · Test workspace {index:02}").into(),
                power: "Stopped".into(),
                status: "Setup required".into(),
                gpu: "No GPU assigned".into(),
                os: "windows".into(),
            });
        }
    }
    rows
}

pub fn validate(values: &[String], gpu: i32, enabled: bool) -> String {
    if !(0..=2).contains(&gpu) {
        return "Select a valid sample GPU.".into();
    }
    if values.len() != 12 {
        return "All twelve sample allocation fields are required.".into();
    }
    if !enabled {
        return String::new();
    }
    if gpu == 2 {
        return "The sample AMD adapter is unavailable. Select an NVIDIA sample GPU.".into();
    }
    for (index, triple) in values.chunks(3).enumerate() {
        let parsed: Result<Vec<u32>, _> = triple.iter().map(|value| value.parse()).collect();
        let category = ["VRAM", "Compute", "Encode", "Decode"][index];
        match parsed {
            Ok(v) if v.len() == 3 && v[0] <= v[1] && v[1] <= v[2] && v[2] <= 1000 => {}
            _ => {
                return format!(
                    "{category}: enter whole mock units with 0 ≤ Minimum ≤ Optimal ≤ Maximum ≤ 1000."
                );
            }
        }
    }
    String::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn allocations_reject_order_and_unavailable_adapter() {
        let mut values: Vec<String> = ["100", "500", "1000"]
            .repeat(4)
            .into_iter()
            .map(str::to_owned)
            .collect();
        assert!(validate(&values, 0, true).is_empty());
        values[0] = "600".into();
        assert!(validate(&values, 0, true).starts_with("VRAM:"));
        assert!(!validate(&values, 2, true).is_empty());
        assert!(validate(&values, 2, false).is_empty());
    }

    #[test]
    fn incomplete_fields_and_invalid_gpu_are_rejected() {
        let values = vec!["100".to_owned(); 12];
        assert!(!validate(&values[..11], 0, true).is_empty());
        assert!(!validate(&[], 0, false).is_empty());
        for gpu in [-1, 3, i32::MAX] {
            assert!(!validate(&values, gpu, false).is_empty());
        }
    }
}
