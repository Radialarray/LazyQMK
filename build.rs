//! Build script for LazyQMK.
//!
//! This script ensures the web frontend is built before compiling the Rust binary.
//! The web frontend build output is embedded into the binary using rust-embed.

use std::env;
use std::path::Path;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=web/src");
    println!("cargo:rerun-if-changed=web/package.json");
    println!("cargo:rerun-if-changed=web/package-lock.json");
    println!("cargo:rerun-if-changed=web/svelte.config.js");
    println!("cargo:rerun-if-changed=web/vite.config.ts");
    println!("cargo:rerun-if-changed=web/tsconfig.json");

    // Only build web frontend if the web feature is enabled
    if env::var("CARGO_FEATURE_WEB").is_ok() {
        build_web_frontend();
    }
}

fn build_web_frontend() {
    let web_dir = Path::new("web");

    // Check if web directory exists
    if !web_dir.exists() {
        eprintln!("Warning: web directory not found, skipping frontend build");
        return;
    }

    // Check if node_modules exists, if not run npm install
    let node_modules = web_dir.join("node_modules");
    if !node_modules.exists() {
        println!("cargo:warning=Installing npm dependencies...");
        let status = Command::new("npm")
            .args(["install", "--prefer-offline"])
            .current_dir(web_dir)
            .status()
            .expect("Failed to run npm install - ensure Node.js and npm are installed");

        assert!(status.success(), "npm install failed");
    }

    // Always build the web frontend to ensure latest changes are embedded
    println!("cargo:warning=Building web frontend...");
    let status = Command::new("npm")
        .args(["run", "build"])
        .current_dir(web_dir)
        .status()
        .expect("Failed to run npm build - ensure Node.js and npm are installed");

    assert!(status.success(), "Web frontend build failed");

    // Verify build output exists
    let build_dir = web_dir.join("build");
    assert!(
        build_dir.exists(),
        "Web frontend build directory not found after build"
    );

    println!("cargo:warning=Web frontend build complete");
}
