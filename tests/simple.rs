extern crate xrt_proc_macro;
use xrt_proc_macro::*;

#[kernel("hls/vscale_u32_sw_emu.xclbin", "vscale_u32")]
pub struct VScaleU32;

#[test]
fn run_vscale_simple_u32() {
    let ms = VScaleU32;
    println!("{}", ms.ans());
}
