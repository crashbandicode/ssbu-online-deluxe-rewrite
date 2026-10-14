use std::{
    env, fs,
    path::PathBuf,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

/// Pinned imgui-smash static archive (see docs/IMGUI_DEPENDENCY.md). The
/// preparation script verifies this digest when it places the file; build.rs
/// only re-checks the sidecar and size so no hashing code is needed here.
const IMGUI_SMASH_SHA256: &str = "8a06be16dbfd50d10bac81ee5bdd2ffa9cd3472773151cc45f1c2101541f8f7c";
const IMGUI_SMASH_SIZE: u64 = 13_851_556;
const IMGUI_SMASH_DIR: &str = "target/native/imgui";
const IMGUI_SMASH_LIB: &str = "libimgui_smash.a";

fn main() {
    println!("cargo:rerun-if-changed=src");
    println!("cargo:rerun-if-changed=Cargo.toml");
    println!("cargo:rerun-if-changed=Cargo.lock");
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/index");
    println!("cargo:rerun-if-changed=.git/refs/heads/main");
    println!("cargo:rerun-if-env-changed=SOURCE_DATE_EPOCH");
    let git = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()
        .filter(|v| v.status.success())
        .map(|v| String::from_utf8_lossy(&v.stdout).trim().to_owned())
        .unwrap_or_else(|| "uncommitted".to_owned());
    let dirty = Command::new("git")
        .args(["status", "--porcelain"])
        .output()
        .map(|v| !v.stdout.is_empty())
        .unwrap_or(true);
    let epoch = env::var("SOURCE_DATE_EPOCH")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or_else(|| {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        });
    let id = format!(
        "{}-{}{}-{epoch}",
        env!("CARGO_PKG_VERSION"),
        &git[..git.len().min(12)],
        if dirty { "-dirty" } else { "" }
    );
    let data = format!(
        "pub const BUILD_ID: &str = {id:?};
pub const GIT_COMMIT: &str = {git:?};
pub const BUILD_EPOCH: u64 = {epoch};
"
    );
    fs::write(
        PathBuf::from(env::var_os("OUT_DIR").expect("Cargo OUT_DIR")).join("build_info.rs"),
        data,
    )
    .expect("write build metadata");

    link_imgui_smash();
}

/// Links the pinned imgui-smash archive for the Switch target only. Host
/// builds and tests never see it. Nothing is downloaded here; the archive is
/// placed by `tools/prepare_imgui.py`, which verifies the SHA-256 and writes a
/// sidecar digest next to it.
fn link_imgui_smash() {
    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("switch") {
        return;
    }
    // aarch64-skyline-switch links with ld.lld. Bind this NRO's references to
    // its own global functions. Undefined NN/skyline imports stay dynamic.
    // This does not stop a later plugin from resolving an export that still
    // uses that plugin's name; hook callbacks keep the deluxe_rewrite_ prefix.
    println!("cargo:rustc-cdylib-link-arg=-Bsymbolic-functions");
    let root = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("Cargo manifest dir"));
    let dir = root.join(IMGUI_SMASH_DIR);
    let lib = dir.join(IMGUI_SMASH_LIB);
    let sidecar = dir.join(format!("{IMGUI_SMASH_LIB}.sha256"));
    println!("cargo:rerun-if-changed={}", lib.display());
    println!("cargo:rerun-if-changed={}", sidecar.display());

    let help = format!(
        "imgui-smash static archive is required for the Switch build.\n\
         expected: {}\n\
         prepare it with:\n  /usr/bin/python3 tools/prepare_imgui.py --source <path-to-libimgui_smash.a>\n\
         or build the pinned public source with /usr/bin/python3 tools/prepare_imgui.py (see docs/IMGUI_DEPENDENCY.md).",
        lib.display()
    );
    let metadata = match fs::metadata(&lib) {
        Ok(metadata) if metadata.is_file() => metadata,
        _ => panic!("{help}"),
    };
    if metadata.len() != IMGUI_SMASH_SIZE {
        panic!(
            "{} has size {} but the pinned archive is {} bytes; re-run the preparation script",
            lib.display(),
            metadata.len(),
            IMGUI_SMASH_SIZE
        );
    }
    let recorded = fs::read_to_string(&sidecar)
        .map(|text| {
            text.split_whitespace()
                .next()
                .unwrap_or_default()
                .to_ascii_lowercase()
        })
        .unwrap_or_default();
    if recorded != IMGUI_SMASH_SHA256 {
        panic!(
            "{} is missing or does not record the pinned digest {}; re-run the preparation script so the archive is verified",
            sidecar.display(),
            IMGUI_SMASH_SHA256
        );
    }
    println!("cargo:rustc-link-search=native={}", dir.display());
    // -bundle keeps the 16 MB archive out of the rlib; the cdylib link pulls
    // only the objects that are referenced (backend, cimgui, ImGui core).
    println!("cargo:rustc-link-lib=static:-bundle=imgui_smash");
}
