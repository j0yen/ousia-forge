use std::fs;
use std::path::Path;

use crate::error::ForgeError;
use crate::model::{DomainSpec, OntologyMeta, OntologySpec};

pub fn load_spec(dir: &Path) -> Result<OntologySpec, ForgeError> {
    if !dir.is_dir() {
        return Err(ForgeError::Spec(format!(
            "spec path is not a directory: {}",
            dir.display()
        )));
    }

    // Load ontology.toml if present
    let meta_path = dir.join("ontology.toml");
    let meta: OntologyMeta = if meta_path.exists() {
        let s = fs::read_to_string(&meta_path)?;
        toml::from_str(&s).map_err(|e| ForgeError::Toml {
            file: meta_path.display().to_string(),
            source: e,
        })?
    } else {
        OntologyMeta::default()
    };

    // Load all other *.toml files as domain specs
    let mut entries: Vec<_> = fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .filter(|e| {
            let name = e.file_name();
            let name = name.to_string_lossy();
            name.ends_with(".toml") && name != "ontology.toml"
        })
        .collect();

    // Sort for determinism
    entries.sort_by_key(|e| e.file_name());

    let mut classes = Vec::new();
    for entry in entries {
        let path = entry.path();
        let s = fs::read_to_string(&path)?;
        let domain: DomainSpec = toml::from_str(&s).map_err(|e| ForgeError::Toml {
            file: path.display().to_string(),
            source: e,
        })?;
        if let Some(cls) = domain.classes {
            classes.extend(cls);
        }
    }

    Ok(OntologySpec { meta, classes })
}
