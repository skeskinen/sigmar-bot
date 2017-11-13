#![feature(slice_patterns)]

extern crate image;
extern crate scrap;
extern crate rand;

extern crate dxgcap;

mod mouse;
mod sigmar;
mod ocr;

fn main() {
    // send_input();

    match ocr::ocr_game_board() {
        None => println!("Couldn't ocr. Board not visible?"),
        _ => (),
    }

}