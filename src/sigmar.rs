use std;
use std::cmp::{min, max};
use std::fmt;
use itertools::Itertools;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Marble {
    Lead,
    Tin,
    Iron,
    Copper,
    Silver,
    Gold,
    Mercury,
    Air,
    Fire,
    Water,
    Earth,
    Vitae,
    Mors,
    Salt,
    Empty
}

pub const MARBLE_VALUES: [Marble; 14] = [Marble::Salt, Marble::Air, Marble::Fire, Marble::Water, Marble::Earth,
    Marble::Lead, Marble::Tin, Marble::Iron, Marble::Copper, Marble::Silver, Marble::Gold, Marble::Mercury,
    Marble::Vitae, Marble::Mors];

impl fmt::Display for Marble {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone)]
pub struct Board {
    pub board: [[Marble; 13]; 13],
    pub middle_x: f32,
    pub middle_y: f32,
    pub tile_w: f32,
    pub tile_h: f32,
}

impl Board {
    fn is_free(&self, c_x: usize, c_y: usize) -> bool {
        let (s_x, s_y) = (c_x - 1, c_y - 1);
        if self.board[c_y][c_x] == Marble::Empty { return false }
        let is_neighbour_empty = |m_x: usize, m_y: usize| { if self.board[s_y + m_y][s_x + m_x] == Marble::Empty { true } else { false }};
        let mut empty_neighbour_sum = 0;

        let neighbours = [(2, 1), (2, 0), (1, 0), (0, 1), (0, 2), (1, 2), (2, 1), (2, 0)];

        for &(m_x, m_y) in &neighbours {
            if is_neighbour_empty(m_x, m_y) {
                empty_neighbour_sum += 1;
                if empty_neighbour_sum >= 3 { return true }
            }
            else {
                empty_neighbour_sum = 0;
            }
        }
        empty_neighbour_sum >= 3
    }

    fn free_marbles(&self) -> Vec<MarblePos> {
        let mut v = Vec::with_capacity(10);
        for y in 1..12 {
            for x in 1..12 {
                if self.is_free(x, y) {
                    v.push(MarblePos {
                        marble: self.board[y][x],
                        x, y
                    })
                }
            }
        }
        v
    }

    pub fn least_metal(&self) -> Marble {
        let mut m_least = Marble::Empty;
        let mut m_least_val = 0;
        for y in 1..12 {
            for x in 1..12 {
                let marble_xy = self.board[y][x];
                let val = match marble_xy {
                    Marble::Lead => 6,
                    Marble::Tin => 5,
                    Marble::Iron => 4,
                    Marble::Copper => 3,
                    Marble::Silver => 2,
                    Marble::Gold => 1,
                    _ => 0,
                };
                if val > m_least_val {
                    m_least_val = val;
                    m_least = marble_xy;
                }
            }
        }
        m_least
    }

    pub fn legal_moves(&self) -> Vec<Move> {
        let mut moves: Vec<Move> = Vec::with_capacity(40);
        let mut free = self.free_marbles();
        free.sort_unstable();

        let groups = free.into_iter().group_by(|mov: &MarblePos| mov.marble);

        let mut elementals: Vec<MarblePos> = Vec::with_capacity(16);
        let mut vitae: Vec<MarblePos> = Vec::with_capacity(4);
        let mut metals: Vec<MarblePos> = Vec::with_capacity(4);

        for (key, it) in &groups {
            let group: Vec<MarblePos> = it.collect();
            if [Marble::Lead, Marble::Tin, Marble::Iron, Marble::Copper, Marble::Silver, Marble::Gold].iter().any(|marble| (*marble) == key) {
                if self.least_metal() == key {
                    if key == Marble::Gold {
                        let &g = group.first().unwrap();
                        moves.push(Move{a: g, b: g});
                    }
                    else {
                        metals.extend(&group);
                    }
                }
            }
            else if key == Marble::Mercury {
                if let Some(&metal) = metals.first() {
                    for mer in group {
                        moves.push(Move{a: metal, b: mer});
                    }
                }
            }
            else if [Marble::Air, Marble::Fire, Marble::Earth, Marble::Water].iter().any(|marble| (*marble) == key) {
                elementals.extend(&group);
                for i in 0..group.len()-1 {
                    for j in i+1..group.len() {
                        moves.push(Move{a: group[i], b: group[j]});
                    }
                }
            }
            else if key == Marble::Vitae {
                vitae.extend(&group);
            }
            else if key == Marble::Mors {
                for mors in group {
                    for &vit in &vitae {
                        moves.push(Move{a: mors, b: vit});
                    }
                }
            }
            else if key == Marble::Salt {
                for i in 0..group.len()-1 {
                    for j in i+1..group.len() {
                        moves.push(Move{a: group[i], b: group[j]});
                    }
                }
                for &salt in &group {
                    for &el in &elementals {
                        moves.push(Move{a: el, b: salt});
                    }
                }
            }
        }
        moves
    }


