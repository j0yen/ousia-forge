# ousia-forge

Compiles a directory of human-auditable TOML files into an OWL 2 DL world ontology (`world-ontology.owl`, RDF/XML, importing BFO 2020).

## Why it exists

An ontology you can reason over is OWL — verbose, machine-shaped RDF/XML that no one wants to hand-edit or review in a pull request. An ontology a person can read and argue with is prose, which a reasoner cannot touch. The two forms pull in opposite directions, and most ontology work pays the cost of editing the machine form by hand.

`ousia-forge` lets you write the readable form and get the machine form for free. The spec is TOML: each class is a few lines naming its IRI suffix, its label, its single BFO parent, an Aristotelian definition, and any annotations; axioms are declared as all-some restrictions or equivalence axioms. `ousia-forge` checks that spec and emits the OWL. It is the first step of the ousia toolchain — the reasoner, the SPARQL layer, and the action gate all operate on the `.owl` artifact this produces, so nothing downstream runs until `ousia-forge` has built it.

## The spec

A spec is a directory. `ontology.toml` carries the ontology IRI, the BFO import, and the custom annotation properties (`aristotelianDefinition`, `philosophicalGrounding`, `aiGuidance`, `moralSignificance`, …). Alongside it sit the domain files — `organisms`, `qualities`, `dispositions`, `roles`, `information`, `processes` — and `defined_classes.toml` for classes given by OWL equivalence axioms rather than a bare parent. The `spec/` directory in this repo is a working seed you can build immediately.

Every class declares exactly one asserted BFO parent. Single inheritance is enforced, not encouraged: a class with zero parents or two-or-more parents is a build error that names the offender.

## Install

```sh
cargo install --path .
```

Requires Rust 1.85+.

## Usage

```sh
# Build world-ontology.owl from the spec directory
ousia-forge build --spec spec/ --out world-ontology.owl

# Validate the spec without emitting anything
ousia-forge check --spec spec/

# Count classes, properties, and axioms in a built ontology
ousia-forge stats --out world-ontology.owl
```

`build` runs the check first, so a spec that fails validation never produces an `.owl`. `check` exits non-zero and names the offending class. Two builds of the same spec produce byte-identical output, which makes the artifact diffable in version control.

## What the seed spec contains

The seed declares six defined classes — `SentientBeing`, `Person`, `Agent`, `Organization`, `JustSociety`, `UnjustSystem` — each emitted with an OWL equivalence axiom, on top of the named BFO-parented classes and a set of foundational SubClassOf all-some restriction axioms. The output parses as well-formed RDF/XML, reloads in `horned-owl`, imports BFO 2020, and places its classes under the `https://w3id.org/world-ontology/` namespace.

## Where it fits

`ousia-forge` is the builder of the **ousia** project — a suite of Rust tools operationalizing a BFO-grounded, OWL 2 DL world ontology. It produces the artifact the rest of the chain consumes: `ousia-reason` (OWL 2 DL entailment), `ousia-sparql` (queries), `ousia-guard` (action gating), and `ousia-mcp` (the MCP surface). The companion [`ousia-atscale`](https://github.com/j0yen/ousia-atscale) grounds AtScale semantic models against the same BFO categories.

## Tests

```sh
cargo test
```

The suite covers a minimal three-class round-trip, the sentience-to-dignity inference chain, and the two-parent rejection. The acceptance criteria the build is written against are listed in `spec/`.

## License

MIT — Joe Yen.
