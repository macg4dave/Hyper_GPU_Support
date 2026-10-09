//! Production-format per-VM fixtures use the same core parser and GUI view model.
use hyper_gpu_support::{gui_model::View, model::Configuration};
use std::path::Path;

const IDS: [&str; 3] = [
    "a5801e91-1083-4e79-a803-000000000001",
    "a5801e91-1083-4e79-a803-000000000003",
    "a5801e91-1083-4e79-a803-000000000006",
];

#[test]
fn cli_and_gui_read_the_same_production_format_without_creating_observations_or_drafts() {
    for id in IDS {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("config/samples/vms").join(format!("{id}.toml"));
        let (mut view, source) = View::from_vm_file(&path).unwrap();
        let cli = Configuration::parse(&source).unwrap();
        assert_eq!(view.configuration.schema, cli.schema);
        assert_eq!(view.configuration.targets, cli.targets);
        assert_eq!(view.configuration.targets[0].vm_id, id);
        assert!(view.inventory.is_none());
        assert!(view.draft.is_none());
        assert!(!view.unsaved);
        let saved = view.saved_desired_text(id);
        let mut draft = view.configuration.targets[0].clone();
        draft.enabled = !draft.enabled;
        view.draft = Some(draft);
        assert_eq!(view.saved_desired_text(id), saved);
        assert_eq!(view.configuration.targets, cli.targets);
        assert!(view.inventory.is_none());
    }
}

#[test]
fn per_vm_boundary_rejects_filename_mismatch_unknown_fields_and_multiple_targets() {
    let source = include_str!("../config/samples/vms/a5801e91-1083-4e79-a803-000000000001.toml");
    let filename = format!("{}.toml", IDS[0]);
    assert!(Configuration::parse_vm_file(source, &format!("{}.toml", IDS[1])).is_err());
    assert!(Configuration::parse_vm_file(source, &filename.to_ascii_uppercase()).is_err());
    assert!(Configuration::parse_vm_file(&format!("{source}\ncompute = 1\n"), &filename).is_err());
    let upper_id = source.replace(IDS[0], &IDS[0].to_ascii_uppercase());
    assert_eq!(Configuration::parse_vm_file(&upper_id, &filename).unwrap().targets[0].vm_id, IDS[0]);
    let mut bundle = Configuration::parse(source).unwrap();
    let mut other = bundle.targets[0].clone();
    other.vm_id = IDS[1].into();
    bundle.targets.push(other);
    let bundled = toml::to_string(&bundle).unwrap();
    assert_eq!(Configuration::parse(&bundled).unwrap().targets.len(), 2);
    assert!(Configuration::parse_vm_file(&bundled, &filename).is_err());
}

#[test]
fn explicit_bundle_split_preserves_intent_and_normalizes_filename_identity() {
    let mut bundle = Configuration { schema: 2, targets: vec![] };
    for source in [
        include_str!("../config/samples/vms/a5801e91-1083-4e79-a803-000000000001.toml"),
        include_str!("../config/samples/vms/a5801e91-1083-4e79-a803-000000000003.toml"),
        include_str!("../config/samples/vms/a5801e91-1083-4e79-a803-000000000006.toml"),
    ] {
        bundle.targets.extend(Configuration::parse(source).unwrap().targets);
    }
    bundle.targets[0].vm_id.make_ascii_uppercase();
    let documents = bundle.vm_documents().unwrap();
    assert_eq!(documents.len(), 3);
    // The source remains untouched, including original capitalization.
    assert_eq!(bundle.targets[0].vm_id, IDS[0].to_ascii_uppercase());
    for target in &bundle.targets {
        let filename = format!("{}.toml", target.vm_id.to_ascii_lowercase());
        let parsed = Configuration::parse_vm_file(&documents[&filename], &filename).unwrap();
        let mut expected = target.clone();
        expected.vm_id.make_ascii_lowercase();
        assert_eq!(parsed.targets, vec![expected]);
    }
}

#[test]
fn bundle_split_refuses_duplicate_identities_invalid_intent_and_unknown_versions() {
    let source = include_str!("../config/samples/vms/a5801e91-1083-4e79-a803-000000000001.toml");
    let valid = Configuration::parse(source).unwrap();
    let mut duplicate = valid.clone();
    let mut upper = duplicate.targets[0].clone();
    upper.vm_id.make_ascii_uppercase();
    duplicate.targets.push(upper);
    assert!(duplicate.vm_documents().is_err());
    let mut invalid = valid.clone();
    invalid.targets[0].gpu_interface = "not-a-provider-interface".into();
    assert!(invalid.vm_documents().is_err());
    let mut future = valid;
    future.schema = 3;
    assert!(future.vm_documents().is_err());
}
