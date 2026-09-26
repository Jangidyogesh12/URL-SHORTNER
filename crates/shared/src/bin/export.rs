fn main() -> Result<(), ts_rs::ExportError> {
    shared::export_all()?;
    println!("TypeScript bindings exported to packages/shared-types/src/generated/");
    Ok(())
}
