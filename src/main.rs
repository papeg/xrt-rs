extern crate xrt_proc_macro;
use xrt_proc_macro::*;

#[kernel("hls/vscale_u32_sw_emu.xclbin", "vscale_u32")]
pub struct MyStruct;

fn main() {
    let ms = MyStruct;
    println!("{}", ms.ans());
}
