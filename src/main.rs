#![feature(slice_patterns)]

extern crate image;
extern crate scrap;
extern crate rand;
extern crate num;
extern crate itertools;

#[macro_use]
extern crate lazy_static;

extern crate dxgcap;

mod mouse;
mod sigmar;
mod ocr;

use sigmar::{Board, Move};

fn main() {
    let mut board = match ocr::ocr_game_board() {
        None => panic!("Couldn't ocr. Board not visible?"),
        Some(board) => board
    };

    let mut moves = board.legal_moves();

    while let Some(&mov) = moves.first() {
        let Move{a, b} = mov;
        println!("{:?}", mov);
        let (x1, y1) = board.pos_to_screen(a.x, a.y);
        let (x2, y2) = board.pos_to_screen(b.x, b.y);

        mouse::click_at(x1, y1);
        mouse::click_at(x2, y2);
        std::thread::sleep(std::time::Duration::from_millis(300));

        board.make_move(mov);

        moves = board.legal_moves();
    }
}