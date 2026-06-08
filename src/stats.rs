/// Parse a built OWL/XML file and print class/property/axiom statistics.
use std::io::BufReader;
use std::path::Path;

use horned_owl::model::*;
use horned_owl::ontology::set::SetOntology;

use crate::error::ForgeError;

pub fn print_stats(out: &Path) -> Result<(), ForgeError> {
    let f = std::fs::File::open(out)?;
    let mut reader = BufReader::new(f);
    let (ont, _): (SetOntology<RcStr>, _) =
        horned_owl::io::owx::reader::read(&mut reader, Default::default())
            .map_err(|e| ForgeError::Spec(format!("reload error: {}", e)))?;

    let mut named_classes = 0u64;
    let mut object_properties = 0u64;
    let mut defined_classes = 0u64;
    let mut restriction_axioms = 0u64;

    for ac in ont.iter() {
        match &ac.component {
            Component::DeclareClass(_) => named_classes += 1,
            Component::DeclareObjectProperty(_) => object_properties += 1,
            Component::EquivalentClasses(_) => defined_classes += 1,
            Component::SubClassOf(s) => {
                if is_restriction(&s.sup) {
                    restriction_axioms += 1;
                }
            }
            _ => {}
        }
    }

    println!("named-classes: {}", named_classes);
    println!("object-properties: {}", object_properties);
    println!("defined-classes: {}", defined_classes);
    println!("restriction-axioms: {}", restriction_axioms);

    Ok(())
}

fn is_restriction(ce: &ClassExpression<RcStr>) -> bool {
    match ce {
        ClassExpression::ObjectSomeValuesFrom { .. } => true,
        ClassExpression::ObjectAllValuesFrom { .. } => true,
        ClassExpression::ObjectIntersectionOf(parts) => parts.iter().any(is_restriction),
        _ => false,
    }
}
