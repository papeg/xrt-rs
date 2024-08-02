//! Module to read out relevant data from an xclbin file
//! Can directly convert the information into actual XRT buffers
use std::collections::HashMap;
use serde::Deserialize;
use crate::Result;
use crate::error::Error;
use crate::managed::arguments::ArgumentType;

/// This struct is what is needed to retrieve the kernel arguments from the xclbin. It is parsed to by serde json
#[derive(Debug, Deserialize)]
pub struct BuildMetadata {
    build_metadata: BuildMetadataContent,
    schema_version: HashMap<String, String>
}
#[derive(Debug, Deserialize)]
pub struct BuildMetadataContent {
    dsa: DSA,
    xclbin: XCLBIN
}
#[derive(Debug, Deserialize)]
pub struct DSA {
    board: HashMap<String, String>,
    board_id: String,
    description: String,
    feature_roms: Vec<HashMap<String, String>>,
    generated_by: HashMap<String, String>,
    name: String,
    vendor: String,
    version_major: String,
    version_minor: String,
}
#[derive(Debug, Deserialize)]
pub struct XCLBIN {
    generated_by: HashMap<String, String>,
    packaged_by: HashMap<String, String>,
    user_regions: Vec<UserRegion>
}
#[derive(Debug, Deserialize)]
pub struct UserRegion {
    base_address: String,
    instance_path: String,
    kernels: Vec<XclbinKernel>,
    name: String,
    #[serde(rename = "type")] 
    typ: String
}
#[derive(Debug, Deserialize, Clone, Copy)]
pub struct XclbinKernel {
    arguments: Vec<HashMap<String, String>>,
    name: String,
    instances: Vec<HashMap<String, String>>,
    ports: Vec<HashMap<String, String>>
}

impl BuildMetadata {
    fn get_kernel(&self, kernel_name: &str) -> Option<XclbinKernel> {
        if self.build_metadata.xclbin.user_regions.len() < 1 {
            return None;
        }
        for user_region in self.build_metadata.xclbin.user_regions {
            for kernel in user_region.kernels {
                if kernel.name == kernel_name {
                    return Some(kernel.clone());
                }
            }
        }
        None
    }
}


/// _Usage_: parse_data!(slice, target_type, range);
/// 
/// Converts a range of bytes into a larger primitive type of the given length.
/// Types must be: 
/// 
/// &Vec<u8> 
/// 
/// Any of the primitive types (std::primitive::{u,i}{8,16,32,64,128})
/// 
/// A Range<usize> of the indices
/// 
/// TODO: Trait that implements from_le_bytes on all ints ->  make templated function instead of macro
macro_rules! parse_data {
    ( $s:expr, $t:ty, $r:expr ) => {
        <$t>::from_le_bytes($s[$r.clone()].try_into().map_err(|_| Error::XclbinByteReadingError($r.start, $r.end))?)
    };
}




/// Read an xclbin from the given path and return as a bytevector. Fails if the magic string was not found at the beginning
pub fn read_xclbin(path: &str) -> Result<Vec<u8>> {
    let data = std::fs::read(path).unwrap();
    if &data[0..7] != b"xclbin2" {
        let found = std::str::from_utf8(&data[0..7]).expect("Unpacking UTF-8");
        return Err(Error::XclbinInvalidMagicString(found.to_owned()));
    }
    return Ok(data);
}

/// A section header of an xclbin file. Leaves out the name since it's irrelevant here
pub struct SectionHeader {
    pub kind: u32,
    pub offset: u64,
    pub size: u64,
}

/// Read all section headers from the bytevector
pub fn get_section_data(data: &Vec<u8>) -> Result<Vec<SectionHeader>> {
    let num_sections: u32 = parse_data!(data, std::primitive::u32, 448..452);
    let mut headers: Vec<SectionHeader> = Vec::new();
    for section_index in 0..num_sections {
        let s = 496 + (40 * section_index) as usize - 40;       // 496 is the number of bytes of the AXLF header at the start of the file; 40 is the size of a section header struct in C
        headers.push(
            SectionHeader { 
                kind: parse_data!(data, std::primitive::u32, s..s+4), 
                offset: parse_data!(data, std::primitive::u64, s+24..s+32), 
                size: parse_data!(data, std::primitive::u64, s+32..s+40) ,
            }
        );
    }
    Ok(headers)
}


/// Find out if a build metadata section exists, and if so, extract the JSON it contains
pub fn get_build_metadata(data: &Vec<u8>, headers: &Vec<SectionHeader>) -> Vec<Result<serde_json::Value>> {
    let matching = headers.iter().filter(|h| (h.kind == 14)).collect::<Vec<_>>();
    if matching.len() == 0 {
        return Vec::new(); // TODO: Change back to returning a single value instead of a vec
    }

    matching.iter().map(|m| {
        let offset = m.offset as usize;
        let size = m.size as usize;
        serde_json::from_slice::<serde_json::Value>(&data[offset..offset+size]).map_err(|e| Error::XclbinInvalidMagicString(e.to_string()))
    }).collect()
}

/// Given the build metadata as a serde_json value, look for a specific kernel and return its "arguments" json value 
pub fn extract_arguments(metadata: &serde_json::Value, kernel_name: &str) -> Result<XclbinKernel> {
    let bm: BuildMetadata = serde_json::from_str(&metadata.to_string()).unwrap(); // TODO: In future avoid this and directly parse
    bm.get_kernel(kernel_name).ok_or(Error::XclbinNoKernelOfSuchName(kernel_name.to_owned()))
}

pub fn get_argument_types(kernel: &XclbinKernel) -> Vec<ArgumentType> {
    let mut v: Vec<ArgumentType> = Vec::new();
    let kernel_args = kernel.arguments.clone();
    kernel_args.arguments.sort_by(|a, b| 
            a.get("id").unwrap().cmp(b.get("id").unwrap())
    );

    for arguments in kernel_args {
        v.push()
    }
}