#![no_std]
#![no_main]

use core::panic::PanicInfo;
use kleinos::qemu::{QemuExitCode, qemu_exit};
use kleinos::vga::{CharColor, Color};
use kleinos::{busy_spin, vga};
use kleinos::x86_64;
use core::fmt::Write;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    let red = CharColor::new(Color::Red, Color::Black);
    let mut screen = vga::VgaScreen::new();
    screen.write(b'*', red, 0, 0);

    x86_64::halt();
}

bootloader::entry_point!(kernel_main);

pub fn kernel_main(_boot_info: &'static bootloader::BootInfo) -> ! {
    let mut screen = vga::VgaScreen::new();

    writeln!(screen, "Hello, Rustaceans!").unwrap();
    screen.new_line();

    busy_spin(100_000_000);
    qemu_exit(QemuExitCode::Success);
}
