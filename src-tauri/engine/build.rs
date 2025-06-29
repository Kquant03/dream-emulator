// Build script for Zig integration

use std::process::Command;

fn main() {
    // Build Zig modules if Zig is installed
    if has_zig() {
        println!("cargo:rerun-if-changed=zig/physics.zig");
        
        Command::new("zig")
            .args(&["build-lib", "zig/physics.zig", "-O", "ReleaseFast"])
            .status()
            .expect("Failed to build Zig physics module");
            
        println!("cargo:rustc-link-lib=static=physics");
    }
}

fn has_zig() -> bool {
    Command::new("zig").arg("version").output().is_ok()
}
