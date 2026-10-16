use super::*;

#[test]
fn draft_tracking_ignores_unchanged_values_and_retains_invalid_edits() {
    let s = state();
    let mut baseline = s.target(id(&s)).unwrap();
    baseline.vram = Some(Allocation {
        minimum: 10,
        optimal: 50,
        maximum: 100,
    });
    let mut values = allocation_values(&baseline);
    let gpu = Some(baseline.gpu_interface.as_str());
    assert!(!s.draft_changed(&baseline, baseline.enabled, gpu, &values));
    values[1] = " 050 ".into();
    assert!(!s.draft_changed(&baseline, baseline.enabled, gpu, &values));
    assert!(s.draft_changed(&baseline, !baseline.enabled, gpu, &values));
    values[1] = "invalid".into();
    assert!(s.draft_changed(&baseline, baseline.enabled, gpu, &values));
    values[1] = "50".into();
    assert!(!s.draft_changed(&baseline, baseline.enabled, gpu, &values));
}

#[test]
fn retained_inventory_allows_memory_drafts_but_cannot_authorize_apply() {
    let mut s = state();
    s.view.inventory.as_mut().unwrap().discovery.gpus[0].vram = Allocation {
        minimum: 0,
        optimal: 1_000_000_000,
        maximum: 8_000_000_000,
    }
    .into();
    s.view.inventory.as_mut().unwrap().discovery.gpus[0].physical_vram_bytes = Some(8_000_000_000);
    s.view.needs_readback = true;
    assert!(s.memory_slider(0, &vec![String::new(); 12]).is_ok());
    let target = s.target(id(&s)).unwrap();
    assert!(s.editor_eligibility(id(&s)).is_ok());
    assert!(s.view.eligibility(&target).is_err());
}

#[test]
fn memory_selection_and_advanced_fields_produce_the_same_draft() {
    let mut s = state();
    s.view.inventory.as_mut().unwrap().discovery.gpus[0].vram = Allocation {
        minimum: 1,
        optimal: 4_000_000_000,
        maximum: 8_000_000_000,
    }
    .into();
    s.view.inventory.as_mut().unwrap().discovery.gpus[0].physical_vram_bytes = Some(8_000_000_000);
    s.view.inventory.as_mut().unwrap().discovery.gpus[0].total_vram = Some(8_000_000_000);
    let mut raw = values("2000000000", "4000000000", "6000000000");
    let selected = s.memory_slider(0, &raw).unwrap().select(55).unwrap();
    raw = s.memory_values(id(&s), 0, &raw, 55).unwrap();
    let draft = s.draft(id(&s), true, 0, &raw).unwrap();
    assert_eq!(draft.vram, Some(selected));
    assert!(s.draft_changed(
        &s.target(id(&s)).unwrap(),
        true,
        Some(&draft.gpu_interface),
        &raw
    ));
    // Unsupported/incomplete raw text must remain intact rather than being
    // replaced with guessed defaults from the slider.
    raw[1] = "invalid".into();
    assert!(s.memory_slider(0, &raw).is_err());
    assert_eq!(raw[1], "invalid");
}

#[test]
fn numeric_memory_edit_preserves_other_fields_and_refuses_invalid_raw_triples() {
    let mut s = state();
    let gpu = &mut s.view.inventory.as_mut().unwrap().discovery.gpus[0];
    gpu.vram = Allocation {
        minimum: 0,
        optimal: 1_000_000_000,
        maximum: 1_000_000_000,
    }
    .into();
    gpu.physical_vram_bytes = Some(8_278_507_520);
    let mut raw = values("0", "4000000000", "8000000000");
    raw[3..].fill("123".into());
    let edited = s
        .memory_text_values(id(&s), 0, &raw, "6.000000001")
        .unwrap();
    assert_eq!(&edited[..3], &["0", "6000000001", "8278507520"]);
    assert_eq!(&edited[3..], &raw[3..]);
    assert!(s.memory_text_values(id(&s), 0, &raw, "9").is_err());
    raw[1] = "invalid".into();
    assert!(s.memory_text_values(id(&s), 0, &raw, "6").is_err());
    assert_eq!(raw[1], "invalid");
}

