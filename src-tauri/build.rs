use std::process::Command;

fn main() {
    let profile = std::env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());
    let config = if profile == "release" {
        "Release"
    } else {
        "Debug"
    };

    println!("cargo:rerun-if-changed=cpp/src/host.h");
    println!("cargo:rerun-if-changed=cpp/src/host.cpp");
    println!("cargo:rerun-if-changed=cpp/src/host_audio.cpp");
    println!("cargo:rerun-if-changed=cpp/src/host_chain.cpp");
    println!("cargo:rerun-if-changed=cpp/src/host_params.cpp");
    println!("cargo:rerun-if-changed=cpp/src/ffi.cpp");
    println!("cargo:rerun-if-changed=cpp/CMakeLists.txt");

    // Configure CMake
    let configure_output = Command::new("cmake")
        .args(["-S", "cpp", "-B", "cpp/build"])
        .output()
        .expect("Failed to run cmake configure");

    if !configure_output.status.success() {
        println!("cargo:warning=CMake configure failed!");
        println!(
            "cargo:warning=stdout: {}",
            String::from_utf8_lossy(&configure_output.stdout)
        );
        println!(
            "cargo:warning=stderr: {}",
            String::from_utf8_lossy(&configure_output.stderr)
        );
        panic!("CMake configure failed");
    }

    // Build CMake project
    let build_output = Command::new("cmake")
        .args(["--build", "cpp/build", "--config", config])
        .output()
        .expect("Failed to run cmake build");

    if !build_output.status.success() {
        println!("cargo:warning=CMake build failed!");
        println!(
            "cargo:warning=stdout: {}",
            String::from_utf8_lossy(&build_output.stdout)
        );
        println!(
            "cargo:warning=stderr: {}",
            String::from_utf8_lossy(&build_output.stderr)
        );
        panic!("CMake build failed");
    }

    // Direct Cargo to search for and link the static library
    let build_output_dir = std::env::current_dir()
        .unwrap()
        .join("cpp")
        .join("build")
        .join(config);

    println!(
        "cargo:rustc-link-search=native={}",
        build_output_dir.display()
    );
    println!("cargo:rustc-link-lib=static=engine");

    // Remove obsolete engine.dll in src-tauri root if present
    let root_dll = std::env::current_dir().unwrap().join("engine.dll");
    if root_dll.exists() {
        let _ = std::fs::remove_file(root_dll);
    }

    tauri_build::build();
}
