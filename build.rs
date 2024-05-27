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

    let c_out_path = PathBuf::from("./src");
    c_bindings
        .write_to_file(c_out_path.join("bindings_c.rs"))
        .expect("writing bindings!");

    let cpp_bindings = bindgen::Builder::default()
        .header("wrapper.hpp")
        .clang_arg(format!("-I{}/include", xilinx_xrt))
        // versions below 17 have boost dependency
        .clang_arg("-std=c++17")
        // only use xrt::ip, as its not supported with the C API
        .allowlist_item("xrt::ip.*")
        .opaque_type("xrt::xclbin.*")
        .opaque_type("xrt::device.*")
        .blocklist_item("uuid_t")
        .blocklist_item("xuid_t")
        .blocklist_item("axlf.*")
        .blocklist_item("xclDeviceHandle")
        .opaque_type("std::.*")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("generating cpp bindings");

    let cpp_out_path = PathBuf::from("./src");
    cpp_bindings
        .write_to_file(cpp_out_path.join("bindings_cpp.rs"))
        .expect("writing bindings!");
}
