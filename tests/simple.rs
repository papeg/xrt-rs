use xrt::device::XRTDevice;
use xrt::run::ERTCommandState;
use xrt::utils::get_xclbin_path;
use xrt::Result;

mod data;

use data::{VScaleTestData, SIZE};

fn run_vscale_simple<T: VScaleTestData + std::fmt::Debug + Copy + std::cmp::PartialEq<T>>(
) -> Result<()> {
    let xclbin_path = get_xclbin_path(&format!("./hls/vscale_{}", T::name()));

    let kernel_name = format!("vscale_{}", T::name());

    let device = XRTDevice::try_from(0)?
        .with_xclbin(&xclbin_path)?
        .with_kernel(&kernel_name)?;

    let add_kernel = device.kernel(&kernel_name)?;
    let mut add_run = add_kernel.run()?;

    let input: [T; SIZE] = [T::input(); SIZE];

    // Set args
    add_run.set_scalar_argument(0, SIZE)?;
    add_run.set_scalar_argument(1, T::scale())?;
    add_run.write_buffer_argument(2, &input, &device, &add_kernel)?;
    add_run.create_read_buffer::<T>(3, SIZE, &device, &add_kernel)?;

    // Run
    let _start_state = add_run.start()?;

    let result_state = add_run.wait_for(1000)?;
    assert_eq!(result_state, ERTCommandState::Completed);

    // Get back data
    let output: Vec<T> = add_run.read_buffer_argument(3, SIZE)?;

    // Check result
    for elem in output {
        assert_eq!(elem, T::output());
    }
    Ok(())
}

#[test]
fn run_vscale_simple_u32() -> Result<()> {
    run_vscale_simple::<u32>()
}

#[test]
fn run_vscale_simple_i32() -> Result<()> {
    run_vscale_simple::<i32>()
}

#[test]
fn run_vscale_simple_u64() -> Result<()> {
    run_vscale_simple::<u64>()
}

#[test]
fn run_vscale_simple_i64() -> Result<()> {
    run_vscale_simple::<i64>()
}

#[test]
fn run_vscale_simple_f32() -> Result<()> {
    run_vscale_simple::<f32>()
}

#[test]
fn run_vscale_simple_f64() -> Result<()> {
    run_vscale_simple::<f64>()
}
