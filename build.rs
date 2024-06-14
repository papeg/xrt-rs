use std::env;
use std::path::PathBuf;

fn main() {
    let xilinx_xrt = env::var("XILINX_XRT").expect("finding XILINX_XRT in env");
    println!("cargo:rustc-link-search={}/lib", xilinx_xrt);
    println!("cargo:rustc-link-lib=xrt_coreutil");

    let c_bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_arg(format!("-I{}/include", xilinx_xrt))
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("generating c bindings");

    let c_bindings_out_path = PathBuf::from("./src/ffi/");
    c_bindings
        .write_to_file(c_bindings_out_path.join("bindings_c.rs"))
        .expect("writing bindings!");
}
