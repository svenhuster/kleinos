#![no_std]
#![no_main]

use core::panic::PanicInfo;
use kleinos::qemu::{QemuExitCode, qemu_exit};
use kleinos::{busy_spin, vga};
use kleinos::vga::{Color, ScreenChar};
use kleinos::x86_64;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    let mut screen = vga::VgaScreen::new();
    screen.write(ScreenChar::new(b'*', Color::Red, Color::Black), 0, 0);

    x86_64::halt();
}

bootloader::entry_point!(kernel_main);

pub fn kernel_main(_boot_info: &'static bootloader::BootInfo) -> ! {
    let mut screen = vga::VgaScreen::new();
    for (col, ch) in b"Hello, Rustaceans!".iter().enumerate() {
        screen.write(ScreenChar::new(*ch, Color::LightGray, Color::Black), 0, col);
    }

    busy_spin(100_000_000);

    qemu_exit(QemuExitCode::Success);
}
