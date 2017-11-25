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

use sigmar::{Move, Marble};

fn main() {

    for i in 0..100 {
        let board = match ocr::ocr_game_board() {
            None => panic!("Couldn't ocr. Board not visible?"),
            Some(board) => board
        };
        let (new_x, new_y) = board.new_game_pos();
        mouse::move_cursor(new_x, new_y);

        if let Some(moves) = board.solve() {
            println!("Counting to 100: {}", i);
            for mov in moves {
                let Move{a, b} = mov;

                let (x1, y1) = board.pos_to_screen(a.x, a.y);
                let (x2, y2) = board.pos_to_screen(b.x, b.y);

                mouse::click_at(x1, y1);
                if b.marble != Marble::Gold {
                    mouse::click_at(x2, y2);
                }
            }
        }
        else {
            println!("Failed to solve. We live in terrible times, good friend.");
        }

        mouse::click_at(new_x, new_y);
        std::thread::sleep(std::time::Duration::from_millis(4500));
    }
}