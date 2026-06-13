# ousia-forge

OWL 2 DL world ontology builder from declarative TOML spec.

## TL;DR

`ousia-forge` compiles a directory of human-auditable TOML files into
`world-ontology.owl` (OWL 2 DL / RDF/XML, importing BFO 2020). It is the gate
tool for the full ousia toolchain — without the `.owl` artifact, downstream
tools (reason, sparql, guard, mcp) have nothing to operate on.

The spec is organized into eight domain TOML files (physical, organisms,
artifacts, qualities, dispositions, roles, information, processes) plus a
top-level `ontology.toml` for namespaces and imports. Each file declares
classes (IRI suffix, label, single BFO parent, Aristotelian definition,
annotations) and axioms (SubClassOf all-some restrictions, equivalence axioms
for defined classes).

## Install

```
cargo install --path .
```

Requires Rust 1.85+.

## Usage

```
# Build world-ontology.owl from spec/
ousia-forge build --spec spec/ --out world-ontology.owl

# Validate spec without producing output
ousia-forge check --spec spec/

# Print class/property/axiom statistics for a built ontology
ousia-forge stats --out world-ontology.owl
```

## Acceptance criteria (all green)

- AC1: Output parses as well-formed XML and reloads in horned-owl without error.
- AC2: Declares `owl:imports` for BFO 2020; classes use `https://w3id.org/world-ontology/` namespace.
- AC3: 6 defined classes (SentientBeing, Person, Agent, Organization, JustSociety, UnjustSystem) emitted with OWL equivalence axioms.
- AC4: ≥10 foundational SubClassOf all-some restriction axioms in seed spec.
- AC5: `check` rejects a class with zero or ≥2 asserted parents (non-zero exit, names offending class).
- AC6: Two builds of the same spec produce byte-identical output.
- AC7: `stats` prints named-class, object-property, defined-class, and restriction-axiom counts.
- AC8: `cargo test` covers minimal 3-class round-trip, sentience→dignity chain, and 2-parent rejection.

## License

MIT — Joe Yen
