use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let task = args.first().map(|s| s.as_str()).unwrap_or("build");

    match task {
        "build" => build_release(),
        other => {
            eprintln!("Unknown task: '{other}'. Available tasks: build");
            std::process::exit(1);
        }
    }
}

fn build_release() {
    println!("=== Building Release Binary ===");
    let status = Command::new("cargo")
        .args([
            "build",
            "--release",
            "--package",
            "shallow-host-gpui",
            "--bin",
            "ShallowHost",
        ])
        .status()
        .expect("Failed to execute cargo build");

    if !status.success() {
        eprintln!(
            "Error: Release build failed with status {:?}",
            status.code()
        );
        std::process::exit(1);
    }

    let release_dir = PathBuf::from("target/release");
    let exe_path = if cfg!(target_os = "windows") {
        release_dir.join("ShallowHost.exe")
    } else {
        release_dir.join("ShallowHost")
    };

    println!("=== Compressing binary with UPX (--best --lzma) ===");
    let upx_status = Command::new("upx")
        .args(["--best", "--lzma"])
        .arg(&exe_path)
        .status();

    match upx_status {
        Ok(s) if s.success() => {
            println!(
                "Successfully built and compressed {} with UPX!",
                exe_path.display()
            );
        }
        Ok(s) => {
            println!(
                "Note: UPX compression returned exit code {:?}, release binary is uncompressed.",
                s.code()
            );
        }
        Err(_) => {
            println!("Note: UPX not found or failed, release binary ready at target/release!");
        }
    }
}
