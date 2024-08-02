use xrt::xclbin_reader::*;

fn main() {
    let raw = read_xclbin("vscale_f32_sw_emu.xclbin").unwrap();
    let section_headers = get_section_data(&raw).unwrap();
    let values = get_build_metadata(&raw, &section_headers);
    for value in values {
        if let Ok(v) = value {
            let args = extract_arguments(&v, "vscale_f32").unwrap();
            for arg in args {
                println!("{}  |  {}  |  {}", arg["name"], arg["type"], arg["size"]);
            }
        }
    }
}