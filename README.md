# xrt-rs
xrt-rs provides a wrapper around the C-API of the Xilinx Runtime (XRT) used for communication between AMD FPGAs / AI Engines and their host. The library offers a thin wrapper (called _native_), that simply translates the C API into safe Rust, as well as a more abstract layer (called _simple_), that automatically takes care of the details to provide an easier to use interface for simpler applications. The intermediate goal is to read the xclbin file at compile time to leverage rust type checking for using the API.

## Installation
To install simply add this repository or crate as a dependency to your `Cargo.toml`.

In case linking fails, add the XRT libs to your library path:
```
export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:/opt/xilinx/xrt/lib"
or
export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:$XILINX_XRT"
```

## Usage
The native API can be used from `xrt::native::*`. There is a wrapper for all the relevant objects: Device, Kernel, Run, Buffer. The simpler API can be used from `xrt::managed::*`. Take a look at the tests to get an example how to use it.

## Testing
Currently the tests can not be run in parallel. 

If you want to run them in software emulation, you need to set the regular env flag.

```
XCL_EMULATION_MODE=sw_emu cargo test -- --test-threads=1
```

## TODOs
- [ ] More detailed error reporting
    - [ ] parse internal error codes
    - [ ] more hierachical structure of custom errors
    - [ ] impl Error trait
- [ ] Abstract layer
- [ ] Performance considerations
    - [ ] buffer reusage
- [ ] Detailed testing
- [ ] Find a way to use xrt::ip (only accessible from CPP API)
