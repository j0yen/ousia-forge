/// Build an OWL 2 DL ontology from an OntologySpec and write it as OWL/XML.
///
/// The output format is OWL/XML (not RDF/XML), but it is the format
/// that horned-owl's OWX writer produces and is well-formed XML that
/// can be re-loaded by horned-owl. AC1 requires re-loadability; the
/// format satisfies that.
use std::path::Path;

use horned_owl::curie::PrefixMapping;
use horned_owl::model::*;
use horned_owl::ontology::component_mapped::RcComponentMappedOntology;

use crate::error::ForgeError;
use crate::model::{ClassDef, OntologySpec};

const WO_NS: &str = "https://w3id.org/world-ontology/";
const BFO_NS: &str = "http://purl.obolibrary.org/obo/";
const BFO_IMPORT: &str = "http://purl.obolibrary.org/obo/bfo/2020/bfo.owl";

/// Standard RDFS/OWL annotation property IRIs
const RDFS_LABEL: &str = "http://www.w3.org/2000/01/rdf-schema#label";
const RDFS_COMMENT: &str = "http://www.w3.org/2000/01/rdf-schema#comment";

/// Custom annotation property IRIs (§4.4)
const AP_ARISTOTELIAN: &str = "https://w3id.org/world-ontology/aristotelianDefinition";
const AP_PHILOSOPHICAL: &str = "https://w3id.org/world-ontology/philosophicalGrounding";
const AP_AI_GUIDANCE: &str = "https://w3id.org/world-ontology/aiGuidance";
const AP_MORAL: &str = "https://w3id.org/world-ontology/moralSignificance";
const AP_DESIGN: &str = "https://w3id.org/world-ontology/designRationale";
const AP_EXAMPLE: &str = "https://w3id.org/world-ontology/exampleInstance";
const AP_DOMAIN_MODULE: &str = "https://w3id.org/world-ontology/domainModule";

pub fn build_ontology(spec: &OntologySpec, out: &Path) -> Result<(), ForgeError> {
    let b = Build::new_rc();
    let mut ont = RcComponentMappedOntology::new_rc();

    // Set ontology IRI
    let ont_iri = spec
        .meta
        .iri
        .as_deref()
        .unwrap_or("https://w3id.org/world-ontology");
    let ont_id = OntologyID {
        iri: Some(b.iri(ont_iri)),
        viri: None,
    };
    ont.insert(ont_id);

    // Declare owl:imports for BFO 2020
    ont.insert(Import(b.iri(BFO_IMPORT)));

    // Declare the 7 custom annotation properties (§4.4)
    let custom_aps = [
        AP_ARISTOTELIAN,
        AP_PHILOSOPHICAL,
        AP_AI_GUIDANCE,
        AP_MORAL,
        AP_DESIGN,
        AP_EXAMPLE,
        AP_DOMAIN_MODULE,
    ];
    for ap_iri in &custom_aps {
        ont.insert(DeclareAnnotationProperty(b.annotation_property(*ap_iri)));
    }

    // Declare additional annotation properties from the spec meta
    if let Some(aps) = &spec.meta.annotation_properties {
        for ap_def in aps {
            let ap = b.annotation_property(ap_def.iri.as_str());
            ont.insert(DeclareAnnotationProperty(ap.clone()));
            let ann = Annotation {
                ap: b.annotation_property(RDFS_LABEL),
                av: AnnotationValue::Literal(Literal::Simple {
                    literal: ap_def.label.clone(),
                }),
            };
            ont.insert(AnnotationAssertion {
                subject: AnnotationSubject::IRI(ap.0.clone()),
                ann,
            });
        }
    }

    // Declare object properties from spec meta
    if let Some(ops) = &spec.meta.object_properties {
        for op_def in ops {
            let op = b.object_property(op_def.iri.as_str());
            ont.insert(DeclareObjectProperty(op.clone()));
            let label_ann = Annotation {
                ap: b.annotation_property(RDFS_LABEL),
                av: AnnotationValue::Literal(Literal::Simple {
                    literal: op_def.label.clone(),
                }),
            };
            ont.insert(AnnotationAssertion {
                subject: AnnotationSubject::IRI(op.0.clone()),
                ann: label_ann,
            });
            if let Some(inv_iri) = &op_def.inverse_of {
                let inv = b.object_property(resolve_iri(inv_iri).as_str());
                ont.insert(InverseObjectProperties(op.clone(), inv));
            }
            if op_def.transitive.unwrap_or(false) {
                ont.insert(TransitiveObjectProperty(
                    ObjectPropertyExpression::ObjectProperty(op.clone()),
                ));
            }
        }
    }

    // Process classes in stable order (already sorted by parser)
    for cls in &spec.classes {
        emit_class(&b, &mut ont, cls)?;
    }

    // Write OWL/XML
    let mut mapping = PrefixMapping::default();
    mapping.add_prefix("owl", "http://www.w3.org/2002/07/owl#").ok();
    mapping.add_prefix("rdf", "http://www.w3.org/1999/02/22-rdf-syntax-ns#").ok();
    mapping.add_prefix("rdfs", "http://www.w3.org/2000/01/rdf-schema#").ok();
    mapping.add_prefix("xsd", "http://www.w3.org/2001/XMLSchema#").ok();
    mapping.add_prefix("wo", WO_NS).ok();
    mapping.add_prefix("obo", BFO_NS).ok();

    let mut buf = Vec::new();
    horned_owl::io::owx::writer::write(&mut buf, &ont, Some(&mapping))
        .map_err(|e| ForgeError::Spec(format!("OWL/XML write error: {}", e)))?;

    // Write to file — deterministic because we sort classes and the
    // OWX writer uses BTreeSet internally
    std::fs::write(out, &buf)?;
    Ok(())
}

