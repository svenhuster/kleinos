#![no_std]
#![no_main]
#![warn(clippy::missing_safety_doc)]
#![warn(clippy::undocumented_unsafe_blocks)]
#![warn(unsafe_op_in_unsafe_fn)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::tests::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use kleinos::{
    busy_spin, println,
    qemu::{QemuExitCode, qemu_exit},
};

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    use core::fmt::Write;

    // Brute-force access to VGA to print panic message
    let mut screen = kleinos::vga::VgaScreen::new();
    write!(screen, "\nPANIC: {}", info).ok();
    kleinos::x86_64::halt();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    use core::fmt::Write;

    // Brute-force access to serial to print panic message
    let mut port = kleinos::serial::SerialPort::new();
    write!(port, "[failed]\n").ok();
    write!(port, "Error: {}\n", info).ok();
    qemu_exit(QemuExitCode::Failure);
}

bootloader::entry_point!(kernel_main);

pub fn kernel_main(_boot_info: &'static bootloader::BootInfo) -> ! {
    #[cfg(test)]
    test_main();

    println!("Hello, Rustaceans!");

    busy_spin(100_000_000);
    qemu_exit(QemuExitCode::Success);
}

#[cfg(test)]
mod tests {
    use kleinos::{
        qemu::{QemuExitCode, qemu_exit},
        serial_print, serial_println,
    };

    pub trait Testable {
        fn run(&self) -> ();
    }

    impl<T> Testable for T
    where
        T: Fn(),
    {
        fn run(&self) {
            serial_print!("{}...\t", core::any::type_name::<T>());
            self();
            serial_println!("[ok]");
        }
    }

    pub fn test_runner(tests: &[&dyn Testable]) {
        serial_println!("Running {} tests", tests.len());
        for test in tests {
            test.run();
        }
        qemu_exit(QemuExitCode::Success);
    }

    #[test_case]
    fn trivial_assertion() {
        assert_eq!(1, 1);
    }
}
