//! VGA text mode driver for 80x25 display.
//!
//! Uses terminal-style coordinates where row 0 is the bottom of the screen.
//! New text appears at the bottom and scrolls upward as lines are added.
//! This matches typical terminal behavior (newest content at bottom).

use core::ptr::write_volatile;
use lazy_static::lazy_static;
use spin::Mutex;

lazy_static! {
    pub static ref SCREEN: Mutex<VgaScreen> = {
        let default_color = ColorCode::new(Color::LightGray, Color::Black);
        Mutex::new(VgaScreen{
            column: 0,
            color_code: default_color,
            // SAFETY: 0xb8000 is identity-mapped by the bootloader and points to
            // the VGA buffer. We are running in ring0 and have access to the
            // buffer.
            buffer: unsafe { &mut *(0xb8000 as *mut _)},
            shadow: [[ScreenChar{character: b' ', color: default_color}; BUFFER_WIDTH]; BUFFER_HEIGHT],
        })
    };
}

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
const _: () = {
    assert!(core::mem::align_of::<ScreenChar>() == 1);
    assert!(core::mem::size_of::<ScreenChar>() == 2);
    assert!(core::mem::offset_of!(ScreenChar, character) == 0);
    assert!(core::mem::offset_of!(ScreenChar, color) == 1);
};

pub const BUFFER_HEIGHT: usize = 25;
pub const BUFFER_WIDTH: usize = 80;

#[derive(Debug)]
pub struct VgaScreen {
    column: usize,
    color_code: ColorCode,
    buffer: &'static mut [[ScreenChar; BUFFER_WIDTH]; BUFFER_HEIGHT],
    shadow: [[ScreenChar; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

impl VgaScreen {
    pub fn flush(&mut self) {
        // SAFETY: After initialization VgaScreen buffer points to the correct
        // memory address for the VGA buffer (identify-mapped by the bootloader)
        // and we have access in ring0. The loop bounds ensure we are within the
        // bounds of is memory region. Access to the buffer is managed via a
        // Mutex. The shadow buffer is the same size and type as the buffer.
        unsafe {
            write_volatile(self.buffer, self.shadow);
        }
    }

    pub fn clear_line(&mut self) {
        for col in self.column..BUFFER_WIDTH {
            self.write(b' ', self.color_code, 0, col);
        }
    }

    pub fn new_line(&mut self) {
        // Move every line up one, top line is lost
        self.shadow.copy_within(1.., 0);
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

        self.shadow[row][col] = ch;
    }

    #[cfg(test)]
    fn read(&self, row: usize, col: usize) -> ScreenChar {
        if row >= BUFFER_HEIGHT || col >= BUFFER_WIDTH {
            panic!("read access to vga buffer out of bounds");
        }

        // Reading starts from the bottom left of the screen to match writing
        let row = BUFFER_HEIGHT - row - 1;

        self.shadow[row][col]
    }
}

impl core::fmt::Write for VgaScreen {
    // Only ASCII will be printed properly on the VGA screen
    fn write_str(&mut self, s: &str) -> Result<(), core::fmt::Error> {
        for ch in s.chars() {
            if ch.is_ascii() {
                self.write_byte(ch as u8);
            } else {
                self.write_byte(0xFE); // write the block char if not ASCII
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
    use x86_64::instructions::interrupts;
    interrupts::without_interrupts(|| {
        let mut vga = SCREEN.lock();
        vga.write_fmt(args).expect("VGA write failed");
        vga.flush();
    });
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
