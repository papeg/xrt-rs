pub fn get_xclbin_path(name: &str) -> String {
    let mode = match std::env::var("XCL_EMULATION_MODE") {
        Ok(val) => val,
        Err(_) => String::from("hw"),
    };

    format!("{}_{}.xclbin", name, mode)
}

pub fn is_null(handle: *mut std::os::raw::c_void) -> bool {
    handle == (std::ptr::null::<std::os::raw::c_void>() as *mut std::os::raw::c_void)
}
