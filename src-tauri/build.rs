use std::path::PathBuf;
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

    // Direct Cargo to search for and link the import library
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let build_output_dir = std::env::current_dir()
        .unwrap()
        .join("cpp")
        .join("build")
        .join(config);

    println!(
        "cargo:rustc-link-search=native={}",
        build_output_dir.display()
    );
    println!("cargo:rustc-link-lib=dylib=shallow_host_cpp");

    // Copy DLL to OUT_DIR and target dir so it is available at runtime/linking
    let dll_name = "shallow_host_cpp.dll";
    let src_dll = build_output_dir.join(dll_name);

    // Copy to OUT_DIR
    if src_dll.exists() {
        let dest_dll_out = out_dir.join(dll_name);
        if let Err(e) = std::fs::copy(&src_dll, &dest_dll_out) {
            println!("cargo:warning=Failed to copy DLL to OUT_DIR: {e}");
        }

        // Copy to target directory (parent of OUT_DIR target/profile/build/shallow-host-xxxx/out)
        if let Some(target_dir) = out_dir.ancestors().nth(3) {
            let dest_dll_target = target_dir.join(dll_name);
            if let Err(e) = std::fs::copy(&src_dll, &dest_dll_target) {
                println!("cargo:warning=Failed to copy DLL to target dir: {e}. If the app is running, close it first.");
            }

            // Also copy to target/profile/deps
            let dest_dll_deps = target_dir.join("deps").join(dll_name);
            if let Err(e) = std::fs::copy(&src_dll, &dest_dll_deps) {
                println!("cargo:warning=Failed to copy DLL to deps: {e}");
            }
        }
    }

    tauri_build::build();
}
