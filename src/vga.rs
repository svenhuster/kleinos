//! VGA text mode driver for 80x25 display.
//!
//! Uses terminal-style coordinates where row 0 is the bottom of the screen.
//! New text appears at the bottom and scrolls upward as lines are added.
//! This matches typical terminal behavior (newest content at bottom).

use core::ptr::{NonNull, read_volatile, write_volatile};

#[derive(Debug, Clone, Copy)]
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

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct ColorCode(u8);

impl ColorCode {
    #[must_use]
    pub const fn new(foreground: Color, background: Color) -> Self {
        Self((background as u8) << 4 | foreground as u8)
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct ScreenChar {
    pub character: u8,
    pub color: ColorCode,
}

// Ensure ScreenChar layout matches the VGA buffer
const _: () = assert!(core::mem::align_of::<ScreenChar>() == 1);
const _: () = assert!(core::mem::size_of::<ScreenChar>() == 2);
const _: () = assert!(core::mem::offset_of!(ScreenChar, character) == 0);
const _: () = assert!(core::mem::offset_of!(ScreenChar, color) == 1);

pub const BUFFER_HEIGHT: usize = 25;
pub const BUFFER_WIDTH: usize = 80;

// SAFETY: VgaScreen::new() creates a handle to the VGA buffer at
// 0xb8000, which is identity-mapped by the bootloader. The Mutex
// ensures exclusive access
pub static SCREEN: crate::Mutex<VgaScreen> = crate::Mutex::new(unsafe { VgaScreen::new() });

#[derive(Debug)]
pub struct VgaScreen {
    column: usize,
    color_code: ColorCode,
    buffer: NonNull<[[ScreenChar; BUFFER_WIDTH]; BUFFER_HEIGHT]>,
}

// SAFETY: VgaScreen contains a raw pointer to the VGA buffer at
// 0xb8000, which is identity-mapped by the bootloader. This memory is
// accessible from any CPU context and remains valid for the kernel's
// lifetime.
unsafe impl Send for VgaScreen {}

impl VgaScreen {
    /// Creates a new VgaScreen with direct buffer access.
    ///
    /// # Safety
    ///
    /// The caller must ensure this is only used when the global `SCREEN` mutex
    /// cannot be used (e.g., in panic handlers to avoid deadlock). The VGA buffer
    /// at 0xb8000 must be identity-mapped by the bootloader.
    #[must_use]
    pub const unsafe fn new() -> Self {
        Self {
            column: 0,
            color_code: ColorCode::new(Color::LightGray, Color::Black),
            // SAFETY: 0xb8000 is a non-null fixed address for the VGA buffer.
            buffer: unsafe { NonNull::new_unchecked(0xb8000 as *mut _) },
        }
    }

    pub fn clear_line(&mut self) {
        for col in self.column..BUFFER_WIDTH {
            self.write(b' ', self.color_code, 0, col);
        }
    }

    pub fn new_line(&mut self) {
        // Move every line up one, top line is lost
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                // SAFETY: After initialization VgaScreen buffer points to
                // the correct memory address for the VGA buffer. The loops
                // ensure we are within the bounds of is memory region.
                unsafe {
                    write_volatile(
                        &mut (*self.buffer.as_ptr())[row - 1][col],
                        read_volatile(&(*self.buffer.as_ptr())[row][col]),
                    );
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
            self.write(byte, self.color_code, 0, self.column);
            self.column += 1;
        }
    }

    pub fn write(&mut self, byte: u8, color: ColorCode, row: usize, col: usize) {
        if row >= BUFFER_HEIGHT || col >= BUFFER_WIDTH {
            panic!("write access to vga buffer out of bounds");
        }

        // Writing starts from the bottom left of the screen
        let row = BUFFER_HEIGHT - row - 1;

        let ch = ScreenChar {
            character: byte,
            color,
        };

        // SAFETY: After initialization VgaScreen points to the VGA buffer
        // address. To get here the bounds check at the beginning of the fn
        // ensured that we are within the correct memory region.
        unsafe { write_volatile(&mut (*self.buffer.as_ptr())[row][col], ch) };
    }

    #[cfg(test)]
    fn read(&self, row: usize, col: usize) -> ScreenChar {
        if row >= BUFFER_HEIGHT || col >= BUFFER_WIDTH {
            panic!("read access to vga buffer out of bounds");
        }

        // Writing starts from the bottom left of the screen
        let row = BUFFER_HEIGHT - row - 1;

        // SAFETY: After initialization VgaScreen points to the VGA buffer
        // address. To get here the bounds check at the beginning of the fn
        // ensured that we are within the correct memory region.
        unsafe { read_volatile(&(*self.buffer.as_ptr())[row][col]) }
    }
}

impl core::fmt::Write for VgaScreen {
    // Only ASCII will be printed properly on the VGA screen
    fn write_str(&mut self, s: &str) -> Result<(), core::fmt::Error> {
        for ch in s.chars() {
            if ch.is_ascii() {
                self.write_byte(ch as u8);
            } else {
                self.write_byte(0xFE); // write the block char
            }
        }
        Ok(())
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: core::fmt::Arguments) {
    use core::fmt::Write;
    SCREEN.lock().write_fmt(args).expect("VGA write failed");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_case]
    fn test_println_simple() {
        println!("test_println_simple output");
    }

    #[test_case]
    fn test_println_many() {
        for _ in 0..200 {
            println!("test_println_many output");
        }
    }

    #[test_case]
    fn test_print_long() {
        for _ in 0..200 {
            print!("test_println_many output");
        }
    }

    #[test_case]
    fn test_println_output() {
        // TODO: check for non-ascii chars
        let s = "Some test string that fits on a single line";
        println!("{}", s);

        let screen = SCREEN.lock();
        for (i, c) in s.chars().enumerate() {
            let screen_char = screen.read(1, i);
            assert_eq!(char::from(screen_char.character) as u8, c as u8);
        }
    }
}
