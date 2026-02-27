#![no_main]
#![no_std]
#![warn(clippy::missing_safety_doc)]
#![warn(clippy::undocumented_unsafe_blocks)]
#![warn(unsafe_op_in_unsafe_fn)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![feature(abi_x86_interrupt)]

pub mod gdt;
pub mod interrupts;
pub mod qemu;
pub mod serial;
pub mod vga;

pub fn init() {
    gdt::init();
    interrupts::init();

    x86_64::instructions::interrupts::enable();
}

pub fn busy_spin(iterations: usize) {
    for _ in 0..iterations {
        core::hint::spin_loop();
    }
}

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

pub trait Testable {
    fn run(&self);
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
    qemu::qemu_exit(qemu::QemuExitCode::Success);
}

pub fn test_panic_handler(info: &core::panic::PanicInfo) -> ! {
    use crate::qemu::{QemuExitCode, qemu_exit};
    use core::fmt::Write;

    let mut port = crate::serial::SERIAL1.lock();
    writeln!(port, "[failed]").ok();
    writeln!(port, "Error: {}", info).ok();
    qemu_exit(QemuExitCode::Failure);
}

#[cfg(test)]
bootloader::entry_point!(lib_test_kernel_main);

#[cfg(test)]
fn lib_test_kernel_main(_boot_info: &'static bootloader::BootInfo) -> ! {
    use crate::qemu::qemu_exit;

    serial::SERIAL1.lock().init();
    init();
    test_main();
    qemu_exit(crate::qemu::QemuExitCode::Success);
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    test_panic_handler(info)
}

#[cfg(test)]
mod tests {
    #[test_case]
    fn test_breakpoint_exceptions() {
        x86_64::instructions::interrupts::int3();
    }
}