#[test]
fn slider_capacity_follows_selected_gpu_and_unknown_capacity_stays_unknown() {
    let mut s = state();
    let gpus = &mut s.view.inventory.as_mut().unwrap().discovery.gpus;
    gpus[0].vram = Allocation {
        minimum: 0,
        optimal: 1_000_000_000,
        maximum: 1_000_000_000,
    }
    .into();
    gpus[0].physical_vram_bytes = Some(8_000_000_000);
    let mut second = gpus[0].clone();
    second.interface = "second physical GPU".into();
    second.physical_vram_bytes = Some(16_000_000_000);
    gpus.push(second);
    let raw = vec![String::new(); 12];
    assert_eq!(s.memory_slider(0, &raw).unwrap().range_start, "1 GB");
    assert_eq!(s.memory_slider(1, &raw).unwrap().range_start, "1 GB");
    assert_eq!(s.memory_slider(0, &raw).unwrap().maximum, 80);
    assert_eq!(s.memory_slider(1, &raw).unwrap().maximum, 160);
    assert_eq!(
        s.memory_slider(1, &raw)
            .unwrap()
            .select(20)
            .unwrap()
            .optimal,
        2_000_000_000
    );
    s.view.inventory.as_mut().unwrap().discovery.gpus[1].physical_vram_bytes = None;
    assert!(s.memory_slider(1, &raw).is_err());
    s.view.inventory.as_mut().unwrap().discovery.gpus[1].physical_vram_bytes = Some(0);
    assert!(s.memory_slider(1, &raw).is_err());
}

#[test]
fn memory_edits_preserve_saved_schemas_other_categories_and_other_vms() {
    for schema in [2, 3] {
        let mut s = state();
        let vm_id = id(&s).to_owned();
        let gpu = &mut s.view.inventory.as_mut().unwrap().discovery.gpus[0];
        gpu.vram = Allocation {
            minimum: 0,
            optimal: 4_000_000_000,
            maximum: 8_000_000_000,
        }
        .into();
        gpu.physical_vram_bytes = Some(8_000_000_000);
        let compatible = Allocation {
            minimum: 10,
            optimal: 50,
            maximum: 100,
        };
        gpu.compute = compatible.clone().into();
        gpu.encode = compatible.clone().into();
        gpu.decode = compatible.clone().into();
        let mut saved = s.target(&vm_id).unwrap();
        saved.vram = Some(Allocation {
            minimum: 0,
            optimal: 0,
            maximum: 0,
        });
        if schema == 3 {
            saved.compute = Some(compatible.clone());
            saved.encode = Some(compatible.clone());
            saved.decode = Some(compatible);
        }
        let mut other = saved.clone();
        other.vm_id = "87654321-4321-4321-4321-cba987654321".into();
        let configuration = Configuration {
            schema,
            targets: vec![saved.clone(), other.clone()],
        };
        let mut store = hyper_gpu_support::configuration_store::StoreSnapshot::default();
        for (filename, source) in configuration.vm_documents().unwrap() {
            store.insert(&filename, Ok(source));
        }
        s.load_committed(store).unwrap();
        let source = s.committed.documents[&vm_id].source.clone();
        let original = allocation_values(&saved);
        assert_eq!(s.memory_slider(0, &original).unwrap().label, "0 GB");
        assert!(s.memory_values(&vm_id, 0, &original, 9).is_err());
        let edited = s.memory_values(&vm_id, 0, &original, 10).unwrap();
        assert_eq!(&edited[3..], &original[3..]);
        let enabled = s.draft(&vm_id, true, 0, &edited).unwrap();
        assert!(enabled.enabled);
        assert_eq!(enabled.vram.as_ref().unwrap().optimal, 1_000_000_000);
        assert_eq!(enabled.compute, saved.compute);
        assert_eq!(enabled.encode, saved.encode);
        assert_eq!(enabled.decode, saved.decode);
        // A zero or omitted request does not disable GPU support.
        assert!(s.draft(&vm_id, true, 0, &original).unwrap().enabled);
        assert!(
            s.draft(&vm_id, true, 0, &vec![String::new(); 12])
                .unwrap()
                .enabled
        );
        assert!(!s.draft(&vm_id, false, 0, &edited).unwrap().enabled);
        assert_eq!(s.target(&vm_id).unwrap(), saved);
        assert_eq!(s.committed.documents[&vm_id].source, source);
        assert_eq!(s.committed.documents[&other.vm_id].target, other);
        assert_eq!(s.view.vm(&vm_id).unwrap().power, Power::Off);
        assert!(s.view.vm(&vm_id).unwrap().gpus.is_empty());
        s.view.needs_readback = true;
        assert!(s.memory_values(&vm_id, 0, &original, 10).is_ok());
        assert!(s.view.eligibility(&enabled).is_err());
    }
}

