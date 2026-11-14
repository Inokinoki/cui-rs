//! Turns the compiled CUI version — reported by `cui-sys` through Cargo's
//! `links = "cui"` metadata channel — into capability cfgs the safe layer
//! gates version-specific APIs on:
//!
//! - `cui_ge_1_5` is set when the compiled CUI is 1.5 or newer, and so on
//!   up the ladder below. APIs added after 1.4 are `#[cfg(cui_ge_1_5)]`-gated,
//!   so one binding release compiles against every CUI in its support window.
//!
//! The full ladder is always *registered* (`rustc-check-cfg`) so the gate
//! names are known to rustc even when they are not set.

use std::env;

/// Capability ladder: one gate per minor release of the planned support
/// window. Extend as the window widens (0.2.0 adds 1.5, …).
const LADDER: [(u32, u32); 6] = [(1, 4), (1, 5), (1, 6), (1, 7), (1, 8), (1, 9)];

fn main() {
    let version = cui_version();
    let (major, minor) = parse(&version);

    for (m, n) in LADDER {
        let name = format!("cui_ge_{m}_{n}");
        println!("cargo:rustc-check-cfg=cfg({name})");
        if (major, minor) >= (m, n) {
            println!("cargo:rustc-cfg={name}");
        }
    }
}

/// The compiled CUI version from cui-sys's links metadata. Modern cargo
/// merges the metadata lines into one `DEP_CUI_METADATA` variable of
/// space-separated `key=value` pairs; older cargo exposed each key directly
/// as `DEP_CUI_VERSION`. Both are accepted here.
fn cui_version() -> String {
    if let Ok(meta) = env::var("DEP_CUI_METADATA") {
        for pair in meta.split_whitespace() {
            if let Some(v) = pair.strip_prefix("version=") {
                return v.to_string();
            }
        }
        panic!("cui-sys metadata lacks a version: {meta:?}");
    }
    env::var("DEP_CUI_VERSION").unwrap_or_else(|_| {
        panic!("cui-sys did not report a version via its links metadata; it must be a dependency")
    })
}

fn parse(version: &str) -> (u32, u32) {
    let mut parts = version.split('.');
    let major = parts.next().and_then(|p| p.parse().ok()).unwrap_or(0);
    let minor = parts.next().and_then(|p| p.parse().ok()).unwrap_or(0);
    (major, minor)
}
