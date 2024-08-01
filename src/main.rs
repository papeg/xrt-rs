use xrt::xclbin_reader::*;

fn main() {
    let raw = read_xclbin("a.xclbin").unwrap();
    let section_headers = get_section_data(&raw).unwrap();
    let build_metadata = get_build_metadata(&raw, &section_headers).unwrap();
    let s = build_metadata.to_string();
    println!("JSON: {}", &s);
}