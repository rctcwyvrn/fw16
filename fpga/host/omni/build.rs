// Register constants come from the generated C header (gateware/regmap →
// host/include/omni_regs.h) through bindgen, so the Rust offsets are the ones
// the shell RTL decodes. The header is plain `#define`s with no #includes, so
// libclang (LIBCLANG_PATH, set by flake.nix) needs no system headers.

use std::env;
use std::path::PathBuf;

fn main() {
    let header =
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("../include/omni_regs.h");
    println!("cargo:rerun-if-changed={}", header.display());

    let bindings = bindgen::Builder::default()
        .header(header.to_str().unwrap())
        .allowlist_var("OMNI_.*")
        .generate()
        .expect("bindgen on omni_regs.h");

    let out = PathBuf::from(env::var("OUT_DIR").unwrap()).join("omni_regs.rs");
    bindings.write_to_file(out).expect("write omni_regs.rs");
}
