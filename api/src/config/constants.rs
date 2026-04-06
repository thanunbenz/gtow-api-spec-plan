pub const BB_CHIP_VALUE: u64 = 10_000_000;

pub const POSITIONS_8: [&str; 8] = ["UTG", "UTG+1", "LJ", "HJ", "CO", "BTN", "SB", "BB"];

pub const POSITIONS_9: [&str; 9] = [
    "UTG", "UTG+1", "UTG+2", "LJ", "HJ", "CO", "BTN", "SB", "BB",
];

pub const RANKS: [char; 13] = ['A', 'K', 'Q', 'J', 'T', '9', '8', '7', '6', '5', '4', '3', '2'];

pub const COMBO_PAIR: f64 = 6.0;
pub const COMBO_SUITED: f64 = 4.0;
pub const COMBO_OFFSUIT: f64 = 12.0;

pub fn rank_index(c: char) -> usize {
    match c {
        'A' => 0,
        'K' => 1,
        'Q' => 2,
        'J' => 3,
        'T' => 4,
        '9' => 5,
        '8' => 6,
        '7' => 7,
        '6' => 8,
        '5' => 9,
        '4' => 10,
        '3' => 11,
        '2' => 12,
        _ => panic!("Invalid rank: {c}"),
    }
}

pub fn combo_count(index: usize) -> f64 {
    let row = index / 13;
    let col = index % 13;
    if row == col {
        COMBO_PAIR
    } else if col > row {
        COMBO_SUITED
    } else {
        COMBO_OFFSUIT
    }
}
