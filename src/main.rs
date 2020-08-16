#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate rlibc;
use core::panic::PanicInfo;

mod vga_buffer;
mod serial;


/// This replaces the typical main() function.
#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("Hello World{}", "!");

    #[cfg(test)]
    test_main();

    loop {}
}


/// This function is called on panic (in regular mode).
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

/// This function is called on panic (during tests).
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    loop {}
}


/// Custom test runner for running unit and integration tests.
#[cfg(test)]
fn test_runner(tests: &[&dyn Fn()]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test();
    }
    exit_qemu(QemuExitCode::Success);
}

/// Enum specifying the exit code when running tests.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

/// Write to a specified port to exit QEMU with the
/// given exit code. The 0xf4 port is specified in
/// Cargo.toml in the metadata given to the bootloader
/// crate.
pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}


#[test_case]
fn trivial_assertion() {
    serial_print!("trivial assertion... ");
    assert_eq!(0, 1);
    serial_println!("[ok]");
}