use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=assets/windows/ShallowHost.ico");
    println!("cargo:rerun-if-changed=assets/windows/ShallowHostDev.ico");

    let profile = env::var("PROFILE").unwrap_or_else(|_| String::from("debug"));
    embed_windows_resources(&profile);

    for source in [
        "cpp/src/host.h",
        "cpp/src/host.cpp",
        "cpp/src/host_audio.cpp",
        "cpp/src/host_chain.cpp",
        "cpp/src/host_scan.cpp",
        "cpp/src/host_params.cpp",
        "cpp/src/ffi.cpp",
        "cpp/src/native_host.h",
        "cpp/src/cxx_bridge.h",
        "cpp/src/cxx_bridge.cpp",
        "cpp/CMakeLists.txt",
        "cpp/patches/juce-vst3-waveshell-class-index.patch",
        "cpp/third_party/xaymar_vst2_juce/xaymar_vst2_preinclude.h",
        "cpp/third_party/xaymar_vst2_juce/include/pluginterfaces/vst2.x/aeffect.h",
        "cpp/third_party/xaymar_vst2_juce/include/pluginterfaces/vst2.x/aeffectx.h",
    ] {
        println!("cargo:rerun-if-changed={source}");
    }

    cxx_build::bridge("src/infrastructure/engine/ffi.rs")
        .file("cpp/src/cxx_bridge.cpp")
        .include("cpp/src")
        .std("c++20")
        .compile("shallow-host-cxxbridge");

    // `cxx-build` uses the release MSVC C++ ABI even for a Cargo debug build.
    // Building JUCE with CMake's Debug configuration defines `_DEBUG`, which
    // changes the CRT and STL ABI and makes the two static libraries unlinkable.
    let native_configuration = "Release";

    run_cmake(["-S", "cpp", "-B", "cpp/cargo-build"], "configure JUCE");
    run_cmake(
        [
            "--build",
            "cpp/cargo-build",
            "--config",
            native_configuration,
        ],
        "build JUCE",
    );

    let library_dir = PathBuf::from(
        env::var_os("CARGO_MANIFEST_DIR").expect("Cargo always sets CARGO_MANIFEST_DIR"),
    )
    .join("cpp")
    .join("cargo-build")
    .join(native_configuration);

    println!("cargo:rustc-link-search=native={}", library_dir.display());
    println!("cargo:rustc-link-lib=static=engine");

    if cfg!(target_os = "windows") {
        for library in [
            "ole32", "user32", "gdi32", "winmm", "imm32", "oleaut32", "version", "advapi32",
            "setupapi", "shell32", "dwmapi", "comdlg32",
        ] {
            println!("cargo:rustc-link-lib=dylib={library}");
        }
    }
}

#[cfg(windows)]
fn embed_windows_resources(profile: &str) {
    let (icon, display_name) = if profile == "release" {
        ("assets/windows/ShallowHost.ico", "ShallowHost")
    } else {
        ("assets/windows/ShallowHostDev.ico", "ShallowHost (Dev)")
    };

    let mut resource = winresource::WindowsResource::new();
    resource
        .set_icon(icon)
        .set("ProductName", display_name)
        .set("FileDescription", display_name)
        .set("InternalName", display_name)
        .set("OriginalFilename", "ShallowHost.exe");
    resource
        .compile()
        .unwrap_or_else(|error| panic!("failed to embed ShallowHost icon `{icon}`: {error}"));
}

#[cfg(not(windows))]
fn embed_windows_resources(_profile: &str) {}

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