#[test]
fn new_vm_can_be_drafted_but_proposal_never_becomes_execution_authority() {
    let mut s = State::default();
    let mut i = inventory();
    i.enrolled.clear();
    s.refresh(i);
    let vm_id = id(&s).to_owned();
    let target = s
        .draft(&vm_id, true, 0, &values("10", "50", "100"))
        .unwrap();
    assert!(!s.enrolled(&target));
    assert!(s.view.configuration.targets.is_empty());
    assert!(!s.target(&vm_id).unwrap().enabled);
    assert!(
        s.view
            .eligibility(&target)
            .unwrap_err()
            .contains("not enrolled")
    );
    assert!(s.view.inventory.as_ref().unwrap().enrolled.is_empty());
    s.view.draft = Some(target.clone());
    s.refresh(inventory());
    assert_eq!(s.view.draft, Some(target.clone()));
    assert!(s.enrolled(&target));
    assert!(s.view.configuration.targets.is_empty());
    assert!(s.view.eligibility(&target).is_ok());
}

#[test]
fn saved_intent_remains_distinct_from_enrollment_observation_and_draft() {
    let mut s = state();
    assert!(s.view.configuration.targets.is_empty());
    assert!(s.view.saved_desired_text(id(&s)).contains("No saved"));
    let mut candidate = s.target(id(&s)).unwrap();
    candidate.enabled = false;
    let configuration = Configuration {
        schema: 2,
        targets: vec![candidate.clone()],
    };
    s.view.configuration = configuration;
    assert!(!s.target(id(&s)).unwrap().enabled);
    assert!(s.view.inventory.as_ref().unwrap().enrolled[0].enabled);
    s.view.draft = Some(
        s.draft(id(&s), true, 0, &values("10", "50", "100"))
            .unwrap(),
    );
    s.refresh(inventory());
    assert_eq!(s.view.configuration.targets[0], candidate);
    assert!(s.view.draft.as_ref().unwrap().enabled);
    assert!(s.view.draft.is_some());
}

#[test]
fn drafts_validate_raw_vram_and_never_mutate_observation_or_enrollment() {
    let s = state();
    let target = s
        .draft(id(&s), true, 0, &values("10", "50", "100"))
        .unwrap();
    assert_eq!(target.vram.unwrap().optimal, 50);
    assert!(s.view.vm(id(&s)).unwrap().gpus.is_empty());
    assert!(s.view.configuration.targets.is_empty());
    assert!(
        s.draft(id(&s), false, 0, &values("", "", ""))
            .unwrap()
            .vram
            .is_none()
    );
    assert_eq!(
        allocation_values(&s.target(id(&s)).unwrap()),
        values("", "", "")
    );
}

#[test]
fn vram_draft_above_one_gb_metadata_does_not_change_saved_state() {
    let mut s = state();
    let gpu = &mut s.view.inventory.as_mut().unwrap().discovery.gpus[0];
    gpu.vram = Allocation {
        minimum: 0,
        optimal: 1_000_000_000,
        maximum: 1_000_000_000,
    }
    .into();
    gpu.physical_vram_bytes = Some(16_000_000_000);
    let saved = s.target(id(&s)).unwrap();
    let raw = s
        .memory_values(id(&s), 0, &vec![String::new(); 12], 160)
        .unwrap();
    let draft = s.draft(id(&s), true, 0, &raw).unwrap();
    assert_eq!(draft.vram.as_ref().unwrap().optimal, 16_000_000_000);
    assert_eq!(draft.vram.as_ref().unwrap().maximum, 16_000_000_000);
    assert_eq!(s.target(id(&s)).unwrap(), saved);
    assert!(s.view.vm(id(&s)).unwrap().gpus.is_empty());
    assert!(s.view.configuration.targets.is_empty());
}

