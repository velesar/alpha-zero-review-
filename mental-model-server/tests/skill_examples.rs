//! The update_viewpoint examples in the viewpoint skills must be accepted by
//! the model exactly as written, so agents copying them produce valid data.

use mental_model_server::model::MentalModel;
use std::path::PathBuf;

/// Skills whose data is parsed into a typed model section
const TYPED: &[(&str, &str)] = &[
    ("vp-f01-tech-stack", "VP-F01"),
    ("vp-f02-structure", "VP-F02"),
    ("vp-f03-build-deploy", "VP-F03"),
    ("vp-s01-module-hierarchy", "VP-S01"),
    ("vp-s02-layer-architecture", "VP-S02"),
    ("vp-s03-domain-model", "VP-S03"),
    ("vp-s04-entity-model", "VP-S04"),
    ("vp-s05-interface-surface", "VP-S05"),
    ("vp-s06-dependency-graph", "VP-S06"),
];

/// The fenced block right after "Call `mental-model/update_viewpoint` with:"
fn example(skill: &str) -> (String, serde_yaml::Value) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../skills")
        .join(skill)
        .join("SKILL.md");
    let text = std::fs::read_to_string(&path).unwrap();
    let marker = "Call `mental-model/update_viewpoint` with:";
    let start = text
        .find(marker)
        .unwrap_or_else(|| panic!("{skill}: no update_viewpoint example"));
    let rest = &text[start + marker.len()..];
    let fence = rest.find("```").unwrap();
    let body_start = rest[fence..].find('\n').unwrap() + fence + 1;
    let body_end = body_start + rest[body_start..].find("```").unwrap();
    let body = &rest[body_start..body_end];
    let parsed: serde_yaml::Value = serde_yaml::from_str(body)
        .unwrap_or_else(|e| panic!("{skill}: example is not valid YAML/JSON: {e}"));
    let viewpoint = parsed["viewpoint"].as_str().unwrap_or_default().to_string();
    (viewpoint, parsed["data"].clone())
}

#[test]
fn skill_update_examples_match_model_schemas() {
    let mut failures = Vec::new();
    for (skill, expected_vp) in TYPED {
        let (viewpoint, data) = example(skill);
        if viewpoint != *expected_vp {
            failures.push(format!("{skill}: example viewpoint is {viewpoint:?}"));
            continue;
        }
        let data: serde_json::Value = serde_json::to_value(&data).unwrap();
        let mut model = MentalModel::default();
        match model.apply_viewpoint(&viewpoint, data) {
            Err(e) => failures.push(format!("{skill}: {e}")),
            Ok(unused) => {
                let section = match viewpoint.as_str() {
                    "VP-F01" => serde_json::to_value(&model.tech_stack),
                    "VP-F02" => serde_json::to_value(&model.structure),
                    "VP-F03" => serde_json::to_value(&model.build_deploy),
                    "VP-S01" => serde_json::to_value(&model.module_hierarchy),
                    "VP-S02" => serde_json::to_value(&model.architecture),
                    "VP-S03" => serde_json::to_value(&model.domain_model),
                    "VP-S04" => serde_json::to_value(&model.entity_model),
                    "VP-S05" => serde_json::to_value(&model.interface_surface),
                    "VP-S06" => serde_json::to_value(&model.hotspots),
                    _ => unreachable!(),
                }
                .unwrap();
                let empty = section.as_object().is_none_or(|o| o.is_empty());
                if empty {
                    failures.push(format!(
                        "{skill}: example fills nothing in the typed section (unused: {unused:?})"
                    ));
                }
            }
        }
    }
    assert!(failures.is_empty(), "\n{}", failures.join("\n"));
}

/// Fenced JSON blocks in a skill that contain findings
fn finding_examples(skill: &str) -> Vec<serde_json::Value> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../skills")
        .join(skill)
        .join("SKILL.md");
    let text = std::fs::read_to_string(&path).unwrap();
    let mut blocks = Vec::new();
    let mut rest = text.as_str();
    while let Some(start) = rest.find("```json\n") {
        let body = &rest[start + "```json\n".len()..];
        let end = body.find("```").unwrap();
        if body[..end].contains("\"base_severity\"") {
            let value: serde_json::Value = serde_json::from_str(&body[..end])
                .unwrap_or_else(|e| panic!("{skill}: invalid JSON example: {e}"));
            blocks.push(value);
        }
        rest = &body[end + 3..];
    }
    blocks
}

#[test]
fn skill_finding_examples_are_valid_inputs() {
    use mental_model_server::server::{AddFindingInput, AddFindingsInput};

    let categories: std::collections::HashMap<String, serde_yaml::Value> = serde_yaml::from_str(
        &std::fs::read_to_string(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../methodology_kb/taxonomies/categories.yaml"),
        )
        .unwrap(),
    )
    .unwrap();

    let mut failures = Vec::new();
    let mut checked = 0;
    for skill in [
        "vp-q01-security",
        "vp-q02-performance",
        "vp-q03-testability",
        "vp-q04-code-style",
        "vp-q05-documentation",
    ] {
        for block in finding_examples(skill) {
            let findings: Vec<(String, String)> = if block.get("findings").is_some() {
                match serde_json::from_value::<AddFindingsInput>(block) {
                    Ok(input) => input
                        .findings
                        .into_iter()
                        .map(|f| (f.base_severity, f.category))
                        .collect(),
                    Err(e) => {
                        failures.push(format!("{skill}: add_findings example: {e}"));
                        continue;
                    }
                }
            } else {
                match serde_json::from_value::<AddFindingInput>(block) {
                    Ok(f) => vec![(f.base_severity, f.category)],
                    Err(e) => {
                        failures.push(format!("{skill}: add_finding example: {e}"));
                        continue;
                    }
                }
            };
            for (severity, category) in findings {
                checked += 1;
                if mental_model_server::ops::parse_severity(&severity).is_none() {
                    failures.push(format!("{skill}: invalid severity {severity}"));
                }
                if !categories.contains_key(&category) {
                    failures.push(format!(
                        "{skill}: category '{category}' is not in methodology_kb/taxonomies/categories.yaml"
                    ));
                }
            }
        }
    }
    assert!(checked > 0, "no finding examples found");
    assert!(failures.is_empty(), "\n{}", failures.join("\n"));
}
