mod candidates;
mod output;

use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use candidates::CandidateSource;

use crate::candidates::{gradle, hadoop, jmeter, kotlin, maven, tomcat};

#[tokio::main]
async fn main() -> Result<()> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    let sources: Vec<(&str, Arc<dyn CandidateSource>)> = vec![
        ("Maven", Arc::new(maven::Maven)),
        ("Gradle", Arc::new(gradle::Gradle)),
        ("Kotlin", Arc::new(kotlin::Kotlin)),
        ("Tomcat", Arc::new(tomcat::Tomcat)),
        ("JMeter", Arc::new(jmeter::Jmeter)),
        ("Hadoop", Arc::new(hadoop::Hadoop)),
    ];

    let mut index_entries = Vec::new();

    for (label, source) in &sources {
        println!("[{label}]");
        match source.fetch_versions().await {
            Ok(versions) => {
                let entry = output::write_candidate(&root, source.as_ref(), versions)?;
                index_entries.push((source.dir_name().to_string(), entry));
            }
            Err(e) => {
                eprintln!("  ✗ ERROR: {e:#}");
            }
        }
    }

    println!("\n[index]");
    output::write_index(&root, index_entries)?;

    println!("\nDone.");
    Ok(())
}
