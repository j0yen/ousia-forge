use indexmap::IndexMap;
use serde::Deserialize;

/// Top-level ontology.toml
#[derive(Debug, Deserialize, Default)]
pub struct OntologyMeta {
    pub iri: Option<String>,
    pub version_iri: Option<String>,
    pub imports: Option<Vec<String>>,
    pub annotation_properties: Option<Vec<AnnotationPropDef>>,
    pub object_properties: Option<Vec<ObjectPropDef>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AnnotationPropDef {
    pub iri: String,
    pub label: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ObjectPropDef {
    pub iri: String,
    pub label: String,
    pub inverse_of: Option<String>,
    pub transitive: Option<bool>,
    pub domain: Option<String>,
    pub range: Option<String>,
}

/// A domain TOML file (e.g. qualities.toml)
#[derive(Debug, Deserialize, Default)]
pub struct DomainSpec {
    pub domain: Option<String>,
    pub classes: Option<Vec<ClassDef>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ClassDef {
    /// IRI suffix (PascalCase). Full IRI = WO_NS + suffix.
    pub iri: String,
    pub label: String,
    /// BFO parent IRI (short form: "BFO:object" or "WO:SentientBeing")
    pub parent: Option<String>,
    pub definition: Option<String>,
    #[serde(default)]
    pub annotations: IndexMap<String, String>,
    /// OWL equivalence axiom (defined class). LHS is this class.
    pub equivalent_to: Option<String>,
    /// SubClassOf all-some restrictions, e.g. "bearer_of some Dignity"
    #[serde(default)]
    pub subclass_of: Vec<String>,
}

/// The fully loaded spec ready for building
#[derive(Debug, Default)]
pub struct OntologySpec {
    pub meta: OntologyMeta,
    pub classes: Vec<ClassDef>,
}
