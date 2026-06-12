mod candidates;
mod output;

use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Result, bail};
use candidates::CandidateSource;

use crate::candidates::{gradle, hadoop, jmeter, kotlin, maven, tomcat};

fn sources() -> Vec<(&'static str, Arc<dyn CandidateSource>)> {
    let mut sources: Vec<(&'static str, Arc<dyn CandidateSource>)> = vec![
        ("Gradle", Arc::new(gradle::Gradle)),
        ("Hadoop", Arc::new(hadoop::Hadoop)),
        ("JMeter", Arc::new(jmeter::Jmeter)),
        ("Kotlin", Arc::new(kotlin::Kotlin)),
        ("Maven", Arc::new(maven::Maven)),
        ("Tomcat", Arc::new(tomcat::Tomcat)),
    ];
    sources.sort_by_key(|(_, source)| source.dir_name());
    sources
}

#[tokio::main]
async fn main() -> Result<()> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    let mut index_entries = Vec::new();
    let mut candidate_changed = false;

    for (label, source) in sources() {
        println!("[{label}]");
        let previous = output::read_candidate_versions(&root, source.dir_name())?;
        let versions = match source.fetch_versions(&previous).await {
            Ok(versions) => versions,
            Err(error) if !previous.is_empty() => {
                eprintln!("  ! using existing data: {error:#}");
                previous
            }
            Err(error) => return Err(error),
        };
        if versions.len() < source.minimum_versions() {
            bail!(
                "{} source returned {} versions, expected at least {}",
                source.display(),
                versions.len(),
                source.minimum_versions()
            );
        }

        let result = output::write_candidate(&root, source.as_ref(), versions)?;
        candidate_changed |= result.changed;
        index_entries.push((source.dir_name().to_string(), result.entry));
    }

    println!("\n[index]");
    output::write_index(&root, index_entries, candidate_changed)?;

    println!("\nDone.");
    Ok(())
}
