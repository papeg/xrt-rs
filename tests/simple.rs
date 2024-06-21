use xrt::device::XRTDevice;
use xrt::device_manager::HardwareDatatype;
use xrt::utils::get_xclbin_path;
use xrt::Result;

mod data;

use data::{VScaleTestData, SIZE};

fn run_vscale_simple<
    T: VScaleTestData + HardwareDatatype + std::fmt::Debug + Copy + std::cmp::PartialEq<T>,
>() -> Result<()> {
    let xclbin_path = get_xclbin_path(&format!("./hls/vscale_{}", T::name()));

    let kernel_name = format!("vscale_{}", T::name());

    let input: [T; SIZE] = [T::input(); SIZE];
    let mut output: [T; SIZE] = [T::zero(); SIZE];

    let device = XRTDevice::try_from(0)?
        .manage()
        .with_xclbin(&xclbin_path)?
        .with_kernel(&kernel_name)?;

    device
        .run(&kernel_name)?
        .set_scalar_input(0, SIZE as u32)?
        .set_scalar_input(1, T::scale())?
        .set_buffer_input(2, &input)?
        .prepare_output_buffer::<T>(3, SIZE)?
        .start()?
        .wait_for(2000)?
        .read_output(3, &mut output)?;

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
