use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs;
use std::path::PathBuf;

use futures::stream::FuturesUnordered;
use futures::stream::StreamExt;
use walkdir::WalkDir;

/// Download the latest protobuf schemas from the Google APIs GitHub repository.
///
/// This test is ignored by default, but can be run with `cargo test sync_schemas -- --ignored`.
#[tokio::test]
#[ignore]
async fn sync_schemas() {
    let client = reqwest::Client::new();
    let cache = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("proto/google");
    let schemas = PREREQUISITE_SCHEMAS
        .iter()
        .chain(GENERATE_FROM_SCHEMAS.iter());

    let mut futures = FuturesUnordered::new();
    for path in schemas.copied() {
        let filename = cache.join(path);
        let client = client.clone();
        futures.push(async move {
            let url = format!("{}/{}", BASE_URI, path);
            let rsp = client.get(url).send().await.unwrap();
            let body = rsp.text().await.unwrap();
            fs::create_dir_all(filename.parent().unwrap()).unwrap();
            fs::write(filename, body).unwrap();
        });
    }

    while futures.next().await.is_some() {}
}

/// Use the protobuf schemas downloaded by the `sync_schemas` test to generate code.
///
/// This test will fail if the code currently in the repository is different from the
/// newly generated code, and will update it in place in that case.
#[test]
fn generated_code_is_fresh() {
    // Generate code into a temporary directory.

    let schemas = GENERATE_FROM_SCHEMAS
        .iter()
        .map(|s| format!("google/{}", s))
        .collect::<Vec<_>>();

    let tmp_dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(&tmp_dir).unwrap();
    tonic_build::configure()
        .build_client(true)
        .build_server(false)
        .format(which::which("rustfmt").is_ok())
        .out_dir(&tmp_dir)
        .compile(&schemas, &["proto"])
        .unwrap();

    // Next, wrangle the generated file names into a directory hierarchy.

    let (mut modules, mut renames) = (Vec::new(), Vec::new());
    for entry in fs::read_dir(&tmp_dir).unwrap() {
        let path = entry.unwrap().path();
        let file_name_str = path.file_name().and_then(|s| s.to_str()).unwrap();
        let (base, _) = file_name_str
            .strip_prefix("google.")
            .unwrap()
            .rsplit_once('.')
            .unwrap();

        let new = match base.rsplit_once('.') {
            Some((dir, fname)) => {
                let mut module = dir.split('.').map(|s| s.to_owned()).collect::<Vec<_>>();
                module.push(fname.to_owned());
                modules.push(module);
                tmp_dir
                    .path()
                    .join(dir.replace('.', "/").replace("r#", ""))
                    .join(format!("{}.rs", fname.replace("r#", "")))
            }
            None => {
                let new = tmp_dir
                    .path()
                    .join(format!("{}.rs", base.replace("r#", "")));
                modules.push(vec![base.to_owned()]);
                new
            }
        };

        renames.push((path, new));
    }

    // Rename the files into place after iterating over the old version.

    for (old, new) in renames {
        fs::create_dir_all(new.parent().unwrap()).unwrap();
        fs::rename(old, new).unwrap();
    }

    // Build the module root and write it to `mod.rs`.

    modules.sort_unstable();
    let mut previous: &[String] = &[];
    let (mut root, mut level) = (String::new(), 0);
    for module in &modules {
        // Find out how many modules to close and what modules to open.

        let parent = &module[..module.len() - 1];
        let (mut close, mut open) = (0, vec![]);
        let components = Ord::max(previous.len(), parent.len());
        for i in 0..components {
            let (prev, cur) = (previous.get(i), parent.get(i));
            if prev == cur && close == 0 && open.is_empty() {
                continue;
            }

            match (prev, cur) {
                (Some(_), Some(new)) => {
                    close += 1;
                    open.push(new);
                }
                (Some(_), None) => close += 1,
                (None, Some(new)) => open.push(new),
                (None, None) => unreachable!(),
            }
        }

        // Close modules.

        let closed = close > 0;
        while close > 0 {
            for _ in 0..((level - 1) * 4) {
                root.push(' ');
            }
            root.push_str("}\n");
            close -= 1;
            level -= 1;
        }

        if closed {
            root.push('\n');
        }

        // Open modules.

        let mut opened = false;
        for component in &open {
            if !opened && !closed {
                root.push('\n');
                opened = true;
            }

            for _ in 0..(level * 4) {
                root.push(' ');
            }

            root.push_str("pub mod ");
            root.push_str(component);
            root.push_str(" {\n");
            level += 1;
        }

        // Write a module declaration for this actual module.

        for _ in 0..(level * 4) {
            root.push(' ');
        }
        root.push_str("pub mod ");
        root.push_str(module.last().unwrap());
        root.push_str(";\n");
        previous = parent;
    }

    while level > 0 {
        level -= 1;
        for _ in 0..(level * 4) {
            root.push(' ');
        }
        root.push_str("}\n");
    }

    fs::write(tmp_dir.path().join("mod.rs"), root).unwrap();

    // Move on to actually comparing the old and new versions.

    let versions = [SOURCE_DIR, tmp_dir.path().to_str().unwrap()]
        .iter()
        .map(|path| {
            let mut files = HashMap::new();
            for entry in WalkDir::new(path) {
                let entry = match entry {
                    Ok(e) => e,
                    Err(_) => continue,
                };

                let is_file = entry.file_type().is_file();
                let rs = entry.path().extension() == Some(OsStr::new("rs"));
                if !is_file || !rs {
                    continue;
                }

                let file = entry.path();
                let name = file.strip_prefix(path).unwrap();
                files.insert(name.to_owned(), fs::read_to_string(file).unwrap());
            }

            files
        })
        .collect::<Vec<_>>();

    // Compare the old version and new version and fail the test if they're different.

    let mut keys = versions[0].keys().collect::<Vec<_>>();
    keys.extend(versions[1].keys());
    keys.sort_unstable();
    keys.dedup();

    if versions[0] != versions[1] {
        let _ = fs::remove_dir_all(SOURCE_DIR);
        fs::rename(tmp_dir, SOURCE_DIR).unwrap();
        panic!("generated code in the repository is outdated, updating...");
    }
}

/// Schema files used as input for the generated code.
const GENERATE_FROM_SCHEMAS: &[&str] = &[
    "devtools/cloudtrace/v2/tracing.proto",
    "devtools/cloudtrace/v2/trace.proto",
    "logging/type/http_request.proto",
    "logging/v2/log_entry.proto",
    "logging/v2/logging.proto",
    "rpc/status.proto",
];

/// Schema files that are dependencies of the `GENERATED_SCHEMAS`.
const PREREQUISITE_SCHEMAS: &[&str] = &[
    "api/annotations.proto",
    "api/resource.proto",
    "api/monitored_resource.proto",
    "api/field_behavior.proto",
    "api/http.proto",
    "api/client.proto",
    "logging/type/log_severity.proto",
    "api/label.proto",
    "api/launch_stage.proto",
    "logging/v2/logging_config.proto",
];

const BASE_URI: &str = "https://raw.githubusercontent.com/googleapis/googleapis/master/google";
const SOURCE_DIR: &str = "src/proto";
