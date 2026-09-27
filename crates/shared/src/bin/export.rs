use std::{fs, path::PathBuf};

const LEGACY_BINDINGS: [&str; 3] = [
    "HealthResponse.ts",
    "HelloResponse.ts",
    "User.ts",
];

fn main() -> Result<(), ts_rs::ExportError> {
    let generated_dir = PathBuf::from(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../packages/shared-types/src/generated"
    ));

    for binding in LEGACY_BINDINGS {
        let _ = fs::remove_file(generated_dir.join(binding));
    }

    shared::export_all()?;
    println!("TypeScript bindings exported to packages/shared-types/src/generated/");
    Ok(())
}
