use xrt::buffer::SyncDirection;
use xrt::buffer::XRTBuffer;
use xrt::device::XRTDevice;
use xrt::ffi::XCL_BO_FLAGS_NONE;
use xrt::kernel::XRTKernel;
use xrt::run::ERTCommandState;
use xrt::run::XRTRun;
use xrt::utils::get_xclbin_path;
use xrt::Result;

mod data;

use data::{VScaleTestData, SIZE};

fn run_vscale_native<T: VScaleTestData + std::fmt::Debug + Copy + std::cmp::PartialEq<T>>(
) -> Result<()> {
    std::env::set_var("XCL_EMULATION_MODE", "sw_emu");

    let mut device = XRTDevice::from_index(0)?;
    device.load_xclbin(get_xclbin_path(&format!("./hls/vscale_{}", T::name())).as_str())?;
    let add_kernel = XRTKernel::new(&format!("vscale_{}", T::name()), &device)?;
    let mut add_run = XRTRun::new(&add_kernel)?;
    let in_buffer = XRTBuffer::new(
        &device,
        SIZE * std::mem::size_of::<T>(),
        XCL_BO_FLAGS_NONE,
        add_kernel.get_memory_group_for_argument(2)?,
    )?;
    let out_buffer = XRTBuffer::new(
        &device,
        SIZE * std::mem::size_of::<T>(),
        XCL_BO_FLAGS_NONE,
        add_kernel.get_memory_group_for_argument(3)?,
    )?;

    let input: [T; SIZE] = [T::input(); SIZE];
    in_buffer.write(&input, 0)?;
    in_buffer.sync(SyncDirection::HostToDevice, None, 0)?;

    // Set args
    add_run.set_argument(0, SIZE)?;
    add_run.set_argument(1, T::scale())?;
    add_run.set_argument(2, in_buffer.get_handle().unwrap())?;
    add_run.set_argument(3, out_buffer.get_handle().unwrap())?;

    // Run
    let result_state = add_run.start(true, 1000)?;
    assert_eq!(result_state, ERTCommandState::Completed);

    // Get back data
    let mut output: [T; SIZE] = [T::zero(); SIZE];
    out_buffer.sync(SyncDirection::DeviceToHost, None, 0)?;
    out_buffer.read(&mut output, 0)?;

    // Check result
    for elem in output {
        assert_eq!(elem, T::output());
    }
    Ok(())
}

#[test]
fn run_vscale_native_u32() -> Result<()> {
    run_vscale_native::<u32>()
}

#[test]
fn run_vscale_native_i32() -> Result<()> {
    run_vscale_native::<i32>()
}

#[test]
fn run_vscale_native_u64() -> Result<()> {
    run_vscale_native::<u64>()
}

#[test]
fn run_vscale_native_i64() -> Result<()> {
    run_vscale_native::<i64>()
}

#[test]
fn run_vscale_native_f32() -> Result<()> {
    run_vscale_native::<f32>()
}

#[test]
fn run_vscale_native_f64() -> Result<()> {
    run_vscale_native::<f64>()
}
