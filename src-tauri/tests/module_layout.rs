use std::{collections::BTreeSet, path::Path};

#[test]
fn native_source_root_contains_only_entrypoints_and_domain_directories() {
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let entries = std::fs::read_dir(&source_root)
        .expect("read native source root")
        .map(|entry| {
            entry
                .expect("read native source entry")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .collect::<BTreeSet<_>>();

    let expected = BTreeSet::from([
        "agents".to_owned(),
        "app".to_owned(),
        "commands".to_owned(),
        "extensions".to_owned(),
        "identity".to_owned(),
        "lib.rs".to_owned(),
        "main.rs".to_owned(),
        "preferences".to_owned(),
        "relay".to_owned(),
    ]);

    assert_eq!(entries, expected);
}
