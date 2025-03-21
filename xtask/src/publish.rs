use crate::{project_root, read_cargo_toml};
use anyhow::Result;
use core::time;
use std::{path::Path, thread::sleep};
use xshell::{cmd, Shell};

// Add more packages as needed, in the right order.
// A package should only be published after all it's dependencies have been published
const PUBLISH_PACKAGES: &[&str] = &[
    "opentelemetry",
    "opentelemetry-http",
    "opentelemetry-semantic-conventions",
    "opentelemetry-jaeger-propagator",
    "opentelemetry-sdk",
    "opentelemetry-proto",
    "opentelemetry-otlp",
    "opentelemetry-stdout",
    "opentelemetry-zipkin",
    "opentelemetry-prometheus",
    "opentelemetry-appender-log",
    "opentelemetry-appender-tracing",
];

pub fn publish(sh: &Shell) -> Result<()> {
    let root = project_root(sh)?;
    // Is a try run unless forced.
    let dry_run = std::env::args()
        .filter(|e| e.eq("--force"))
        .collect::<Vec<String>>()
        .is_empty();

    for package in PUBLISH_PACKAGES.to_owned().iter() {
        // Check for the Dihectory
        let path = Path::new(&root).join(package);

        if !path.is_dir() {
            eprintln!("Skipping: {package} is not a valid package directory.");
            continue;
        }
        // Set the current directory to the project root.
        sh.change_dir(&path);

        println!("==================================================");
        println!("Processing package: {package}");
        let cargo = match read_cargo_toml(path.join("Cargo.toml")) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("{e}");
                continue;
            }
        };

        let name = cargo.package.name.as_str();
        let version = cargo.package.version.as_str();
        if name.is_empty() || version.is_empty() {
            eprintln!("Empty Name or Version");
            continue;
        }
        let tag = format!("{name}-{version}");
        let tag_message = format!("{name} {version} release.");
        let tag_command = format!("git tag -a \"{tag}\" -m \"{tag_message}\"");
        let push_tag_command = format!("git push origin \"{tag}\"");

        if dry_run {
            let _ = cmd!(sh, "cargo publish --dry-run").run();
            println!("DRY RUN: {tag_command}");
            println!("DRY RUN: {push_tag_command}");
        } else {
            let mut pub_cmd = cmd!(sh, "cargo publish");
            pub_cmd.set_ignore_stdout(false);
            let _ = pub_cmd.run();
            let _ = cmd!(sh, "{tag_command}").run();
            let _ = cmd!(sh, "{push_tag_command}").run();
        }
        println!("Published {name} {version}");

        println!("Sleeping for 15 seconds before next package...");
        // Sleep for 15 seconds to allow crates.io to index the previous package
        sleep(time::Duration::from_secs(15));
    }

    Ok(())
}
