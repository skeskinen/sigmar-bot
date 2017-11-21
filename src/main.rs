#![feature(slice_patterns)]

extern crate image;
extern crate scrap;
extern crate rand;
extern crate num;

#[macro_use]
extern crate lazy_static;

extern crate dxgcap;

mod mouse;
mod sigmar;
mod ocr;

fn main() {
    // send_input();

    let board = match ocr::ocr_game_board() {
        None => panic!("Couldn't ocr. Board not visible?"),
        Some(board) => board
    };

}