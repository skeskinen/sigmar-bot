use std::cmp::{min, max};

#[derive(Debug, Clone, Copy)]
pub enum Marble {
    Salt,
    Air,
    Fire,
    Water,
    Earth,
    Lead,
    Tin,
    Iron,
    Copper,
    Silver,
    Gold,
    QuickS,
    Vitae,
    Mors,
    Empty
}

pub type Board = [[Marble; 13]; 13];

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