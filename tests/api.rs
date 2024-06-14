use xrt::device::XRTDevice;
use xrt::kernel::XRTKernel;
use xrt::run::XRTRun;
use xrt::buffer::XRTBuffer;
use xrt::buffer::SyncDirection;
use xrt::common::ERTCommandState;
use xrt::common::XRTError;
use xrt::ffi::XCL_BO_FLAGS_NONE;
use xrt::utils::get_xclbin_path;

#[test]
fn simple_add_test() -> Result<(), XRTError> {
    std::env::set_var("XCL_EMULATION_MODE", "sw_emu");

    let mut device = XRTDevice::from_index(0).expect("Wrong device");
    device.load_xclbin(get_xclbin_path("./hls/add").as_str()).expect("xclbin err");
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
    out_buffer.sync(SyncDirection::DeviceToHost, None, 0).expect("sync err");
    out_buffer.read(&mut result, 0).expect("read err");

    // Check result
    assert_eq!(result[0], 8);
    Ok(())
}

