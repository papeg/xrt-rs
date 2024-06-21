use xrt::ffi::*;
use xrt::utils::get_xclbin_path;

mod data;
use data::{VScaleTestData, SIZE};

fn run_vscale_raw_sync<T: VScaleTestData + std::fmt::Debug + Copy + std::cmp::PartialEq<T>>() {
    let device_handle: xrtDeviceHandle = unsafe { xrtDeviceOpen(0) };

    assert_ne!(
        device_handle,
        std::ptr::null::<std::os::raw::c_void>() as *mut std::os::raw::c_void
    );

    let xclbin_path =
        std::ffi::CString::new(get_xclbin_path(&format!("./hls/vscale_{}", T::name())))
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

    let kernel_name = std::ffi::CString::new(format!("vscale_{}", T::name()))
        .expect("creating CString for kernel name");

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
        unsafe { xrtRunSetArg(add_kernel_run_handle, 0, SIZE as std::ffi::c_uint) },
        0,
    );

    assert_eq!(
        unsafe { xrtRunSetArg(add_kernel_run_handle, 1, T::scale()) },
        0,
    );

    let mut input: [T; SIZE] = [T::input(); SIZE];
    let input_ptr: *mut T = &mut input[0];

    let input_group_id: std::os::raw::c_int = unsafe { xrtKernelArgGroupId(add_kernel_handle, 2) };

    assert!(input_group_id >= 0);

    let input_buffer_handle: xrtBufferHandle = unsafe {
        xrtBOAlloc(
            device_handle,
            SIZE * std::mem::size_of::<T>(),
            XCL_BO_FLAGS_NONE as std::os::raw::c_ulong,
            input_group_id as std::os::raw::c_uint,
        )
    };

    assert_eq!(
        unsafe {
            xrtBOWrite(
                input_buffer_handle,
                input_ptr as *mut std::os::raw::c_void,
                SIZE * std::mem::size_of::<T>(),
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
                SIZE * std::mem::size_of::<T>(),
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
            SIZE * std::mem::size_of::<T>(),
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
                SIZE * std::mem::size_of::<T>(),
                0,
            )
        },
        0,
    );

    let mut output: [T; SIZE] = [T::zero(); SIZE];
    let output_ptr: *mut T = &mut output[0];
    assert_eq!(
        unsafe {
            xrtBORead(
                output_buffer_handle,
                output_ptr as *mut std::os::raw::c_void,
                SIZE * std::mem::size_of::<T>(),
                0,
            )
        },
        0,
    );

    for elem in output {
        assert_eq!(elem, T::output());
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

#[test]
fn run_vscale_raw_sync_u32() {
    run_vscale_raw_sync::<u32>();
}

#[test]
fn run_vscale_raw_sync_i32() {
    run_vscale_raw_sync::<i32>();
}

#[test]
fn run_vscale_raw_sync_u64() {
    run_vscale_raw_sync::<u64>();
}

#[test]
fn run_vscale_raw_sync_i64() {
    run_vscale_raw_sync::<i64>();
}

#[test]
fn run_vscale_raw_sync_f32() {
    run_vscale_raw_sync::<f32>();
}

#[test]
fn run_vscale_raw_sync_f64() {
    run_vscale_raw_sync::<f64>();
}

fn run_vscale_raw_memmap<T: VScaleTestData + std::fmt::Debug + Copy + std::cmp::PartialEq<T>>() {
    let device_handle: xrtDeviceHandle = unsafe { xrtDeviceOpen(0) };

    assert_ne!(
        device_handle,
        std::ptr::null::<std::os::raw::c_void>() as *mut std::os::raw::c_void
    );

    let xclbin_path =
        std::ffi::CString::new(get_xclbin_path(&format!("./hls/vscale_{}", T::name())))
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

    let kernel_name = std::ffi::CString::new(format!("vscale_{}", T::name()))
        .expect("creating CString for kernel name");

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
        unsafe { xrtRunSetArg(add_kernel_run_handle, 0, SIZE as std::ffi::c_uint) },
        0,
    );

    assert_eq!(
        unsafe { xrtRunSetArg(add_kernel_run_handle, 1, T::scale()) },
        0,
    );

    let input_group_id: std::os::raw::c_int = unsafe { xrtKernelArgGroupId(add_kernel_handle, 2) };

    assert!(input_group_id >= 0);

    let input_buffer_handle: xrtBufferHandle = unsafe {
        xrtBOAlloc(
            device_handle,
            SIZE * std::mem::size_of::<T>(),
            XCL_BO_FLAGS_NONE as std::os::raw::c_ulong,
            input_group_id as std::os::raw::c_uint,
        )
    };

    let input_ptr: *mut std::os::raw::c_void = unsafe { xrtBOMap(input_buffer_handle) };

    assert_ne!(
        input_ptr,
        std::ptr::null::<std::os::raw::c_void>() as *mut std::os::raw::c_void
    );

    let input: [T; SIZE] = [T::input(); SIZE];

    for i in 0..input.len() {
        unsafe { *(input_ptr.wrapping_add(i * std::mem::size_of::<T>()) as *mut T) = input[i] };
    }

    assert_eq!(
        unsafe { xrtRunSetArg(add_kernel_run_handle, 2, input_buffer_handle) },
        0,
    );

    let output_group_id: std::os::raw::c_int = unsafe { xrtKernelArgGroupId(add_kernel_handle, 3) };

    assert!(output_group_id >= 0);

    let output_buffer_handle: xrtBufferHandle = unsafe {
        xrtBOAlloc(
            device_handle,
            SIZE * std::mem::size_of::<T>(),
            XCL_BO_FLAGS_NONE as std::os::raw::c_ulong,
            output_group_id as std::os::raw::c_uint,
        )
    };

    let output_ptr: *mut std::os::raw::c_void = unsafe { xrtBOMap(output_buffer_handle) };

    assert_ne!(
        output_ptr,
        std::ptr::null::<std::os::raw::c_void>() as *mut std::os::raw::c_void
    );

    assert_eq!(
        unsafe { xrtRunSetArg(add_kernel_run_handle, 3, output_buffer_handle) },
        0,
    );

    assert_eq!(unsafe { xrtRunStart(add_kernel_run_handle) }, 0);

    assert_eq!(
        unsafe { xrtRunWait(add_kernel_run_handle) },
        ert_cmd_state_ERT_CMD_STATE_COMPLETED,
    );

    let mut output: [T; SIZE] = [T::zero(); SIZE];

    for i in 0..output.len() {
        output[i] = unsafe { *(output_ptr.wrapping_add(i * std::mem::size_of::<T>()) as *mut T) };
    }

    for elem in output {
        assert_eq!(elem, T::output());
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

#[test]
fn run_vscale_raw_memmap_u32() {
    run_vscale_raw_memmap::<u32>();
}

#[test]
fn run_vscale_raw_memmap_i32() {
    run_vscale_raw_memmap::<i32>();
}

#[test]
fn run_vscale_raw_memmap_u64() {
    run_vscale_raw_memmap::<u64>();
}

#[test]
fn run_vscale_raw_memmap_i64() {
    run_vscale_raw_memmap::<i64>();
}

#[test]
fn run_vscale_raw_memmap_f32() {
    run_vscale_raw_memmap::<f32>();
}

#[test]
fn run_vscale_raw_memmap_f64() {
    run_vscale_raw_memmap::<f64>();
}
