use std::path::{Path, PathBuf};

const WRITE: &str = "OGURPCHIK_WRITE_BINDINGS";

fn squeezed(text: &str) -> String {
    text.split_whitespace().collect()
}

fn generate(filter: &Path, output: &Path) -> String {
    let _ = std::fs::remove_file(output);
    windows_bindgen::builder()
        .input_default()
        .flat()
        .filter_file(filter)
        .output(output)
        .write();
    std::fs::read_to_string(output).unwrap_or_default()
}

#[test]
fn the_bindings_are_what_their_filter_writes() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let (filter, held) = (root.join("src/bindings.txt"), root.join("src/bindings.rs"));
    let fresh: PathBuf = std::env::temp_dir().join(format!("ogurpchik-bindings-{}.rs", std::process::id()));
    let written = generate(&filter, &fresh);
    let _ = std::fs::remove_file(&fresh);
    assert!(!written.is_empty(), "windows-bindgen wrote nothing from {}", filter.display());

    if std::env::var_os(WRITE).is_some() {
        std::fs::write(&held, &written).expect("write the bindings");
        return;
    }
    let held_text = std::fs::read_to_string(&held).unwrap_or_default();
    assert!(
        squeezed(&held_text) == squeezed(&written),
        "{} is not what {} writes: run `{WRITE}=1 cargo test --test bindings`",
        held.display(),
        filter.display()
    );
}
