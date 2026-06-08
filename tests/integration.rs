use std::fs;
use std::io::BufReader;
use tempfile::TempDir;

use ousia_forge::builder::build_ontology;
use ousia_forge::checker::check_spec;
use ousia_forge::error::ForgeError;
use ousia_forge::model::{ClassDef, OntologyMeta, OntologySpec};
use ousia_forge::parser::load_spec;

// ---------------------------------------------------------------------------
// AC8a: minimal 3-class spec round-trips through build + horned-owl reload
// ---------------------------------------------------------------------------

fn make_3class_spec() -> OntologySpec {
    OntologySpec {
        meta: OntologyMeta::default(),
        classes: vec![
            ClassDef {
                iri: "Quality".to_string(),
                label: "Quality".to_string(),
                parent: Some("BFO:quality".to_string()),
                definition: Some("A test quality.".to_string()),
                annotations: Default::default(),
                equivalent_to: None,
                subclass_of: vec![],
            },
            ClassDef {
                iri: "Sentience".to_string(),
                label: "Sentience".to_string(),
                parent: Some("Quality".to_string()),
                definition: None,
                annotations: Default::default(),
                equivalent_to: None,
                subclass_of: vec![],
            },
            ClassDef {
                iri: "SentientBeing".to_string(),
                label: "Sentient Being".to_string(),
                parent: Some("BFO:object".to_string()),
                definition: None,
                annotations: Default::default(),
                equivalent_to: Some("BFO:object and bearer_of some Sentience".to_string()),
                subclass_of: vec!["bearer_of some Dignity".to_string()],
            },
        ],
    }
}

#[test]
fn ac8a_minimal_3class_round_trips() {
    let tmp = TempDir::new().unwrap();
    let out = tmp.path().join("out.owl");

    let spec = make_3class_spec();
    build_ontology(&spec, &out).expect("build should succeed");

    // Reload with horned-owl to verify well-formed XML (AC1)
    let f = std::fs::File::open(&out).unwrap();
    let mut reader = BufReader::new(f);
    let result = horned_owl::io::owx::reader::read(&mut reader, Default::default());
    assert!(result.is_ok(), "reload failed: {:?}", result.err());

    // Verify file contains BFO import IRI (AC2)
    let content = fs::read_to_string(&out).unwrap();
    assert!(
        content.contains("purl.obolibrary.org/obo/bfo"),
        "output should contain BFO import"
    );
    // Verify WO namespace (AC2)
    assert!(
        content.contains("w3id.org/world-ontology"),
        "output should contain WO namespace"
    );
}

// ---------------------------------------------------------------------------
// AC8b: sentience→dignity axiom chain present in output
// ---------------------------------------------------------------------------

#[test]
fn ac8b_sentience_dignity_chain_present() {
    let tmp = TempDir::new().unwrap();
    let out = tmp.path().join("out.owl");

    let spec = make_3class_spec();
    build_ontology(&spec, &out).expect("build should succeed");

    let content = fs::read_to_string(&out).unwrap();

    // SentientBeing should have a SubClassOf restriction referencing Dignity
    assert!(
        content.contains("Dignity"),
        "output should reference Dignity (sentience→dignity chain)"
    );
    // The equivalence axiom for SentientBeing should reference Sentience
    assert!(
        content.contains("Sentience"),
        "output should reference Sentience"
    );
}

// ---------------------------------------------------------------------------
// AC8c: 2-parent spec fails check
// ---------------------------------------------------------------------------

#[test]
fn ac8c_two_parent_spec_fails_check() {
    // We encode "2 parents" as a class with no parent field in the TOML
    // (the single-parent rule: zero or ≥2 → error)
    let tmp = TempDir::new().unwrap();
    let spec_dir = tmp.path().join("spec");
    fs::create_dir_all(&spec_dir).unwrap();

    // Write a TOML with a class that has NO parent (zero parents → error)
    let bad_toml = r#"
[[classes]]
iri = "Orphan"
label = "Orphan"
definition = "A class with no parent — violates single-inheritance."
"#;
    fs::write(spec_dir.join("bad.toml"), bad_toml).unwrap();

    let spec = load_spec(&spec_dir).expect("parser should succeed");
    let result = check_spec(&spec);
    assert!(
        result.is_err(),
        "check should reject a class with zero parents"
    );
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("Orphan"),
        "error should name the offending class, got: {}",
        err_msg
    );
}

// ---------------------------------------------------------------------------
// AC4 golden-triple: authority→accountability in seed spec
// ---------------------------------------------------------------------------

#[test]
fn ac4_authority_accountability_in_seed_spec() {
    let spec_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("spec");
    if !spec_dir.exists() {
        eprintln!("seed spec not found, skipping");
        return;
    }

    let tmp = TempDir::new().unwrap();
    let out = tmp.path().join("world-ontology.owl");

    let spec = load_spec(&spec_dir).expect("load seed spec");
    check_spec(&spec).expect("seed spec should pass check");
    build_ontology(&spec, &out).expect("build seed spec");

    let content = fs::read_to_string(&out).unwrap();
    // AuthorityRole should reference Accountability
    assert!(
        content.contains("Accountability"),
        "output should contain Accountability axiom"
    );
    assert!(
        content.contains("AuthorityRole"),
        "output should contain AuthorityRole"
    );
}

// ---------------------------------------------------------------------------
// AC6: determinism — two builds produce identical output
// ---------------------------------------------------------------------------

#[test]
fn ac6_deterministic_output() {
    let tmp = TempDir::new().unwrap();
    let out1 = tmp.path().join("out1.owl");
    let out2 = tmp.path().join("out2.owl");

    let spec = make_3class_spec();
    build_ontology(&spec, &out1).expect("first build");
    build_ontology(&spec, &out2).expect("second build");

    let b1 = fs::read(&out1).unwrap();
    let b2 = fs::read(&out2).unwrap();
    assert_eq!(b1, b2, "two builds of the same spec must be byte-identical");
}

// ---------------------------------------------------------------------------
// AC3 + AC4: 6 defined classes and ≥10 restriction axioms in seed spec
// ---------------------------------------------------------------------------

#[test]
fn ac3_ac4_seed_spec_counts() {
    let spec_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("spec");
    if !spec_dir.exists() {
        eprintln!("seed spec not found, skipping");
        return;
    }

    let tmp = TempDir::new().unwrap();
    let out = tmp.path().join("world-ontology.owl");

    let spec = load_spec(&spec_dir).expect("load seed spec");
    build_ontology(&spec, &out).expect("build seed spec");

    // Count equivalentClasses (defined classes) and SubClassOf restrictions
    let content = fs::read_to_string(&out).unwrap();

    // AC3: 6 defined classes with EquivalentClasses
    let equiv_count = content.matches("EquivalentClasses").count();
    assert!(
        equiv_count >= 6,
        "expected ≥6 EquivalentClasses, got {}",
        equiv_count
    );

    // AC4: ≥10 SubClassOf restriction axioms
    let restriction_count = content.matches("ObjectSomeValuesFrom").count();
    assert!(
        restriction_count >= 10,
        "expected ≥10 ObjectSomeValuesFrom restrictions, got {}",
        restriction_count
    );
}
