use crate::error::ForgeError;
use crate::model::OntologySpec;

/// Validates the spec:
/// - Every class must have exactly 1 parent (zero or ≥2 parents → error)
pub fn check_spec(spec: &OntologySpec) -> Result<(), ForgeError> {
    let mut errors: Vec<String> = Vec::new();

    for cls in &spec.classes {
        match &cls.parent {
            None => {
                errors.push(format!(
                    "class '{}' has zero asserted parents (single-inheritance required)",
                    cls.iri
                ));
            }
            Some(_) => {
                // exactly one parent — OK
            }
        }
    }

    if !errors.is_empty() {
        return Err(ForgeError::Check(errors.join("; ")));
    }

    Ok(())
}