#[test]
fn incomplete_overflow_out_of_bounds_and_unsupported_fields_are_blocked() {
    let s = state();
    for v in [
        values("10", "", "100"),
        values("-1", "50", "100"),
        values("10", "50", "18446744073709551616"),
        values("10", "101", "100"),
    ] {
        assert!(s.draft(id(&s), true, 0, &v).is_err(), "{v:?}");
    }
    let mut v = values("10", "50", "100");
    v[4] = "50".into();
    assert!(
        s.draft(id(&s), true, 0, &v)
            .unwrap_err()
            .contains("Compute")
    );
    assert!(s.draft(id(&s), true, 0, &v[..3]).is_err());
}

#[test]
fn supported_compute_draft_survives_unavailable_vram_and_preserves_observations() {
    use hyper_gpu_support::model::ResourceCapability;
    let mut s = state();
    let gpu = &mut s.view.inventory.as_mut().unwrap().discovery.gpus[0];
    gpu.vram = ResourceCapability::Unavailable("VRAM range unreadable".into());
    gpu.compute = ResourceCapability::Supported(Allocation {
        minimum: 10,
        optimal: 50,
        maximum: 100,
    });
    let mut raw = values("", "", "");
    raw[3..6].clone_from_slice(&["10".into(), "50".into(), "100".into()]);
    let target = s.draft(id(&s), true, 0, &raw).unwrap();
    assert_eq!(allocation_values(&target), raw);
    assert_eq!(target.compute.as_ref().unwrap().optimal, 50);
    assert!(target.vram.is_none());
    assert!(s.view.vm(id(&s)).unwrap().compute.is_none());
    assert!(
        s.view.inventory.as_ref().unwrap().enrolled[0]
            .compute
            .is_none()
    );
    let mut invalid = raw.clone();
    invalid[4] = "101".into();
    assert!(s.draft(id(&s), true, 0, &invalid).is_err());
    invalid[3] = "".into();
    assert!(s.draft(id(&s), true, 0, &invalid).is_err());
    raw[6..9].clone_from_slice(&["10".into(), "50".into(), "100".into()]);
    assert!(
        s.draft(id(&s), true, 0, &raw)
            .unwrap_err()
            .contains("Encode")
    );
    assert!(s.draft(id(&s), false, 0, &raw).is_ok());
    raw[6..9].fill(String::new());
    assert!(s.draft(id(&s), true, 0, &raw).is_ok());
}

#[test]
fn duplicate_gpu_names_do_not_grant_a_different_pair() {
    let mut s = state();
    let i = s.view.inventory.as_mut().unwrap();
    let mut other = i.discovery.gpus[0].clone();
    other.interface = r"\\?\PCI#VEN_10DE&DEV_2D05#other\GPUPARAV".into();
    i.discovery.gpus.push(other);
    assert!(
        s.draft(id(&s), true, 1, &values("", "", ""))
            .unwrap_err()
            .contains("not enrolled")
    );
    assert!(s.draft(id(&s), true, -1, &values("", "", "")).is_err());
}

#[test]
fn failed_reads_and_removed_enrollment_block_preview_without_losing_drafts() {
    let mut s = state();
    let original = s
        .draft(id(&s), true, 0, &values("10", "50", "100"))
        .unwrap();
    s.view.draft = Some(original.clone());
    s.view.needs_readback = true;
    assert!(s.draft(id(&s), true, 0, &values("10", "50", "100")).is_ok());
    assert!(s.view.eligibility(&original).is_err());
    let mut next = inventory();
    next.enrolled.clear();
    s.refresh(next);
    assert_eq!(s.view.draft, Some(original));
    assert!(
        s.draft(id(&s), true, 0, &values("10", "50", "100"))
            .is_err()
    );
}

