#![no_std]
#![no_main]
#![warn(clippy::missing_safety_doc)]
#![warn(clippy::undocumented_unsafe_blocks)]
#![warn(unsafe_op_in_unsafe_fn)]

use kleinos::{
    busy_spin, println,
    qemu::{QemuExitCode, qemu_exit},
};

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    use core::fmt::Write;

    // Brute-force access to VGA to print panic message
    let mut screen = kleinos::vga::VgaScreen::new();
    write!(screen, "\nPANIC: {}", info).ok();
    kleinos::x86_64::halt();
}

bootloader::entry_point!(kernel_main);

pub fn kernel_main(_boot_info: &'static bootloader::BootInfo) -> ! {
    println!("Hello, Rustaceans!");

    busy_spin(100_000_000);
    qemu_exit(QemuExitCode::Success);
}
