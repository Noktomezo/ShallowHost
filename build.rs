use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=assets/windows/ShallowHost.ico");
    embed_windows_resources();

    for source in [
        "cpp/src/host.h",
        "cpp/src/host.cpp",
        "cpp/src/host_audio.cpp",
        "cpp/src/host_chain.cpp",
        "cpp/src/host_params.cpp",
        "cpp/src/ffi.cpp",
        "cpp/src/native_host.h",
        "cpp/src/cxx_bridge.h",
        "cpp/src/cxx_bridge.cpp",
        "cpp/CMakeLists.txt",
        "cpp/patches/juce-vst3-waveshell-class-index.patch",
    ] {
        println!("cargo:rerun-if-changed={source}");
    }

    cxx_build::bridge("src/engine/ffi.rs")
        .file("cpp/src/cxx_bridge.cpp")
        .include("cpp/src")
        .std("c++20")
        .compile("shallow-host-cxxbridge");

    let profile = env::var("PROFILE").unwrap_or_else(|_| String::from("debug"));
    let configuration = if profile == "release" {
        "Release"
    } else {
        "Debug"
    };

    run_cmake(["-S", "cpp", "-B", "cpp/cargo-build"], "configure JUCE");
    run_cmake(
        ["--build", "cpp/cargo-build", "--config", configuration],
        "build JUCE",
    );

    let library_dir = PathBuf::from(
        env::var_os("CARGO_MANIFEST_DIR").expect("Cargo always sets CARGO_MANIFEST_DIR"),
    )
    .join("cpp")
    .join("cargo-build")
    .join(configuration);

    println!("cargo:rustc-link-search=native={}", library_dir.display());
    println!("cargo:rustc-link-lib=static=engine");

    if cfg!(target_os = "windows") {
        for library in [
            "ole32", "user32", "gdi32", "winmm", "imm32", "oleaut32", "version", "advapi32",
            "setupapi", "shell32", "dwmapi",
        ] {
            println!("cargo:rustc-link-lib=dylib={library}");
        }
    }
}

#[cfg(windows)]
fn embed_windows_resources() {
    winresource::WindowsResource::new()
        .set_icon("assets/windows/ShallowHost.ico")
        .compile()
        .unwrap_or_else(|error| panic!("failed to embed ShallowHost icon: {error}"));
}

#[cfg(not(windows))]
fn embed_windows_resources() {}

fn run_cmake<const N: usize>(arguments: [&str; N], action: &str) {
    let output = Command::new("cmake")
        .args(arguments)
        .output()
        .unwrap_or_else(|error| panic!("failed to {action}: {error}"));

    if !output.status.success() {
        panic!(
            "failed to {action}\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
