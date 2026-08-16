use std::env;
use std::fs;
use std::path::PathBuf;

fn version_numeric() -> String {
    let version = env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "0.0.0".into());
    let mut parts: Vec<&str> = version.split('.').collect();
    parts.resize(4, "0");
    parts.join(", ")
}

fn write_rc(
    path: &PathBuf,
    original_filename: &str,
    file_description: &str,
    product_name: &str,
    icons: &[(&str, &str)],
) -> std::io::Result<()> {
    let version = env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "0.0.0".into());
    let numeric = version_numeric();

    let mut rc = String::new();
    rc.push_str("#pragma code_page(65001)\n");
    rc.push_str("1 VERSIONINFO\n");
    rc.push_str("FILEFLAGS 0x0\n");
    rc.push_str("FILETYPE 0x1\n");
    rc.push_str("FILEOS 0x40004\n");
    rc.push_str("FILEFLAGSMASK 0x3f\n");
    rc.push_str(&format!("FILEVERSION {numeric}\n"));
    rc.push_str("FILESUBTYPE 0x0\n");
    rc.push_str(&format!("PRODUCTVERSION {numeric}\n"));
    rc.push_str("{\n");
    rc.push_str("BLOCK \"StringFileInfo\"\n");
    rc.push_str("{\n");
    rc.push_str("BLOCK \"000004b0\"\n");
    rc.push_str("{\n");
    rc.push_str(&format!("VALUE \"ProductVersion\", \"{version}\"\n"));
    rc.push_str(&format!("VALUE \"ProductName\", \"{product_name}\"\n"));
    rc.push_str("VALUE \"LegalCopyright\", \"dece1ver c 2026\"\n");
    rc.push_str(&format!(
        "VALUE \"OriginalFilename\", \"{original_filename}\"\n"
    ));
    rc.push_str(&format!("VALUE \"FileVersion\", \"{version}\"\n"));
    rc.push_str(&format!(
        "VALUE \"FileDescription\", \"{file_description}\"\n"
    ));
    rc.push_str("}\n");
    rc.push_str("}\n");
    rc.push_str("BLOCK \"VarFileInfo\" {\n");
    rc.push_str("VALUE \"Translation\", 0x0, 0x04b0\n");
    rc.push_str("}\n");
    rc.push_str("}\n");
    for (id, icon) in icons {
        rc.push_str(&format!("{id} ICON \"{icon}\"\n"));
    }
    fs::write(path, rc)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return Ok(());
    }

    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    let manifest_dir = env::var("CARGO_MANIFEST_DIR")?;

    let cncr_rc = out_dir.join("cncr.rc");
    write_rc(
        &cncr_rc,
        "cncr.exe",
        "CNC Remedy",
        "CNC Remedy",
        &[
            ("1", "icons/rename256.ico"),
            ("2", "icons/rename48.ico"),
            ("3", "icons/rename32.ico"),
        ],
    )?;

    let cnctt_rc = out_dir.join("cnctt.rc");
    write_rc(
        &cnctt_rc,
        "cnctt.exe",
        "NC Tool Table",
        "NC Tool Table",
        &[("1", "icons/table.ico")],
    )?;

    let include_dir = manifest_dir;
    embed_resource::compile_for(
        &cncr_rc,
        ["cncr"],
        embed_resource::ParamsIncludeDirs(&[include_dir.as_str()]),
    )
    .manifest_optional()?;
    embed_resource::compile_for(
        &cnctt_rc,
        ["cnctt"],
        embed_resource::ParamsIncludeDirs(&[include_dir.as_str()]),
    )
    .manifest_optional()?;

    Ok(())
}
