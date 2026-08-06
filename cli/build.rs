use std::env;
use std::fs;
use std::io;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    let package_json = manifest_dir.join("../package.json");
    let package = fs::read_to_string(&package_json)?;
    let package: serde_json::Value = serde_json::from_str(&package)?;
    let package_version = package
        .get("version")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| io::Error::other("root package.json must contain a string version"))?;
    let version = env::var("VULCANUM_VERSION").unwrap_or_else(|_| package_version.to_owned());

    println!("cargo:rerun-if-changed={}", package_json.display());
    println!("cargo:rerun-if-env-changed=VULCANUM_VERSION");
    println!("cargo:rustc-env=VULCANUM_VERSION={version}");

    Ok(())
}
