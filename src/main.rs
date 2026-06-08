use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod error;
mod model;
mod parser;
mod builder;
mod checker;
mod stats;

use error::ForgeError;

#[derive(Parser)]
#[command(name = "ousia-forge", about = "Build the World Ontology from a declarative TOML spec")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Build OWL 2 DL RDF/XML from a spec directory
    Build {
        #[arg(long, value_name = "DIR")]
        spec: PathBuf,
        #[arg(long, value_name = "FILE")]
        out: PathBuf,
    },
    /// Validate the spec directory without emitting output
    Check {
        #[arg(long, value_name = "DIR")]
        spec: PathBuf,
    },
    /// Print class/property/axiom counts for a built ontology
    Stats {
        #[arg(long, value_name = "FILE")]
        out: PathBuf,
    },
}

fn main() {
    // Reset SIGPIPE so pipes to head/jq don't panic
    #[cfg(unix)]
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_DFL);
    }

    let cli = Cli::parse();
    let result = run(cli);
    if let Err(e) = result {
        eprintln!("error: {}", e);
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> Result<(), ForgeError> {
    match cli.command {
        Commands::Build { spec, out } => {
            let ontology_spec = parser::load_spec(&spec)?;
            checker::check_spec(&ontology_spec)?;
            builder::build_ontology(&ontology_spec, &out)?;
            eprintln!("built: {}", out.display());
            Ok(())
        }
        Commands::Check { spec } => {
            let ontology_spec = parser::load_spec(&spec)?;
            checker::check_spec(&ontology_spec)?;
            eprintln!("check: ok");
            Ok(())
        }
        Commands::Stats { out } => {
            stats::print_stats(&out)?;
            Ok(())
        }
    }
}
