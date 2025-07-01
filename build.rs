use anyhow::Result;
use flate2::write::GzEncoder;
use flate2::Compression;
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
            .arg("--release")
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()?
            .wait()?;
        Ok(())
    }
}

fn main() {
    if std::env::var("BUILD_IN_PROGRESS").is_ok() {
        println!("Build script already running, skipping to prevent recursion");
        return;
    }
    std::env::set_var("BUILD_IN_PROGRESS", "1");
    
    // Set up secp256k1 to use system library
    #[cfg(target_os = "macos")]
    {
        // Force use of system secp256k1 library
        std::env::set_var("SECP256K1_SYS_USE_PKG_CONFIG", "1");
        println!("cargo:rustc-env=SECP256K1_SYS_USE_PKG_CONFIG=1");
        
        // Set PKG_CONFIG_PATH for Homebrew
        if std::path::Path::new("/usr/local/lib/pkgconfig").exists() {
            let pkg_config_path = std::env::var("PKG_CONFIG_PATH").unwrap_or_default();
            let new_path = if pkg_config_path.is_empty() {
                "/usr/local/lib/pkgconfig".to_string()
            } else {
                format!("/usr/local/lib/pkgconfig:{}", pkg_config_path)
            };
            std::env::set_var("PKG_CONFIG_PATH", &new_path);
            println!("cargo:rustc-env=PKG_CONFIG_PATH={}", new_path);
        }
        
        // Set up LLVM toolchain for macOS
        if std::env::var("AR").is_err() {
            if std::path::Path::new("/usr/local/opt/llvm/bin/llvm-ar").exists() {
                std::env::set_var("AR", "/usr/local/opt/llvm/bin/llvm-ar");
                println!("cargo:rustc-env=AR=/usr/local/opt/llvm/bin/llvm-ar");
            } else if std::path::Path::new("/opt/homebrew/opt/llvm/bin/llvm-ar").exists() {
                std::env::set_var("AR", "/opt/homebrew/opt/llvm/bin/llvm-ar");
                println!("cargo:rustc-env=AR=/opt/homebrew/opt/llvm/bin/llvm-ar");
            }
        }
        
        if std::env::var("CC").is_err() {
            if std::path::Path::new("/usr/local/opt/llvm/bin/clang").exists() {
                std::env::set_var("CC", "/usr/local/opt/llvm/bin/clang");
                println!("cargo:rustc-env=CC=/usr/local/opt/llvm/bin/clang");
            } else if std::path::Path::new("/opt/homebrew/opt/llvm/bin/clang").exists() {
                std::env::set_var("CC", "/opt/homebrew/opt/llvm/bin/clang");
                println!("cargo:rustc-env=CC=/opt/homebrew/opt/llvm/bin/clang");
            }
        }
    }
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
    let crates_dir = out_dir
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    std::env::set_current_dir(&crates_dir).unwrap();

    build_alkane(wasm_str, vec![]).unwrap();
    let mod_name = "alkane_pandas_ap69".to_owned();
    let f: Vec<u8> = fs::read(
        &Path::new(&wasm_str)
            .join("wasm32-unknown-unknown")
            .join("release")
            .join(mod_name.clone() + ".wasm"),
    )
    .unwrap();
    let compressed: Vec<u8> = compress(f.clone()).unwrap();
    fs::write(
        &Path::new(&wasm_str)
            .join("wasm32-unknown-unknown")
            .join("release")
            .join(mod_name.clone() + ".wasm.gz"),
        &compressed,
    )
    .unwrap();
}