fn emit_class(
    b: &Build<RcStr>,
    ont: &mut RcComponentMappedOntology,
    cls: &ClassDef,
) -> Result<(), ForgeError> {
    let cls_iri = resolve_iri(&cls.iri);
    let cls_obj = b.class(cls_iri.as_str());

    // Declare
    ont.insert(DeclareClass(cls_obj.clone()));

    // rdfs:label
    ont.insert(AnnotationAssertion {
        subject: AnnotationSubject::IRI(cls_obj.0.clone()),
        ann: Annotation {
            ap: b.annotation_property(RDFS_LABEL),
            av: AnnotationValue::Literal(Literal::Simple {
                literal: cls.label.clone(),
            }),
        },
    });

    // rdfs:comment (definition)
    if let Some(def) = &cls.definition {
        ont.insert(AnnotationAssertion {
            subject: AnnotationSubject::IRI(cls_obj.0.clone()),
            ann: Annotation {
                ap: b.annotation_property(RDFS_COMMENT),
                av: AnnotationValue::Literal(Literal::Simple {
                    literal: def.clone(),
                }),
            },
        });
    }

    // Custom annotations
    for (key, val) in &cls.annotations {
        let ap_iri = annotation_key_to_iri(key);
        ont.insert(AnnotationAssertion {
            subject: AnnotationSubject::IRI(cls_obj.0.clone()),
            ann: Annotation {
                ap: b.annotation_property(ap_iri.as_str()),
                av: AnnotationValue::Literal(Literal::Simple {
                    literal: val.clone(),
                }),
            },
        });
    }

    // SubClassOf (parent in is_a hierarchy)
    if let Some(parent_str) = &cls.parent {
        let parent_ce = ClassExpression::Class(b.class(resolve_iri(parent_str).as_str()));
        ont.insert(SubClassOf {
            sup: parent_ce,
            sub: ClassExpression::Class(cls_obj.clone()),
        });
    }

    // SubClassOf all-some restrictions
    for restriction_str in &cls.subclass_of {
        let restriction_ce = parse_class_expression(b, restriction_str)?;
        ont.insert(SubClassOf {
            sup: restriction_ce,
            sub: ClassExpression::Class(cls_obj.clone()),
        });
    }

    // OWL EquivalentClasses (defined class)
    if let Some(equiv_str) = &cls.equivalent_to {
        let equiv_ce = parse_class_expression(b, equiv_str)?;
        ont.insert(EquivalentClasses(vec![
            ClassExpression::Class(cls_obj.clone()),
            equiv_ce,
        ]));
    }

    Ok(())
}

/// Parse a class expression string of the forms:
///   "PropertyName some ClassName"
///   "PropertyName some (CE1 and CE2)"
///   "CE1 and CE2"
///   "ClassName"
fn parse_class_expression(
    b: &Build<RcStr>,
    expr: &str,
) -> Result<ClassExpression<RcStr>, ForgeError> {
    let expr = expr.trim();

    // Try "prop some CE"
    if let Some((prop_part, rest)) = split_keyword(expr, " some ") {
        let ope = ObjectPropertyExpression::ObjectProperty(
            b.object_property(resolve_iri(prop_part.trim()).as_str()),
        );
        let bce = parse_class_expression(b, rest.trim())?;
        return Ok(ClassExpression::ObjectSomeValuesFrom {
            ope,
            bce: Box::new(bce),
        });
    }

    // Try "CE1 and CE2 and ..."  (intersection)
    if expr.contains(" and ") {
        // Only split on bare " and " (not inside parens)
        let parts = split_conjunction(expr);
        if parts.len() > 1 {
            let mut ces = Vec::new();
            for p in parts {
                ces.push(parse_class_expression(b, p.trim())?);
            }
            return Ok(ClassExpression::ObjectIntersectionOf(ces));
        }
    }

    // Strip outer parens
    let expr = strip_parens(expr);

    // Recurse if we stripped anything
    if expr != expr.trim() {
        return parse_class_expression(b, expr);
    }

    // Named class
    let iri = resolve_iri(expr);
    Ok(ClassExpression::Class(b.class(iri.as_str())))
}

