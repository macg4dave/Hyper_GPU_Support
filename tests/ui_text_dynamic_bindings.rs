//! Execute migrated data-only templates against the independent output fixtures.
use hyper_gpu_support::ui_text;

#[test]
fn migrated_dynamic_bindings_preserve_historical_outputs() {
    let outputs: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/ui-text/outputs.json")).unwrap();
    let revisions: std::collections::BTreeMap<String, String> =
        serde_json::from_str(include_str!("fixtures/ui-text/revised-dynamic.json")).unwrap();
    let mut exercised = std::collections::BTreeSet::new();
    for case in outputs["cases"].as_array().unwrap() {
        let key = case["key"].as_str().unwrap_or("");
        let inputs = &case["inputs"];
        let actual = match key {
            "credentials.guest_sign_in.message" => {
                ui_text::credentials::guest_sign_in::message(inputs["vm_name"].as_str().unwrap())
            }
            "dashboard.vm_list.count" => ui_text::dashboard::vm_list::count(
                usize::try_from(inputs["count"].as_u64().unwrap()).unwrap(),
            ),
            "diagnostics.history.graphics.last_success" => {
                ui_text::diagnostics::history::graphics::last_success(
                    inputs["unix_time"].as_u64().unwrap(),
                )
            }
            "operations.action.review_prompt" => {
                ui_text::operations::action::review_prompt(inputs["vm_name"].as_str().unwrap())
            }
            "operations.apply.completed.body" => {
                ui_text::operations::apply::completed::body(inputs["vm_name"].as_str().unwrap())
            }
            "operations.apply.recovery.body" => {
                ui_text::operations::apply::recovery::body(inputs["vm_name"].as_str().unwrap())
            }
            "operations.apply.review.vm_heading" => {
                ui_text::operations::apply::review::vm_heading(inputs["vm_name"].as_str().unwrap())
            }
            "operations.apply.unsaved.effect_summary" => {
                ui_text::operations::apply::unsaved::effect_summary(
                    inputs["vm_name"].as_str().unwrap(),
                )
            }
            "operations.import.recovery.body" => {
                ui_text::operations::configuration_import::recovery::body(
                    inputs["operation_id"].as_str().unwrap(),
                    usize::try_from(inputs["destination_count"].as_u64().unwrap()).unwrap(),
                    usize::try_from(inputs["published_count"].as_u64().unwrap()).unwrap(),
                )
            }
            "operations.enrollment.artifact_pin" => ui_text::operations::enrollment::artifact_pin(
                inputs["artifact_name"].as_str().unwrap(),
                inputs["digest"].as_str().unwrap(),
            ),
            "operations.enrollment.review.body" => ui_text::operations::enrollment::review::body(
                inputs["vm_name"].as_str().unwrap(),
                inputs["gpu_name"].as_str().unwrap(),
            ),
            "operations.profile_deletion.completed.body" => {
                ui_text::operations::profile_deletion::completed::body(
                    inputs["vm_name"].as_str().unwrap(),
                )
            }
            "operations.profile_deletion.review.body" => {
                ui_text::operations::profile_deletion::review::body(
                    inputs["vm_name"].as_str().unwrap(),
                    inputs["vm_id"].as_str().unwrap(),
                    inputs["gpu_interface"].as_str().unwrap(),
                )
            }
            "operations.recovery.review.body" => {
                ui_text::operations::recovery::review::body(inputs["vm_name"].as_str().unwrap())
            }
            "operations.recovery.vm_unfinished" => {
                ui_text::operations::recovery::vm_unfinished(inputs["vm_name"].as_str().unwrap())
            }
            "operations.setup.unsaved.effect_summary" => {
                ui_text::operations::setup::unsaved::effect_summary(
                    inputs["vm_name"].as_str().unwrap(),
                )
            }
            "operations.verify.completed.body" => {
                ui_text::operations::verify::completed::body(inputs["vm_name"].as_str().unwrap())
            }
            "operations.verify.unknown.body" => {
                ui_text::operations::verify::unknown::body(inputs["vm_name"].as_str().unwrap())
            }
            _ => continue,
        };
        let expected = revisions.get(key).map_or_else(
            || case["expected"].as_str().unwrap().to_owned(),
            |template| {
                ::ui_text::template::parse(template)
                    .unwrap()
                    .into_iter()
                    .map(|segment| match segment {
                        ::ui_text::template::Segment::Literal(text) => text,
                        ::ui_text::template::Segment::Parameter(name) => {
                            inputs[&name].as_str().unwrap().to_owned()
                        }
                    })
                    .collect::<String>()
            },
        );
        assert_eq!(actual.as_str(), expected, "{}", case["id"]);
        exercised.insert(key);
    }
    assert_eq!(exercised.len(), 18);
}
