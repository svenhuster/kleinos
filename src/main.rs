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

    // SAFETY: Creating a new VGA screen with direct access to the VGA
    // buffer. This is intentional to avoid deadlocking while waiting for the
    // lock that might be held by the fn that just panicked. The buffer is
    // identity-mapped to 0xb8000 by the bootloader, available in all CPU
    // contexts and during the kernel's lifetime. Corrupting the display is
    // acceptable during panic.
    let mut screen = unsafe { kleinos::vga::VgaScreen::new() };
    write!(screen, "\nPANIC: {}", info).ok();
    kleinos::x86_64::halt();
}

bootloader::entry_point!(kernel_main);

pub fn kernel_main(_boot_info: &'static bootloader::BootInfo) -> ! {
    println!("Hello, Rustaceans!");

    busy_spin(100_000_000);
    qemu_exit(QemuExitCode::Success);
}
