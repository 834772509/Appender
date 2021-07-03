extern crate embed_resource;

use std::process::Command;

fn main() {
    embed_resource::compile("./resource/resource.rc");

    Command::new("lib").args(&["YY_Thunks_for_WinXP.obj", "/out:YY_Thunks_for_WinXP.lib"]).status().unwrap();
    println!("cargo:rustc-link-lib=static=YY_Thunks_for_WinXP");
}
