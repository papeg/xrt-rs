use xrt::xclbin_reader::*;

fn main() {
    let raw = read_xclbin("vscale_f32_sw_emu.xclbin").unwrap();
    let section_headers = get_section_data(&raw).unwrap();
    let values = get_build_metadata(&raw, &section_headers);
    for value in values {
        match value {
            Ok(v) => println!("JSON: {}\n\n", serde_json::to_string_pretty(&v).unwrap()),
            Err(e) => println!("ERR: {:#?}", e)
        };
    }
}