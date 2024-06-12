#[allow(unused_imports)]
use std::collections::HashMap;
#[allow(unused_imports)]
use crate::components::common::*;
#[allow(unused_imports)]
use crate::components::device::*;
#[allow(unused_imports)]
use crate::components::kernel::*;
#[allow(unused_imports)]
use crate::components::run::*;
use crate::get_xclbin_path;

#[test]
fn test_addition() -> Result<(), XRTError> {
    
    let arglist = HashMap::from([
        (0, ArgumentType::Passed),
        (1, ArgumentType::Passed),
        (2, ArgumentType::NotRealizedBuffer(4, IOMode::Output))
    ]);

    let mut device = XRTDevice::from_index(0)?;
    device.load_xclbin(get_xclbin_path("add"))?;
    device.load_kernel("add".to_owned(), arglist)?;
    
    let kernel = device.get_kernel("add").ok_or(XRTError::GeneralError(String::from("a")))?;

    let args = HashMap::from([
        (0, Argument::Direct(2)),
        (1, Argument::Direct(3)),
        (2, Argument::BufferContent(vec![0, 0, 0, 0])) 
    ]);

    let run = kernel.create_run(args)?;
    let finished_state = run.start_run(true, true)?;

    assert_eq!(finished_state, ERTCommandState::Completed);

   Ok(()) 
}