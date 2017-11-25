use std;
use std::cmp::{min, max};
use std::fmt;
use itertools::Itertools;
use rand::{thread_rng, Rng};
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Marble {
    Lead = 0,
    Tin = 1,
    Iron = 2,
    Copper = 3,
    Silver = 4,
    Gold = 5,
    Mercury = 6,
    Air = 7,
    Fire = 8,
    Water = 9,
    Earth = 10,
    Vitae = 11,
    Mors = 12,
    Salt = 13,
    Empty = 1000
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
    middle_x: f32,
    middle_y: f32,
    tile_w: f32,
    tile_h: f32,
    hash: u64,
}

lazy_static! {
    static ref ZOBRIST_TABLE: Vec<Vec<Vec<u64>>> = {
        let mut rng = thread_rng();
        let mut ret = vec![vec![vec![0; 14]; 13]; 13];
        for i1 in &mut ret {
            for i2 in i1 {
                for i3 in i2 {
                    *i3 = rng.next_u64();
                }
            }
        }
        ret
    };
}

impl Board {
    pub fn new(board: [[Marble; 13]; 13], middle_x: f32, middle_y: f32, tile_w: f32, tile_h: f32) -> Board {
        let hash: u64 = 0;
        let mut ret = Board {
            board, middle_x, middle_y, tile_w, tile_h, hash
        };

        for y in 1..12 {
            for x in 1..12 {
                ret.hash_add_rem_marble(x, y, board[y][x]);
            }
        }
        ret
    }

    fn hash_add_rem_marble(&mut self, x: usize, y: usize, m: Marble) {
        if m != Marble::Empty {
            self.hash = self.hash ^ ZOBRIST_TABLE[y][x][m as usize];
        }
    }

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


    pub fn solve(&self) -> Option<Vec<Move>> {
        let mut board = self.clone();
        let mut visited: HashSet<u64> = HashSet::with_capacity(60000);
        go_solve(&mut board, &mut visited, 1)
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
        self.hash_add_rem_marble(pos.x, pos.y, pos.marble);
    }

    pub fn add_marble(&mut self, pos: MarblePos) {
        self.board[pos.y][pos.x] = pos.marble;
        self.hash_add_rem_marble(pos.x, pos.y, pos.marble);
    }

    pub fn make_move(&mut self, mov: Move) {
        let Move{a, b} = mov;
        self.remove_marble(a);
        if mov.b.marble != Marble::Gold {
            self.remove_marble(b);
        }
    }

    pub fn reverse_move(&mut self, mov: Move) {
        let Move{a, b} = mov;
        self.add_marble(a);
        if mov.b.marble != Marble::Gold {
            self.add_marble(b);
        }
    }
}

const TOTAL_MOVES: usize = 28;

fn go_solve(board: &mut Board, visited: &mut HashSet<u64>, depth: usize) -> Option<Vec<Move>> {

    if visited.contains(&board.hash) { return None }
    let legal = board.legal_moves();

    for m in legal.into_iter() {
        board.make_move(m);

        if depth == TOTAL_MOVES {
            let v = vec![m; TOTAL_MOVES];
            return Some(v);
        }

        if let Some(mut vec) = go_solve(board, visited, depth + 1) {
            vec[depth - 1] = m;
            return Some(vec)
        }
        visited.insert(board.hash);

        board.reverse_move(m);
    }
    return None;
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