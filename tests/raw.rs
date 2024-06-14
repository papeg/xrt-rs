use xrt::ffi::*;
use xrt::utils::get_xclbin_path;

#[test]
fn run_kernel_raw() {
    let device_handle: xrtDeviceHandle = unsafe { xrtDeviceOpen(0) };

    assert_ne!(
        device_handle,
        std::ptr::null::<std::os::raw::c_void>() as *mut std::os::raw::c_void
    );

    let xclbin_path = std::ffi::CString::new(get_xclbin_path("./hls/vscale_u32"))
        .expect("creating CString for xclbin_path");

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

    let kernel_name =
        std::ffi::CString::new("vscale_u32").expect("creating CString for kernel name");

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

    assert_eq!(
        unsafe { xrtRunSetArg(add_kernel_run_handle, 0, 16 as std::ffi::c_uint) },
        0,
    );

    assert_eq!(
        unsafe { xrtRunSetArg(add_kernel_run_handle, 1, 6 as std::ffi::c_uint) },
        0,
    );

    let mut input: [u32; 16] = [7; 16];
    let input_ptr: *mut u32 = &mut input[0];

    let input_group_id: std::os::raw::c_int = unsafe { xrtKernelArgGroupId(add_kernel_handle, 2) };

    assert!(input_group_id >= 0);

    let input_buffer_handle: xrtBufferHandle = unsafe {
        xrtBOAlloc(
            device_handle,
            16 * 4,
            XCL_BO_FLAGS_NONE as std::os::raw::c_ulong,
            input_group_id as std::os::raw::c_uint,
        )
    };

    assert_eq!(
        unsafe {
            xrtBOWrite(
                input_buffer_handle,
                input_ptr as *mut std::os::raw::c_void,
                16 * 4,
                0,
            )
        },
        0,
    );

    assert_eq!(
        unsafe {
            xrtBOSync(
                input_buffer_handle,
                xclBOSyncDirection_XCL_BO_SYNC_BO_TO_DEVICE,
                16 * 4,
                0,
            )
        },
        0,
    );

    assert_eq!(
        unsafe { xrtRunSetArg(add_kernel_run_handle, 2, input_buffer_handle) },
        0,
    );

    let output_group_id: std::os::raw::c_int = unsafe { xrtKernelArgGroupId(add_kernel_handle, 3) };

    assert!(output_group_id >= 0);

    let output_buffer_handle: xrtBufferHandle = unsafe {
        xrtBOAlloc(
            device_handle,
            16 * 4,
            XCL_BO_FLAGS_NONE as std::os::raw::c_ulong,
            output_group_id as std::os::raw::c_uint,
        )
    };

    assert_eq!(
        unsafe { xrtRunSetArg(add_kernel_run_handle, 3, output_buffer_handle) },
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
                output_buffer_handle,
                xclBOSyncDirection_XCL_BO_SYNC_BO_FROM_DEVICE,
                16 * 4,
                0,
            )
        },
        0,
    );

    let mut output: [u32; 16] = [0; 16];
    let output_ptr: *mut u32 = &mut output[0];
    assert_eq!(
        unsafe {
            xrtBORead(
                output_buffer_handle,
                output_ptr as *mut std::os::raw::c_void,
                16 * 4,
                0,
            )
        },
        0,
    );

    for elem in output {
        assert_eq!(elem, 6 * 7);
    }

    assert_eq! {
        unsafe {
            xrtBOFree(output_buffer_handle)
        },
        0,
    }

    assert_eq! {
        unsafe {
            xrtBOFree(input_buffer_handle)
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
