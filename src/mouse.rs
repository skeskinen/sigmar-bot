#[link(name="User32")]
extern "system" {
    pub fn SendInput(cInputs: u32, pInputs: *mut Input, cbSize: i32) -> u32;
}

use std;
use std::time::Duration;

#[repr(C)]
pub struct Input {
    pub input_type: u32,
    pub input_data: MouseInput,
}

#[repr(C)]
pub struct MouseInput {
    pub dx: i32,
    pub dy: i32,
    pub mouse_data: u32,
    pub flags: u32,
    pub time: u32,
    pub extra_info: u64,
}

const COORD_MAX: f32 = 65_535.0;
const LEFTDOWN: u32 = 0x0002;
const LEFTUP: u32 = 0x0004;

fn send_input(input: MouseInput) {
    let mut input = Input{
        input_type: 0,
        input_data: input
    };
    unsafe { SendInput(1, &mut input, std::mem::size_of::<Input>() as i32); }
}

fn move_cursor(x: f32, y: f32) {
    send_input(MouseInput {
        dx: (COORD_MAX * x) as i32,
        dy: (COORD_MAX * y) as i32,
        mouse_data: 0x0,
        flags: 0x8001,
        time: 0,
        extra_info: 0,
    });
}

fn click(flags: u32) {
    send_input(MouseInput {
        dx: 0,
        dy: 0,
        mouse_data: 0x0,
        flags: flags,
        time: 0,
        extra_info: 0,
    });
}

pub fn click_at(x: f32, y: f32) {
    move_cursor(x, y);
    std::thread::sleep( Duration::from_millis(10));
    click(LEFTDOWN);
    std::thread::sleep( Duration::from_millis(80));
    click(LEFTUP);
    std::thread::sleep( Duration::from_millis(80));
}