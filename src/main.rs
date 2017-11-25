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

use sigmar::{Move};

fn main() {

    for _i in 0..50 {
        let board = match ocr::ocr_game_board() {
            None => panic!("Couldn't ocr. Board not visible?"),
            Some(board) => board
        };

        let moves = board.solve();
        for mov in moves {
            let Move{a, b} = mov;
            println!("{:?}", mov);
            let (x1, y1) = board.pos_to_screen(a.x, a.y);
            let (x2, y2) = board.pos_to_screen(b.x, b.y);

            mouse::click_at(x1, y1);
            mouse::click_at(x2, y2);
            std::thread::sleep(std::time::Duration::from_millis(300));
        }

        let (new_x, new_y) = board.new_game_pos();
        mouse::click_at(new_x, new_y);
        std::thread::sleep(std::time::Duration::from_secs(5));
    }
}