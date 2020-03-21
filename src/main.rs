#![allow(unused_mut)]

use std::ptr;

use winapi::ctypes::{wchar_t};
use winapi::shared::ntdef::NULL;
use winapi::um::wincon::{CreateConsoleScreenBuffer,
                         SetConsoleScreenBufferSize,
                         SetConsoleActiveScreenBuffer,
                         WriteConsoleOutputCharacterW,
                         CONSOLE_TEXTMODE_BUFFER,
                         COORD};
use winapi::um::winnt::{GENERIC_READ, GENERIC_WRITE, HANDLE, SHORT};

fn main() {
    const SCREEN_WIDTH: usize = 120;
    const SCREEN_HEIGHT: usize = 40;

    let mut screen = ['B' as wchar_t; SCREEN_WIDTH*SCREEN_HEIGHT];
    unsafe {
        let console_buffer: HANDLE = CreateConsoleScreenBuffer(
         GENERIC_READ | GENERIC_WRITE,
         0,
         ptr::null(),
         CONSOLE_TEXTMODE_BUFFER,
         NULL
        );
        let mut dw_bytes_writen = 0;

        SetConsoleScreenBufferSize(console_buffer, COORD { X: SCREEN_WIDTH as SHORT, Y: SCREEN_HEIGHT as SHORT });
        SetConsoleActiveScreenBuffer(console_buffer);

        loop {

            WriteConsoleOutputCharacterW(console_buffer,
                                         &screen[0],
                                         (SCREEN_WIDTH * SCREEN_HEIGHT) as u32,
                                         COORD { X: 0, Y: 0 },
                                         &mut dw_bytes_writen
            );
        }
    }
}