#[test]
fn pending_journal_blocks_new_reviews_without_a_save_record_and_retains_the_draft() {
    for verification_only in [false, true] {
        let mut s = state();
        let vm_id = id(&s).to_owned();
        let draft = s
            .draft(&vm_id, true, 0, &values("10", "50", "100"))
            .unwrap();
        s.view.draft = Some(draft.clone());
        let mut next = inventory();
        let settings = next.discovery.vms[0].settings.clone();
        next.managed.insert(
            vm_id.clone(),
            Some(hyper_gpu_support::workflow::Journal {
                schema: 1,
                vm_id: vm_id.clone(),
                gpu_interface: draft.gpu_interface.clone(),
                original: settings.clone(),
                applied: settings,
                prepared: None,
                pending: true,
                restore_power: Power::Off,
                verification_only,
                last_verified: None,
                latest_check: None,
                preparation_started: None,
                check_restored: false,
                settings_released: false,
            }),
        );
        s.refresh(next.clone());
        assert!(s.recovery.is_none());
        let actions = s.selected_actions(0, &allocation_values(&draft));
        assert!(actions.recovery);
        assert!(
            actions.forget_allowed,
            "profile deletion remains separately reviewable"
        );
        for action in ["verify", "repair", "remove-gpu"] {
            assert!(actions.reason(action).unwrap().contains("Review recovery"));
        }
        assert!(
            s.action_recovery_message()
                .unwrap()
                .contains("Renamable VM")
        );
        assert!(
            s.editor_eligibility(&vm_id)
                .unwrap_err()
                .contains("recovery")
        );
        assert!(
            s.draft(&vm_id, true, 0, &values("10", "50", "100"))
                .is_err()
        );
        assert!(
            s.accept_review(
                review_plan(),
                &draft,
                hyper_gpu_support::configuration_store::Revision::Missing,
            )
            .is_err()
        );
        assert!(s.approved_plan.is_none());
        assert_eq!(s.view.draft, Some(draft));
        assert!(
            s.view.inventory.as_ref().unwrap().managed[&vm_id]
                .as_ref()
                .unwrap()
                .pending
        );
        next.managed
            .get_mut(&vm_id)
            .unwrap()
            .as_mut()
            .unwrap()
            .pending = false;
        s.refresh(next);
        assert!(s.editor_eligibility(&vm_id).is_ok());
    }
}

#[test]
fn disappearing_vm_keeps_its_draft_identity_until_discard() {
    let mut s = state();
    s.view.draft = Some(s.target(id(&s)).unwrap());
    let selected = s.view.selected.clone();
    let mut next = inventory();
    next.discovery.vms[0].vm_id = "12345678-1234-1234-1234-000000000002".into();
    s.refresh(next);
    assert_eq!(s.view.selected, selected);
    assert!(s.view.vm(id(&s)).is_none());
    assert!(s.draft(id(&s), true, 0, &values("", "", "")).is_err());
    s.view.discard();
    s.refresh(inventory());
    assert!(s.view.vm(id(&s)).is_some());
}

#[test]
fn changed_provider_bounds_do_not_reject_existing_vram_byte_requests() {
    let mut s = state();
    s.view.configuration.targets = s.view.inventory.as_ref().unwrap().enrolled.clone();
    s.view.configuration.targets[0].vram = Some(Allocation {
        minimum: 10,
        optimal: 150,
        maximum: 200,
    });
    assert!(s.editor_eligibility(id(&s)).is_ok());
    let old = allocation_values(&s.target(id(&s)).unwrap());
    assert!(s.draft(id(&s), true, 0, &old).is_ok());
    assert!(s.draft(id(&s), true, 0, &values("10", "50", "100")).is_ok());
    assert!(s.draft(id(&s), false, 0, &values("", "", "")).is_ok());
}

#[test]
fn malformed_committed_configuration_cannot_echo_secrets_into_validation() {
    let mut s = state();
    let vm_id = id(&s).to_owned();
    s.committed.insert(
        &format!("{vm_id}.toml"),
        Ok("password='private-parser-secret'".into()),
    );
    let error = s.editor_eligibility(&vm_id).unwrap_err();
    assert!(error.contains("Committed configuration"));
    assert!(!error.contains("private-parser-secret"));
}
