//! Code-owned active set. No resource wording or fallback English belongs here.
use serde::Deserialize;

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
/// Allowed generated argument types; resource metadata cannot add new types.
pub enum ParameterType {
    Text,
    U64,
    Usize,
    I32,
    Message,
    Formatted,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
/// Strict resource declaration, compared with the code-owned parameter contract.
pub struct Placeholder {
    pub name: String,
    #[serde(rename = "type")]
    pub kind: ParameterType,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
/// Exact non-text metadata, owned by developers rather than resource editors.
pub struct Metadata {
    pub screen: String,
    pub state: String,
    pub audience: String,
    pub intent: String,
    pub consequences: String,
    pub severity: String,
    pub space_hint: String,
}

#[derive(Clone, Debug)]
/// Occurrence, producer and native-size constraints independent of resource files.
pub struct Parameter {
    pub declaration: Placeholder,
    pub minimum: usize,
    pub maximum: Option<usize>,
    /// Safe source and preparation rule; never arbitrary adapter or secret prose.
    pub source: &'static str,
    /// Proven maximum for each occurrence, required on bounded native surfaces.
    pub maximum_utf16: Option<usize>,
}

#[derive(Clone, Debug)]
/// Fixed global property or Rust-only parameterised accessor.
pub enum Consumer {
    Fixed,
    Parameterised,
}

#[derive(Clone, Debug)]
/// A required semantic key and its complete generation/validation contract.
pub struct Message {
    pub key: String,
    pub consumer: Consumer,
    pub metadata: Metadata,
    /// Argument order is independent of resource/template order.
    pub parameters: Vec<Parameter>,
    /// Includes the native terminating NUL; absent for advisory layout hints.
    pub native_budget: Option<usize>,
}

impl Message {
    /// The owning feature also determines the required flat TOML filename.
    pub fn feature(&self) -> &str {
        self.key.split('.').next().unwrap_or_default()
    }
    /// Flattening respects Slint's hyphen/underscore identifier equivalence.
    pub fn slint_name(&self) -> String {
        self.key.replace(['.', '_'], "-")
    }
}

fn fixed(
    key: &str,
    screen: &str,
    state: &str,
    intent: &str,
    consequences: &str,
    severity: &str,
    space: &str,
) -> Message {
    Message {
        key: key.into(),
        consumer: Consumer::Fixed,
        metadata: Metadata {
            screen: screen.into(),
            state: state.into(),
            audience: "operator".into(),
            intent: intent.into(),
            consequences: consequences.into(),
            severity: severity.into(),
            space_hint: space.into(),
        },
        parameters: vec![],
        native_budget: None,
    }
}

fn parameter(
    name: &str,
    kind: ParameterType,
    source: &'static str,
    bound: Option<usize>,
) -> Parameter {
    Parameter {
        declaration: Placeholder {
            name: name.into(),
            kind,
        },
        minimum: 1,
        maximum: Some(1),
        source,
        maximum_utf16: bound,
    }
}

/// Representative closed subset for LANG-004. All other mapping rows remain pending.
pub fn active() -> Vec<Message> {
    let mut result = vec![fixed(
        "operations.verify.title",
        "Verify review and operation dialog",
        "Verify selected; title only",
        "Name the reviewed current-setup check.",
        "Title only; never selects an operation or grants approval.",
        "information",
        "Dialog heading or action button; inspect long wording at supported width/scaling.",
    )];
    for key in [
        "common.power.off",
        "common.power.running",
        "common.state.unknown",
    ] {
        result.push(fixed(
            key,
            "Reviewed power presentation",
            key,
            "Display the selected typed power category.",
            "No effects; presentation only.",
            "neutral",
            "Short power label; visual fit is advisory.",
        ));
    }
    let mut consent = fixed(
        "operations.apply.power_consent",
        "Apply review consent",
        "Reviewed power changes require consent",
        "Approve the reviewed lifecycle and disclose its successful final power.",
        "Use the plan's final power; success is conditional, and the checkbox is separate authority.",
        "warning",
        "Wrapped consent text; do not omit the conditional final-state fact.",
    );
    consent.parameters.push(parameter("final_power", ParameterType::Message, "Typed plan.preview.restore_power selects a catalogue submessage; not initial power or arbitrary error prose.", None));
    consent.consumer = Consumer::Parameterised;
    result.push(consent);
    let mut prompt = fixed(
        "credentials.guest_sign_in.message",
        "Native Windows guest credential prompt; explanation dialog body",
        "No saved opt-in credential; explanation precedes input",
        "Identify the guest account and explain optional local storage.",
        "Approved guest steps only; Remember is opt-in for the current user and exact VM.",
        "information",
        "Native message budget is code-owned; explanation and prompt must remain readable.",
    );
    prompt.parameters.push(parameter("vm_name", ParameterType::Text, "Existing guest name filter removes control characters and takes at most 256 Unicode scalar values.", Some(512)));
    prompt.native_budget = Some(1024);
    prompt.consumer = Consumer::Parameterised;
    result.push(prompt);
    for (key, budget) in [
        ("credentials.native.caption", Some(128)),
        ("credentials.explanation.caption", None),
    ] {
        let mut caption = fixed(
            key,
            "Native guest sign-in panels",
            key,
            "Identify the local guest sign-in panel.",
            "No effects; presentation only.",
            "information",
            "Native caption API budget where applicable; visual fit is advisory.",
        );
        caption.native_budget = budget;
        result.push(caption);
    }
    let mut wrapper = fixed(
        "credentials.explanation.wrapper",
        "Guest sign-in explanation MessageBox",
        "Before native credential input",
        "Explain how to proceed or cancel.",
        "Cancel returns to the application without starting guest work.",
        "information",
        "Wrapped explanation; MessageBox does not inherit Credential UI's message budget.",
    );
    wrapper.parameters.push(parameter(
        "guest_message",
        ParameterType::Message,
        "Catalogue guest_sign_in.message composed with the existing filtered guest name.",
        None,
    ));
    wrapper.consumer = Consumer::Parameterised;
    result.push(wrapper);
    result
}
