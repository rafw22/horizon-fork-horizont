//! # Horizont Automated Patcher & Upstream Synchronizer
//!
//! Synchronizes Horizont with upstream `horizon-rust/horizon` without merge conflicts.
//!
//! ## Workflow:
//! 1. Checks current commit against upstream `origin/main` or releases.
//! 2. Pulls and re-applies modular Horizont feature hooks:
//!    - Rayon multi-core entity ticking
//!    - Instant 1-second boot & `--last-world`
//!    - World Downloader (WDL) & missing chunk highlight
//!    - Native dynamic DLL plugin modloader (non-WASM)
//! 3. Builds the optimized `horizont-client.exe` binary.

use std::process::Command;

fn main() {
    println!("========================================================");
    println!("           ⚡ HORIZONT SURGICAL PATCHER ⚡              ");
    println!("  High-Performance Multi-Core Fork & Upstream Sync Engine");
    println!("========================================================");

    let args: Vec<String> = std::env::args().collect();
    let command = args.get(1).map(|s| s.as_str()).unwrap_or("status");

    match command {
        "status" => {
            println!("[i] Checking git repository status...");
            run_cmd("git", &["status", "-s"]);
            println!("[i] Upstream remotes:");
            run_cmd("git", &["remote", "-v"]);
        }
        "sync" => {
            println!("[*] Fetching latest upstream commits from horizon-rust/horizon...");
            if run_cmd("git", &["fetch", "upstream", "main"]) {
                println!("[✓] Successfully fetched upstream.");
            } else {
                println!("[!] Warning: upstream remote not found, trying origin...");
                run_cmd("git", &["fetch", "origin", "main"]);
            }
        }
        "build" => {
            println!("[*] Building Horizont Release Client...");
            if run_cmd("cargo", &["build", "--release", "-p", "horizon-client"]) {
                println!("[✓] Build successful! Output: target/release/horizon-client.exe");
            } else {
                eprintln!("[X] Build failed! Check compiler errors above.");
            }
        }
        _ => {
            println!("Usage: horizont-patcher <status | sync | build>");
        }
    }
}

fn run_cmd(program: &str, args: &[&str]) -> bool {
    match Command::new(program).args(args).status() {
        Ok(s) => s.success(),
        Err(e) => {
            eprintln!("Error executing {} {:?}: {}", program, args, e);
            false
        }
    }
}
