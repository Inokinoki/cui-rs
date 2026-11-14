//! Compiles CUI for the `cui-sys` crate.
//!
//! The C sources are not committed to this repository. `UPSTREAM.toml` pins
//! an upstream release (version, tag and the SHA-256 of its release
//! assets); this script downloads the pre-amalgamated `cui.h`/`cui.c` from
//! that release, verifies the digests and the version embedded in the
//! header, caches them under `OUT_DIR`, and compiles the single `cui.c`
//! with the `cc` crate.
//!
//! Without network access, pre-populate the offline cache with
//! `./vendor.sh --fetch` (the same verified files, stored in `cui-sys/c/`,
//! git-ignored) or amalgamate a local upstream checkout with
//! `./vendor.sh <path>` — an existing `cui-sys/c/` always takes precedence
//! and is compiled as-is.

use sha2::{Digest, Sha256};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const HEADER: &str = "cui.h";
const SOURCE: &str = "cui.c";

struct Pin {
    version: String,
    tag: String,
    base_url: String,
    sha256_h: String,
    sha256_c: String,
}

impl Pin {
    fn parse(src: &str) -> Pin {
        let field = |key: &str| -> String {
            src.lines()
                .find_map(|l| {
                    let l = l.trim();
                    let rest = l.strip_prefix(key)?.trim_start().strip_prefix('=')?;
                    rest.trim().trim_matches('"').to_string().into()
                })
                .unwrap_or_else(|| panic!("UPSTREAM.toml: missing `{key}`"))
        };
        Pin {
            version: field("version"),
            tag: field("tag"),
            base_url: field("base_url"),
            sha256_h: field("sha256_cui_h"),
            sha256_c: field("sha256_cui_c"),
        }
    }
}

fn main() {
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let pin = Pin::parse(
        &fs::read_to_string(manifest.join("UPSTREAM.toml"))
            .expect("UPSTREAM.toml is missing; it pins the upstream release"),
    );

    // An offline cache in cui-sys/c/ (see vendor.sh) takes precedence and is
    // trusted as-is: it exists precisely to build without network access.
    let local_cache = manifest.join("c");
    let dir = if local_cache.join(SOURCE).is_file() && local_cache.join(HEADER).is_file() {
        let version = header_version(&fs::read_to_string(local_cache.join(HEADER)).unwrap());
        if version != pin.version {
            panic!(
                "cui-sys/c/ holds CUI {version} but UPSTREAM.toml pins {}.\n\
                 Re-run `./vendor.sh --pin {}` or `./vendor.sh --fetch`, or remove cui-sys/c/.",
                pin.version, pin.tag
            );
        }
        local_cache
    } else {
        fetch_and_verify(&pin)
    };

    let version = header_version(&fs::read_to_string(dir.join(HEADER)).unwrap());
    println!("cargo:rustc-env=CUI_VERSION_STRING={version}");

    let mut b = cc::Build::new();
    b.file(dir.join(SOURCE));
    b.include(&dir);
    b.flag_if_supported("-std=c99");
    b.warnings(false);

    if !cfg!(feature = "default-fonts") {
        b.define("CUI_NO_DEFAULT_FONTS", None);
    }
    if !cfg!(feature = "stb-image") {
        b.define("CUI_NO_STB_IMAGE", None);
    }
    if !cfg!(feature = "app") {
        b.define("CUI_NO_APP", None);
    }
    if cfg!(feature = "text-shaping") {
        b.define("CUI_TEXT_SHAPING", None);
    }

    let target = env::var("TARGET").unwrap();
    match target.as_str() {
        // macOS: the window backend is Objective-C; the amalgamation must be
        // compiled as ObjC (upstream CI uses `-x objective-c -fobjc-arc`).
        t if t.contains("apple") => {
            b.flag("-x");
            b.flag("objective-c");
            b.flag("-fobjc-arc");
            b.define("GL_SILENCE_DEPRECATION", None);
            for fw in ["Cocoa", "QuartzCore", "OpenGL"] {
                println!("cargo:rustc-link-lib=framework={fw}");
            }
        }
        t if t.contains("windows") => {
            for lib in ["user32", "gdi32", "opengl32", "shell32", "imm32", "dwmapi", "comdlg32", "ole32"] {
                println!("cargo:rustc-link-lib=dylib={lib}");
            }
        }
        // Linux/BSD (X11 backend; SDL2/GLFW builds are not wired up here yet).
        _ => {
            println!("cargo:rustc-link-lib=dylib=X11");
            println!("cargo:rustc-link-lib=dylib=dl");
            println!("cargo:rustc-link-lib=dylib=m");
        }
    }

    b.compile("cui");
    println!("cargo:rerun-if-changed=UPSTREAM.toml");
    println!("cargo:rerun-if-changed=build.rs");
}

