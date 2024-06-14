//! xrt-rs is a wrapper around AMDs XRT C-Bindings, aiming to provide higher level abstraction and error handling.
//! 
//! # Example
//! This is roughly how one would use the wrapper to interact with a datacenter FPGA:
//! ```
//! let mut device = XRTDevice::from_index(0)?;
//! device.load_xclbin("my_xclbin.xclbin")?;
//! device.load_kernel("add_kernel")?;
//! devce.run_kernel(...)?;
//! ```
//! 
//! Alternatively, builder-style constructors are also available

#![allow(clippy::all)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(improper_ctypes)]

use components::buffers::XRTBuffer;
use components::device::XRTDevice;
use components::common::*;
use components::kernel::XRTKernel;
use components::run::XRTRun;
include!("bindings_c.rs");
// tests for cpp bindings are failing
//include!("bindings_cpp.rs");
pub mod components;

pub fn get_xclbin_path(name: &str) -> String {
    let mode = match std::env::var("XCL_EMULATION_MODE") {
        Ok(val) => val,
        Err(_) => String::from("hw"),
    };

    format!("{}_{}.xclbin", name, mode)
}

#[test]
fn simple_add_test() -> Result<(), XRTError> {
    let mut device = XRTDevice::from_index(0).expect("Wrong device");
    device.load_xclbin(get_xclbin_path("add").as_str()).expect("xclbin err");
    let add_kernel = XRTKernel::new("add", &device).expect("Kernel error");
    let mut add_run = XRTRun::new(&add_kernel).expect("Run error"); 
    let out_buffer = XRTBuffer::new(&device, 4, XCL_BO_FLAGS_NONE, add_kernel.get_memory_group_for_argument(2).expect("mem group err")).expect("buffer error");

    // Set args
    add_run.set_argument(0, 3).expect("set arg 1 err"); 
    add_run.set_argument(1, 5).expect("set arg 2 err");
    add_run.set_argument(2, out_buffer.get_handle().unwrap()).expect("set arg 3 err");

    // Run
    let result_state = add_run.start(true, 1000).expect("start run err");
    assert_eq!(result_state, ERTCommandState::Completed);

    // Get back data
    let mut result: [u32; 1] = [0];
    out_buffer.sync(components::buffers::SyncDirection::DeviceToHost, None, 0).expect("sync err");
    out_buffer.read(&mut result, 0).expect("read err");

    // Check result
    assert_eq!(result[0], 8);
    Ok(())
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

    assert_eq!(unsafe { xrtRunSetArg(add_kernel_run_handle, 0, arg) }, 0,);

    assert_eq!(unsafe { xrtRunSetArg(add_kernel_run_handle, 1, arg) }, 0,);

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
        unsafe { xrtRunSetArg(add_kernel_run_handle, 2, return_buffer_handle) },
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

    
    let mut result: u32 = 0;
    let result_ptr: *mut u32 = &mut result;
    assert_eq!(
        unsafe {
            xrtBORead(return_buffer_handle, result_ptr as *mut std::os::raw::c_void, 4, 0)
        },
        0,
    );

    assert_eq!(result, 2);


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
