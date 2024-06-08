#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(improper_ctypes)]
include!("bindings_c.rs");
// tests for cpp bindings are failing
//include!("bindings_cpp.rs");
pub mod wrapper;

pub fn get_xclbin_path(name: &str) -> String {
    let mode = match std::env::var("XCL_EMULATION_MODE") {
        Ok(val) => val,
        Err(_) => String::from("hw"),
    };

    format!("{}_{}.xclbin", name, mode)
}

#[test]
fn run_kernel_raw() {
    let device_handle: xrtDeviceHandle = unsafe { xrtDeviceOpen(0) };

    assert_ne!(
        device_handle,
        std::ptr::null::<std::os::raw::c_void>() as *mut std::os::raw::c_void
    );

    let xclbin_path =
        std::ffi::CString::new(get_xclbin_path("add")).expect("creating CString for xclbin_path");

    let xclbin_handle = unsafe { xrtXclbinAllocFilename(xclbin_path.as_ptr() as *const i8) };

    assert_ne!(
        xclbin_handle,
        std::ptr::null::<std::os::raw::c_void>() as *mut std::os::raw::c_void
    );

    assert_eq!(
        unsafe { xrtDeviceLoadXclbinHandle(device_handle, xclbin_handle) },
        0,
    );

    let mut xclbin_uuid: xuid_t = [0; 16];

    assert_eq!(
        unsafe { xrtXclbinGetUUID(xclbin_handle, xclbin_uuid.as_mut_ptr()) },
        0,
    );

    let kernel_name = std::ffi::CString::new("add").expect("creating CString for kernel name");

    let add_kernel_handle: xrtKernelHandle = unsafe {
        xrtPLKernelOpen(
            device_handle,
            xclbin_uuid.as_mut_ptr(),
            kernel_name.as_ptr(),
        )
    };

    assert_ne!(
        add_kernel_handle,
        std::ptr::null::<std::os::raw::c_void>() as *mut std::os::raw::c_void
    );

    let add_kernel_run_handle: xrtRunHandle = unsafe { xrtRunOpen(add_kernel_handle) };

    assert_ne!(
        add_kernel_run_handle,
        std::ptr::null::<std::os::raw::c_void>() as *mut std::os::raw::c_void
    );

    let arg: u32 = 1;

    assert_eq!(unsafe { xrtRunSetArg(add_kernel_run_handle, 0, arg as std::ffi::c_uint) }, 0,);

    assert_eq!(unsafe { xrtRunSetArg(add_kernel_run_handle, 1, arg as std::ffi::c_uint) }, 0,);

    let group_id_handle: std::os::raw::c_int = unsafe { xrtKernelArgGroupId(add_kernel_handle, 2) };

    assert!(group_id_handle >= 0);

    let return_buffer_handle: xrtBufferHandle = unsafe {
        xrtBOAlloc(
            device_handle,
            4,
            XCL_BO_FLAGS_NONE as std::os::raw::c_ulong,
            group_id_handle as std::os::raw::c_uint,
        )
    };

    assert_eq!(
        unsafe { xrtRunSetArg(add_kernel_run_handle, group_id_handle, return_buffer_handle) },
        0,
    );

    assert_eq!(unsafe { xrtRunStart(add_kernel_run_handle) }, 0);

    assert_eq!(
        unsafe { xrtRunWait(add_kernel_run_handle) },
        ert_cmd_state_ERT_CMD_STATE_COMPLETED,
    );

    assert_eq!(
        unsafe {
            xrtBOSync(
                return_buffer_handle,
                xclBOSyncDirection_XCL_BO_SYNC_BO_FROM_DEVICE,
                4,
                0,
            )
        },
        0,
    );

    /*
    let mut result: u32 = 0;
    let result_ptr: *mut u32 = &mut result;
    assert_eq!(
        unsafe {
            xrtBORead(return_buffer_handle, result_ptr as *mut std::os::raw::c_void, 4, 0)
        },
        0,
    );

    assert_eq!(result, 2);
    */

    assert_eq! {
        unsafe {
            xrtBOFree(return_buffer_handle)
        },
        0,
    }

    assert_eq! {
        unsafe {
            xrtRunClose(add_kernel_run_handle)
        },
        0,
    }

    assert_eq! {
        unsafe {
            xrtKernelClose(add_kernel_handle)
        },
        0,
    }

    assert_eq! {
        unsafe {
            xrtDeviceClose(device_handle)
        },
        0,
    };
}