/// Downloads the pinned release assets into `OUT_DIR`, verifies both SHA-256
/// digests and the version embedded in the header, and returns the cache
/// directory. The cache survives across builds; only `cargo clean` clears it.
fn fetch_and_verify(pin: &Pin) -> PathBuf {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let dir = out_dir.join(format!("cui-{}", pin.version));
    if dir.join(".verified").exists() {
        return dir;
    }

    let tmp = out_dir.join(format!(".cui-{}-{}.tmp", pin.version, std::process::id()));
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(&tmp).expect("could not create the download cache directory");

    for (name, expected) in [(HEADER, &pin.sha256_h), (SOURCE, &pin.sha256_c)] {
        let url = format!("{}/{}/{name}", pin.base_url, pin.tag);
        let dest = tmp.join(name);
        download(&url, &dest);
        let bytes = fs::read(&dest).unwrap_or_else(|e| panic!("downloaded {url} is unreadable: {e}"));
        let got = hex_sha256(&bytes);
        if got != *expected {
            panic!(
                "SHA-256 mismatch for {url}:\n  expected {expected}\n  got      {got}\n\
                 Refusing to compile. If upstream re-published this asset, refresh the\n\
                 pin with `./vendor.sh --pin {}`.",
                pin.tag
            );
        }
    }

    let header = fs::read_to_string(tmp.join(HEADER)).unwrap();
    let version = header_version(&header);
    if version != pin.version {
        panic!(
            "the header for {} says CUI {version}, but UPSTREAM.toml pins {}.\n\
             Refresh the pin with `./vendor.sh --pin {}`.",
            pin.tag, pin.version, pin.tag
        );
    }

    fs::create_dir_all(&dir).unwrap();
    fs::rename(tmp.join(HEADER), dir.join(HEADER)).unwrap();
    fs::rename(tmp.join(SOURCE), dir.join(SOURCE)).unwrap();
    fs::write(dir.join(".verified"), format!("{}\n", pin.version)).unwrap();
    let _ = fs::remove_dir_all(&tmp);
    dir
}

fn download(url: &str, dest: &Path) {
    let status = Command::new("curl")
        .args(["--fail", "--location", "--proto", "=https", "--silent", "--show-error", "--output"])
        .arg(dest)
        .arg(url)
        .status()
        .unwrap_or_else(|e| {
            panic!(
                "could not run curl ({e}). Install curl, or pre-populate the offline\n\
                 cache while online: `./vendor.sh --fetch`"
            )
        });
    if !status.success() {
        panic!(
            "downloading {url} failed — check your network connection, or run\n\
             `./vendor.sh --fetch` while online"
        );
    }
}

fn hex_sha256(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    format!("{:x}", h.finalize())
}

fn header_version(header: &str) -> String {
    header
        .lines()
        .find_map(|l| l.strip_prefix("#define CUI_VERSION_STRING "))
        .map(|rest| rest.trim().trim_matches('"').to_string())
        .unwrap_or_else(|| panic!("CUI_VERSION_STRING not found in the downloaded cui.h"))
}
