//! Module to read out relevant data from an xclbin file
//! Can directly convert the information into actual XRT buffers
use crate::Result;
use crate::error::Error;

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
pub fn get_build_metadata(data: &Vec<u8>, headers: &Vec<SectionHeader>) -> Result<serde_json::Value> {
    let matching = headers.iter().filter(|h| h.kind == 14).collect::<Vec<_>>();
    if matching.len() == 0 {
        return Err(Error::XclbinNoBuildMetadataSection);
    }

    // Better be explicit than have everything be a oneliner
    let offset = matching[0].offset as usize;
    let size = matching[0].size as usize;
    serde_json::from_slice::<serde_json::Value>(&data[offset..offset+size]).map_err(|e| Error::XclbinInvalidMagicString(e.to_string()))
}


// Read the XCLBIN file and create buffers and scalar arguments accordingly. This produces an argumentMapping that
// a kernel can use to check whether its supplied arguments are correct. It also avoids having to ask the user what arguments
// are needed and of what type
/*
pub fn conjure_kernel_arguments(kernel_name: &str) -> Result<ArgumentMapping> {

}
*/