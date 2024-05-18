use std::env;
use std::path::PathBuf;

fn main() {
    let xilinx_xrt = env::var("XILINX_XRT").expect("finding XILINX_XRT in env");
    println!("cargo:rustc-link-search={}/lib", xilinx_xrt);
    println!("cargo:rustc-link-lib=xrt_coreutil");

    let bindings = bindgen::Builder::default()
        .header("wrapper.hpp")
        .clang_arg(format!("-I{}/include", xilinx_xrt))
        // versions below 17 have boost dependency
        .clang_arg("-std=c++17")
        .allowlist_item("xrt::.*")
        .allowlist_item("xrt_core::.*")
        .allowlist_item("info::.*")
        .opaque_type("std::.*")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("generating bindings");

    let out_path = PathBuf::from("./src");
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("writing bindings!");
}
