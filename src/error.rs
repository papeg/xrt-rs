#[derive(Debug)]
pub enum Error {
    GeneralError(String),
    DeviceOpenError,
    UnopenedDeviceError,
    CStringCreationError,
    XclbinFileAllocError,
    XclbinLoadError,
    XclbinUUIDRetrievalError,
    DeviceNotReadyError,
    KernelCreationError,
    KernelNotLoadedYetError,
    KernelArgRtrvError,
    BOCreationError,
    RunCreationError,
    RunNotCreatedYetError,
    SetRunArgError,
    BONotCreatedYet,
    BOWriteError,
    BOReadError,
    BOSyncError,
}

pub type Result<T> = std::result::Result<T, Error>;

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