    pub fn solve(&self) -> Vec<Move> {
        let mut board = self.clone();
        let mut moves: Vec<Move> = Vec::with_capacity(30);
        go_solve(&mut board, &mut moves);
        moves
    }

    pub fn pos_to_screen(&self, x: usize, y: usize) -> (f32, f32) {
        let offset_x = x as f32 - 6.0 + (y as f32 - 6.0) / 2.0;
        let offset_y = y as f32 - 6.0;
        (self.middle_x + offset_x * self.tile_w, self.middle_y -  offset_y * self.tile_h)
    }

    pub fn new_game_pos(&self) -> (f32, f32) {
        let offset_x = self.tile_w as f32 * -5.0;
        let offset_y = self.tile_h as f32 * 6.5;
        (self.middle_x + offset_x, self.middle_y + offset_y)
    }
    
    pub fn remove_marble(&mut self, pos: MarblePos) {
        self.board[pos.y][pos.x] = Marble::Empty;
    }

    pub fn make_move(&mut self, mov: Move) {
        let Move{a, b} = mov;
        self.remove_marble(a);
        self.remove_marble(b);
    }

    pub fn reverse_move(&mut self, mov: Move) {
        let Move{a, b} = mov;
        self.board[a.y][a.x] = a.marble;
        self.board[b.y][b.x] = b.marble;
    }
}

const TOTAL_MOVES: usize = 28;

fn go_solve(board: &mut Board, moves_ref: &mut Vec<Move>) -> bool {
    let legal = board.legal_moves();

    for m in legal.into_iter() {
        board.make_move(m);
        moves_ref.push(m);

        if moves_ref.len() == TOTAL_MOVES {
            return true;
        }
        if go_solve(board, moves_ref) {
            return true
        }

        moves_ref.pop();
        board.reverse_move(m);
    }
    return false;
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Move {
    pub a: MarblePos,
    pub b: MarblePos,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct MarblePos {
    pub x: usize,
    pub y: usize,
    pub marble: Marble,
}

impl PartialOrd for MarblePos {
    fn partial_cmp(&self, other: &MarblePos) -> Option <std::cmp::Ordering> {
        Some(self.cmp(&other))
    }
}
impl Ord for MarblePos {
    fn cmp(&self, other: &MarblePos) -> std::cmp::Ordering {
        self.marble.cmp(&other.marble)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RowDesc {
    pub x_min: i32,
    pub x_max: i32,
}

pub fn board_rows() -> Vec<RowDesc> {
    let a = (0..11).map(|r| RowDesc{x_min: max(-r + 5, 0), x_max: min(15-r, 10)});
    return a.collect();
}

#[cfg(test)]
mod tests {
    use ::sigmar::*;

    #[test]
    fn test_board_rows() {
        let rows = board_rows();
        assert_eq!(RowDesc{x_min: 5, x_max: 10},  rows[0]);
        assert_eq!(RowDesc{x_min: 0, x_max: 10},  rows[5]);
        assert_eq!(RowDesc{x_min: 0, x_max: 5},  *rows.last().unwrap());
    }
}