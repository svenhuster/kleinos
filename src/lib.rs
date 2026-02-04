#![no_std]

pub fn busy_spin(iterations: usize) {
    for _ in 0..iterations {
        core::hint::spin_loop();
    }
}

pub mod x86_64 {
    pub fn halt() -> ! {
        // SAFETY: cli/hlt are safe to execute in ring 0. As we run
        // single-threaded in ring0 no other process will 'sti'
        unsafe {
            core::arch::asm!(
                "cli",
                "2: hlt",
                "jmp 2b",
                options(nomem, nostack, preserves_flags, noreturn)
            );
        }
    }

    pub fn reset() -> ! {
        // SAFETY: 0x64 is the keyboard controller command port.
        // Command 0xFE pulses the CPU reset line.
        unsafe {
            outb(0x64, 0xFE);
        }
        halt();
    }

    /// # Safety
    ///
    /// Caller must ensure the port access is valid and won't cause undefined
    /// behavior (e.g., triggering DMA to invalid memory, corrupting hardware
    /// state).
    pub unsafe fn outb(port: u16, value: u8) {
        // SAFETY: Caller needs to ensure guarantees are met
        unsafe {
            core::arch::asm!("out dx, al", in("dx") port, in("al") value, options(nomem, nostack, preserves_flags));
        }
    }

    /// # Safety
    ///
    /// Caller must ensure the port access is valid and won't cause undefined
    /// behavior (e.g., triggering DMA to invalid memory, corrupting hardware
    /// state).
    pub unsafe fn inb(port: u16) -> u8 {
        let value: u8;
        // SAFETY: Caller needs to ensure guarantees are met
        unsafe {
            core::arch::asm!("in al, dx", in("dx") port, out("al") value, options(nomem, nostack, preserves_flags));
        }
        value
    }
}

pub mod qemu {
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
                "2: hlt",
                "jmp 2b",
                in("dx") 0xf4u16,
                in("eax") exit_code as u32,
                options(nomem, nostack, preserves_flags, noreturn),
            );
        }
    }
}

pub mod vga {
    use core::ptr::{read_volatile, write_volatile};

    const BUFFER_HEIGHT: usize = 25;
    const BUFFER_WIDTH: usize = 80;
    const BUFFER: *mut [[ScreenChar; BUFFER_WIDTH]; BUFFER_HEIGHT] = 0xb8000 as *mut _;

    #[derive(Clone, Copy)]
    #[repr(u8)]
    pub enum Color {
        Black = 0,
        Blue = 1,
        Green = 2,
        Cyan = 3,
        Red = 4,
        Magenta = 5,
        Brown = 6,
        LightGray = 7,
        DarkGray = 8,
        LightBlue = 9,
        LightGreen = 10,
        LightCyan = 11,
        LightRed = 12,
        Pink = 13,
        Yellow = 14,
        White = 15,
    }

    #[derive(Clone, Copy)]
    #[repr(transparent)]
    struct CharColor(u8);

    impl CharColor {
        const fn new(foreground: Color, background: Color) -> Self {
            Self((background as u8) << 4 | foreground as u8)
        }
    }

    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct ScreenChar {
        character: u8,
        color: CharColor,
    }

    const _: () = assert!(core::mem::size_of::<ScreenChar>() == 2);

    impl ScreenChar {
        pub const fn new(character: u8, fg: Color, bg: Color) -> Self {
            let color = CharColor::new(fg, bg);
            Self { character, color }
        }
    }

    pub struct VgaScreen {
        column: usize,
        buffer: *mut [[ScreenChar; BUFFER_WIDTH]; BUFFER_HEIGHT],
    }

    impl VgaScreen {
        pub const fn new() -> Self {
            Self {
                column: 0,
                buffer: BUFFER,
            }
        }
    }

    impl Default for VgaScreen {
        fn default() -> Self {
            Self::new()
        }
    }

    impl VgaScreen {
        pub fn write_str(&mut self, s: &[u8]) {
            for byte in s {
                self.write_byte(*byte);
            }
        }

        pub fn clear_line(&mut self) {
            for _ in self.column..BUFFER_WIDTH {
                self.write_byte(b' ');
            }
            self.column = 0;
        }

        pub fn new_line(&mut self) {
            // Move every line up one, top line is lost
            for row in 1..BUFFER_HEIGHT {
                for col in 0..BUFFER_WIDTH {
                    unsafe {
                        let prev = read_volatile(&(*self.buffer)[row][col]);
                        write_volatile(&mut (*self.buffer)[row-1][col], prev);
                    }
                }
            }
            self.column = 0;
            self.clear_line();
        }

        pub fn write_byte(&mut self, byte: u8) {
            if self.column >= BUFFER_WIDTH {
                self.new_line();
            }
            if byte == b'\n' {
                self.new_line();
            } else {
                let ch = ScreenChar::new(byte, Color::LightGray, Color::Black);
                self.write(ch, 0, self.column);
                self.column += 1;
            }
        }

        pub fn write(&mut self, ch: ScreenChar, row: usize, col: usize) {
            // Writing starts from the bottom left of the screen
            let row = BUFFER_HEIGHT-row-1;
            unsafe {write_volatile(&mut (*self.buffer)[row][col], ch)};
        }
    }
}
