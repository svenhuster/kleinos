#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failure = 0x11,
}

pub fn qemu_exit(exit_code: QemuExitCode) -> ! {
    // SAFETY: 0xF4 is the port for QEMU exit.
    // 'hlt' is safe to execute in ring 0.
    unsafe {
        core::arch::asm!(
            "out dx, eax",
            "cli",
            "2: hlt",
            "jmp 2b",
            in("dx") 0xf4u16,
            in("eax") exit_code as u32,
            options(nostack, noreturn),
        );
    }
}
