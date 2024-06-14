use xrt::buffer::SyncDirection;
use xrt::buffer::XRTBuffer;
use xrt::device::XRTDevice;
use xrt::ffi::XCL_BO_FLAGS_NONE;
use xrt::kernel::XRTKernel;
use xrt::run::ERTCommandState;
use xrt::run::XRTRun;
use xrt::utils::get_xclbin_path;
use xrt::Result;

#[test]
fn simple_vscale_u32_test() -> Result<()> {
    std::env::set_var("XCL_EMULATION_MODE", "sw_emu");

    let mut device = XRTDevice::from_index(0)?;
    device.load_xclbin(get_xclbin_path("./hls/vscale_u32").as_str())?;
    let add_kernel = XRTKernel::new("vscale_u32", &device)?;
    let mut add_run = XRTRun::new(&add_kernel)?;
    let in_buffer = XRTBuffer::new(
        &device,
        16 * 4,
        XCL_BO_FLAGS_NONE,
        add_kernel.get_memory_group_for_argument(2)?,
    )?;
    let out_buffer = XRTBuffer::new(
        &device,
        16 * 4,
        XCL_BO_FLAGS_NONE,
        add_kernel.get_memory_group_for_argument(3)?,
    )?;

    let input: [u32; 16] = [7; 16];
    in_buffer.write(&input, 0)?;
    in_buffer.sync(SyncDirection::HostToDevice, None, 0)?;

    // Set args
    add_run.set_argument(0, 16)?;
    add_run.set_argument(1, 6)?;
    add_run.set_argument(2, in_buffer.get_handle().unwrap())?;
    add_run.set_argument(3, out_buffer.get_handle().unwrap())?;

    // Run
    let result_state = add_run.start(true, 1000)?;
    assert_eq!(result_state, ERTCommandState::Completed);

    // Get back data
    let mut output: [u32; 16] = [0; 16];
    out_buffer.sync(SyncDirection::DeviceToHost, None, 0)?;
    out_buffer.read(&mut output, 0)?;

    // Check result
    for elem in output {
        assert_eq!(elem, 42);
    }
    Ok(())
}
