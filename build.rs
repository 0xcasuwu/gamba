use anyhow::Result;
use flate2::write::GzEncoder;
use flate2::Compression;
use hex;
use std::env;
use std::fs;
use std::io::prelude::*;
use std::path::Path;
use std::process::{Command, Stdio};

fn compress(binary: Vec<u8>) -> Result<Vec<u8>> {
    let mut writer = GzEncoder::new(Vec::<u8>::with_capacity(binary.len()), Compression::best());
    writer.write_all(&binary)?;
    Ok(writer.finish()?)
}

fn build_alkane(wasm_str: &str, features: Vec<&'static str>) -> Result<()> {
    if features.len() != 0 {
        let _ = Command::new("cargo")
            .env("CARGO_TARGET_DIR", wasm_str)
            .arg("build")
            .arg("--target")
            .arg("wasm32-unknown-unknown")
            .arg("--release")
            .arg("--features")
            .arg(features.join(","))
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()?
            .wait()?;
        Ok(())
    } else {
        Command::new("cargo")
            .env("CARGO_TARGET_DIR", wasm_str)
            .arg("build")
            .arg("--target")
            .arg("wasm32-unknown-unknown")
            .arg("--release")
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()?
            .wait()?;
        Ok(())
    }
}

fn main() {
    let env_var = env::var_os("OUT_DIR").unwrap();
    let base_dir = Path::new(&env_var)
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let out_dir = base_dir.join("release");
    let wasm_dir = base_dir.parent().unwrap().join("alkanes");
    fs::create_dir_all(&wasm_dir).unwrap();
    let wasm_str = wasm_dir.to_str().unwrap();
    let write_dir = Path::new(&out_dir)
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("src");

    fs::create_dir_all(&write_dir.join("precompiled")).unwrap();
    fs::create_dir_all(&write_dir.join("tests").join("std")).unwrap();
    
    // Use CARGO_MANIFEST_DIR for reliable path resolution
    let crates_dir = if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        Path::new(&manifest_dir).join("alkanes")
    } else {
        out_dir
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("alkanes")
    };
    
    eprintln!("Looking for alkanes directory at: {:?}", crates_dir);
    
    // Check if directory exists
    if !crates_dir.exists() {
        eprintln!("Error: alkanes directory not found at {:?}", crates_dir);
        
        // Create empty mod.rs for precompiled directory  
        fs::write(
            &write_dir.join("precompiled").join("mod.rs"),
            "// Auto-generated build modules\n// No alkanes directory found\n"
        ).unwrap();
        return;
    }
    
    std::env::set_current_dir(&crates_dir).unwrap();
    let mods = fs::read_dir(&crates_dir)
        .unwrap()
        .filter_map(|v| {
            let entry = v.ok()?;
            let name = entry.file_name().into_string().ok()?;
            
            // Skip if it's a file
            if !entry.path().is_dir() {
                return None;
            }
            
            // Skip non-alkane directories (build artifacts, etc.)
            if name.starts_with(".") || name == "target" || name == "wasm32-unknown-unknown" || name == "release" || name == "gamba" {
                return None;
            }
            
            let cargo_toml_path = entry.path().join("Cargo.toml");
            if !cargo_toml_path.exists() {
                eprintln!("Skipping {} - no Cargo.toml found", name);
                return None;
            }
            
            Some(name)
        })
        .collect::<Vec<String>>();
        
    eprintln!("Found alkane projects: {:?}", mods);
    
    let built_modules = mods.into_iter()
        .filter_map(|v| -> Option<String> {
            let result = (|| -> Result<String> {
                eprintln!("Building alkane: {}", v);
                let alkane_dir = crates_dir.clone().join(v.clone());
                std::env::set_current_dir(&alkane_dir)?;
                
                // Check if this directory has a valid Cargo.toml with lib target
                let cargo_toml_content = fs::read_to_string("Cargo.toml").unwrap_or_default();
                if !cargo_toml_content.contains("[lib]") && !cargo_toml_content.contains("crate-type") {
                    eprintln!("Skipping {} - not a library crate", v);
                    return Err(anyhow::anyhow!("Not a library crate"));
                }
                
                // Build the alkane to WASM
                build_alkane(wasm_str, vec![])?;
                std::env::set_current_dir(&crates_dir)?;
                
                let subbed = v.clone().replace("-", "_");
                let wasm_path = Path::new(&wasm_str)
                    .join("wasm32-unknown-unknown")
                    .join("release")
                    .join(subbed.clone() + ".wasm");
                    
                if !wasm_path.exists() {
                    eprintln!("Error: WASM file not found at {:?}", wasm_path);
                    eprintln!("Available files in release dir:");
                    let release_dir = Path::new(&wasm_str).join("wasm32-unknown-unknown").join("release");
                    if let Ok(entries) = fs::read_dir(&release_dir) {
                        for entry in entries {
                            if let Ok(entry) = entry {
                                eprintln!("  {:?}", entry.file_name());
                            }
                        }
                    }
                    return Err(anyhow::anyhow!("WASM file not found"));
                }
                
                let f: Vec<u8> = fs::read(&wasm_path)?;
                let compressed: Vec<u8> = compress(f.clone())?;
                fs::write(&Path::new(&wasm_str).join("wasm32-unknown-unknown").join("release").join(subbed.clone() + ".wasm.gz"), &compressed)?;
                let data: String = hex::encode(&f);
                
                let build_file_path = write_dir.join("tests").join("std").join(subbed.clone() + "_build.rs");
                fs::write(
                    &build_file_path,
                    String::from("use hex_lit::hex;\n#[allow(long_running_const_eval)]\npub fn get_bytes() -> Vec<u8> { (&hex!(\"")
                        + data.as_str()
                        + "\")).to_vec() }",
                )?;
                
                eprintln!("Generated build file: {:?}", build_file_path);
                Ok(subbed)
            })();
            
            match result {
                Ok(subbed) => {
                    eprintln!("Successfully built alkane: {} -> {}", v, subbed);
                    Some(subbed)
                },
                Err(e) => {
                    eprintln!("Failed to build {}: {}", v, e);
                    None
                }
            }
        })
        .collect::<Vec<String>>();
        
    eprintln!("Built modules: {:?}", built_modules);
    
    let mod_rs_path = write_dir.join("tests").join("std").join("mod.rs");
    let mod_content = if built_modules.is_empty() {
        "// Auto-generated build modules\n// No alkanes successfully compiled\n".to_string()
    } else {
        built_modules.into_iter()
            .fold(String::from("// Auto-generated build modules\n"), |r, v| {
                r + "pub mod " + v.as_str() + "_build;\n"
            })
    };
    
    fs::write(&mod_rs_path, mod_content).unwrap();
    eprintln!("Generated mod.rs at: {:?}", mod_rs_path);
}