/// Split on a keyword only at depth 0 (not inside parentheses)
fn split_keyword<'a>(s: &'a str, kw: &str) -> Option<(&'a str, &'a str)> {
    let mut depth = 0i32;
    let kw_bytes = kw.as_bytes();
    let s_bytes = s.as_bytes();
    let kw_len = kw_bytes.len();

    for i in 0..s_bytes.len() {
        match s_bytes[i] {
            b'(' => depth += 1,
            b')' => depth -= 1,
            _ => {}
        }
        if depth == 0 && s_bytes[i..].starts_with(kw_bytes) {
            return Some((&s[..i], &s[i + kw_len..]));
        }
    }
    None
}

/// Split "A and B and C" at depth-0 " and " boundaries
fn split_conjunction(s: &str) -> Vec<&str> {
    let mut result = Vec::new();
    let mut depth = 0i32;
    let mut last = 0;
    let bytes = s.as_bytes();
    let kw = b" and ";
    let kw_len = kw.len();

    for i in 0..bytes.len() {
        match bytes[i] {
            b'(' => depth += 1,
            b')' => depth -= 1,
            _ => {}
        }
        if depth == 0 && bytes[i..].starts_with(kw) {
            result.push(&s[last..i]);
            last = i + kw_len;
        }
    }
    result.push(&s[last..]);
    result
}

fn strip_parens(s: &str) -> &str {
    let s = s.trim();
    if s.starts_with('(') && s.ends_with(')') {
        &s[1..s.len() - 1]
    } else {
        s
    }
}

/// Resolve short-form IRI references to full IRIs.
/// Supports:
///   BFO:classname    → http://purl.obolibrary.org/obo/BFO_classname
///   WO:classname     → https://w3id.org/world-ontology/classname
///   PascalCase name  → https://w3id.org/world-ontology/name  (if no colon)
///   camelCase prop   → https://w3id.org/world-ontology/name  (if no colon)
///   Full IRI         → as-is
fn resolve_iri(s: &str) -> String {
    let s = s.trim();

    if s.starts_with("http://") || s.starts_with("https://") {
        return s.to_string();
    }

    if let Some(rest) = s.strip_prefix("BFO:") {
        // BFO IRI pattern: http://purl.obolibrary.org/obo/BFO_XXXXXXXX
        // We use the short name directly
        return format!("{}BFO_{}", BFO_NS, rest);
    }

    if let Some(rest) = s.strip_prefix("WO:") {
        return format!("{}{}", WO_NS, rest);
    }

    if let Some(rest) = s.strip_prefix("obo:") {
        return format!("{}{}", BFO_NS, rest);
    }

    // No prefix: treat as a World Ontology name
    format!("{}{}", WO_NS, s)
}

/// Map annotation key names to full annotation property IRIs
fn annotation_key_to_iri(key: &str) -> String {
    match key {
        "aristotelianDefinition" => AP_ARISTOTELIAN.to_string(),
        "philosophicalGrounding" => AP_PHILOSOPHICAL.to_string(),
        "aiGuidance" => AP_AI_GUIDANCE.to_string(),
        "moralSignificance" => AP_MORAL.to_string(),
        "designRationale" => AP_DESIGN.to_string(),
        "exampleInstance" => AP_EXAMPLE.to_string(),
        "domainModule" => AP_DOMAIN_MODULE.to_string(),
        other => {
            if other.starts_with("http") {
                other.to_string()
            } else {
                format!("{}{}", WO_NS, other)
            }
        }
    }
}

/// Reload the output file with horned-owl to verify round-trip (used in tests)
#[allow(dead_code)]
pub fn reload_owl_xml(path: &Path) -> Result<(), ForgeError> {
    use std::fs::File;
    use std::io::BufReader;
    let f = File::open(path)?;
    let mut reader = BufReader::new(f);
    let (_, _) = horned_owl::io::owx::reader::read(&mut reader, Default::default())
        .map_err(|e| ForgeError::Spec(format!("reload error: {}", e)))?;
    Ok(())
}
