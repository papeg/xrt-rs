use xrt::xclbin_reader::*;

extern crate xrt_proc_macro;
use xrt_proc_macro::*;

#[kernel("pathto.xclbin", "vscale_u32")]
pub struct MyStruct;

fn main() {
    let values = get_arguments("hls/vscale_f32_sw_emu.xclbin", "vscale_f32").unwrap();
    for arg in values {
        println!("{}  |  {}  |  {}", arg["name"], arg["type"], arg["size"]);
    }

    let ms = MyStruct;
    println!("{}", ms.ans());
}