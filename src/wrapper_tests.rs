#[test]
fn emu_open_device_test() -> Result<(), XRTError> {
    let device = XRTDevice::from_index(0)?;
    assert!(device.device_handle.is_some());
    Ok(())
}

#[test]
fn emu_open_device_load_xclbin_test() -> Result<(), XRTError> {
    use crate::get_xclbin_path;

    let mut device = XRTDevice::from_index(0)?;
    assert!(device.device_handle.is_some());
    let xclbin_path = get_xclbin_path("add");
    device.load_xclbin(&xclbin_path)?;
    assert!(device.xclbin_handle.is_some());
    assert!(device.xclbin_uuid.is_some());

    Ok(())
}

#[test]
fn emu_open_device_load_xclbin_builder_test() -> Result<(), XRTError> {
    use crate::get_xclbin_path;

    let xclbin_path = get_xclbin_path("add");
    let device = XRTDevice::from_index(0)?
        .with_xclbin(&xclbin_path)?
        .with_kernel("add")?;

    assert!(device.device_handle.is_some());
    assert!(device.xclbin_handle.is_some());
    assert!(device.xclbin_uuid.is_some());

    Ok(())
}
