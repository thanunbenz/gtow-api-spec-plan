use crate::config::constants::{POSITIONS_8, POSITIONS_9};
use crate::types::api::{NextGame, NextGamePlayer, SpotGame, SpotGamePlayer, StreetInfo};
use crate::types::s3::{S3Node, S3Settings};

pub enum PlayerOrder {
    /// Spot-solution: starts from active_position, wraps clockwise
    ActiveFirst,
    /// Next-actions: natural table order (UTG, UTG+1, ..., BB)
    TableOrder,
}

/// Map S3 player index (0-7) to 9-seat API index (0-8), inserting UTG+2 at index 2
pub fn s3_to_api_seat(s3_idx: usize) -> usize {
    if s3_idx < 2 { s3_idx } else { s3_idx + 1 }
}

/// Map API seat (0-8) to S3 player index, returns None for UTG+2 (virtual)
fn api_seat_to_s3(api_seat: usize) -> Option<usize> {
    match api_seat {
        0 | 1 => Some(api_seat),
        2 => None, // UTG+2 virtual
        s => Some(s - 1),
    }
}

struct PlayerState {
    folded: Vec<bool>,
    chips_on_table: Vec<u64>,
    ante_total: u64,
    num_s3_players: usize,
}

fn compute_player_state(node: &S3Node, settings: &S3Settings) -> PlayerState {
    let num = settings.handdata.stacks.len();
    let ante = if settings.handdata.blinds.len() > 2 { settings.handdata.blinds[2] } else { 0 };
    let mut folded = vec![false; num];
    let mut chips = vec![0u64; num];

    let sb_idx = num - 2;
    let bb_idx = num - 1;
    chips[sb_idx] = settings.handdata.blinds[1];
    chips[bb_idx] = settings.handdata.blinds[0];

    for seq in &node.sequence {
        let p = seq.player;
        if p < num {
            match seq.action_type.as_str() {
                "F" => folded[p] = true,
                "R" | "C" => chips[p] = seq.amount,
                _ => {}
            }
        }
    }

    PlayerState {
        folded,
        chips_on_table: chips,
        ante_total: ante * num as u64,
        num_s3_players: num,
    }
}

fn compute_game_numbers(state: &PlayerState, settings: &S3Settings) -> (f64, f64, f64) {
    let bb = settings.handdata.blinds[0] as f64;
    let pot_chips: u64 = state.chips_on_table.iter().sum::<u64>() + state.ante_total;
    let pot_bb = pot_chips as f64 / bb;
    let start_pot_chips = state.ante_total + settings.handdata.blinds[0] + settings.handdata.blinds[1];
    let start_pot_bb = start_pot_chips as f64 / bb;
    (pot_bb, start_pot_bb, bb)
}

fn compute_pot_odds_str(state: &PlayerState, active_s3: usize, pot_chips: u64) -> Option<String> {
    let max_bet = state.chips_on_table.iter().copied().max().unwrap_or(0);
    let hero_bet = state.chips_on_table[active_s3];
    let to_call = max_bet.saturating_sub(hero_bet);
    if pot_chips > 0 && to_call > 0 {
        let odds = to_call as f64 / (pot_chips + to_call) as f64;
        Some(format!("{odds:.3}"))
    } else {
        None
    }
}

fn get_relative_pos(api_seat: usize, is_folded: bool) -> Option<String> {
    if is_folded { return None; }
    let btn_api = 6; // BTN is seat 6 in 9-handed
    if api_seat == btn_api { Some("IP".to_string()) } else { Some("OOP".to_string()) }
}

// ---------- Spot-solution game (field order: relative_postflop_position first) ----------

pub fn build_spot_game(node: &S3Node, settings: &S3Settings) -> SpotGame {
    let state = compute_player_state(node, settings);
    let (pot_bb, start_pot_bb, bb) = compute_game_numbers(&state, settings);
    let pot_chips: u64 = state.chips_on_table.iter().sum::<u64>() + state.ante_total;
    let active_s3 = node.player;
    let active_api = s3_to_api_seat(active_s3);

    let mut players = Vec::with_capacity(9);
    for i in 0..9 {
        let api_seat = (active_api + i) % 9;
        players.push(build_spot_player(api_seat, active_api, &state, settings, bb));
    }

    SpotGame {
        players,
        current_street: StreetInfo {
            street_type: "PREFLOP".to_string(),
            start_pot: format_bb_3dec(start_pot_bb),
            end_pot: format_bb_3dec(pot_bb),
        },
        pot: format_bb_3dec(pot_bb),
        pot_odds: compute_pot_odds_str(&state, active_s3, pot_chips),
        active_position: POSITIONS_9[active_api].to_string(),
        board: String::new(),
        bet_display_name: "RAISE".to_string(),
    }
}

fn build_spot_player(api_seat: usize, active_api: usize, state: &PlayerState, settings: &S3Settings, bb: f64) -> SpotGamePlayer {
    let position = POSITIONS_9[api_seat];

    if let Some(s3_idx) = api_seat_to_s3(api_seat) {
        let stack_bb = settings.handdata.stacks[s3_idx] as f64 / bb;
        let chips_bb = state.chips_on_table[s3_idx] as f64 / bb;
        let current_bb = stack_bb - chips_bb;
        let is_folded = state.folded[s3_idx];
        let btn_api = 6;

        SpotGamePlayer {
            relative_postflop_position: get_relative_pos(api_seat, is_folded),
            hand: None,
            is_dealer: api_seat == btn_api,
            is_folded,
            is_hero: api_seat == active_api,
            is_active: api_seat == active_api,
            stack: format_bb_3dec(stack_bb),
            current_stack: format_bb_3dec(current_bb),
            chips_on_table: format_bb_smart(chips_bb),
            bounty: None,
            profile: None,
            position: position.to_string(),
            bounty_in_bb: None,
        }
    } else {
        // Virtual UTG+2 player
        let ante_bb = if settings.handdata.blinds.len() > 2 {
            settings.handdata.blinds[2] as f64 / bb
        } else { 0.0 };
        // Use average stack for virtual player
        let avg_stack: f64 = settings.handdata.stacks.iter().map(|s| *s as f64 / bb).sum::<f64>() / state.num_s3_players as f64;

        SpotGamePlayer {
            relative_postflop_position: None,
            hand: None,
            is_dealer: false,
            is_folded: true,
            is_hero: false,
            is_active: false,
            stack: format_bb_3dec(avg_stack),
            current_stack: format_bb_3dec(avg_stack - ante_bb),
            chips_on_table: format_bb_smart(0.0),
            bounty: None,
            profile: None,
            position: position.to_string(),
            bounty_in_bb: None,
        }
    }
}

// ---------- Next-actions game (field order: position first) ----------

pub fn build_next_game(node: &S3Node, settings: &S3Settings) -> NextGame {
    let state = compute_player_state(node, settings);
    let (pot_bb, start_pot_bb, bb) = compute_game_numbers(&state, settings);
    let pot_chips: u64 = state.chips_on_table.iter().sum::<u64>() + state.ante_total;
    let active_s3 = node.player;
    let active_api = s3_to_api_seat(active_s3);

    let mut players = Vec::with_capacity(9);
    for api_seat in 0..9 {
        players.push(build_next_player(api_seat, active_api, &state, settings, bb));
    }

    NextGame {
        players,
        current_street: StreetInfo {
            street_type: "PREFLOP".to_string(),
            start_pot: format_bb_3dec(start_pot_bb),
            end_pot: format_bb_3dec(pot_bb),
        },
        pot: format_bb_3dec(pot_bb),
        pot_odds: compute_pot_odds_str(&state, active_s3, pot_chips),
        active_position: POSITIONS_9[active_api].to_string(),
        board: String::new(),
        bet_display_name: "RAISE".to_string(),
    }
}

fn build_next_player(api_seat: usize, active_api: usize, state: &PlayerState, settings: &S3Settings, bb: f64) -> NextGamePlayer {
    let position = POSITIONS_9[api_seat];

    if let Some(s3_idx) = api_seat_to_s3(api_seat) {
        let stack_bb = settings.handdata.stacks[s3_idx] as f64 / bb;
        let chips_bb = state.chips_on_table[s3_idx] as f64 / bb;
        let current_bb = stack_bb - chips_bb;
        let is_folded = state.folded[s3_idx];
        let btn_api = 6;

        NextGamePlayer {
            position: position.to_string(),
            relative_postflop_position: get_relative_pos(api_seat, is_folded),
            hand: None,
            is_dealer: api_seat == btn_api,
            is_folded,
            is_hero: api_seat == active_api,
            is_active: api_seat == active_api,
            stack: format_bb_3dec(stack_bb),
            current_stack: format_bb_3dec(current_bb),
            chips_on_table: format!("{chips_bb:.2}"),
            bounty: None,
            bounty_in_bb: None,
            profile: None,
        }
    } else {
        let ante_bb = if settings.handdata.blinds.len() > 2 {
            settings.handdata.blinds[2] as f64 / bb
        } else { 0.0 };
        let avg_stack: f64 = settings.handdata.stacks.iter().map(|s| *s as f64 / bb).sum::<f64>() / state.num_s3_players as f64;

        NextGamePlayer {
            position: position.to_string(),
            relative_postflop_position: None,
            hand: None,
            is_dealer: false,
            is_folded: true,
            is_hero: false,
            is_active: false,
            stack: format_bb_3dec(avg_stack),
            current_stack: format_bb_3dec(avg_stack - ante_bb),
            chips_on_table: "0.00".to_string(),
            bounty: None,
            bounty_in_bb: None,
            profile: None,
        }
    }
}

// ---------- Shared helpers ----------

pub fn count_remaining_players(node: &S3Node, num_players: usize) -> usize {
    let mut folded = vec![false; num_players];
    for seq in &node.sequence {
        if seq.action_type == "F" && seq.player < num_players {
            folded[seq.player] = true;
        }
    }
    folded.iter().filter(|&&f| !f).count()
}

pub fn compute_pot_bb(node: &S3Node, settings: &S3Settings) -> f64 {
    let bb = settings.handdata.blinds[0] as f64;
    let ante = if settings.handdata.blinds.len() > 2 { settings.handdata.blinds[2] as f64 } else { 0.0 };
    let num = settings.handdata.stacks.len();
    let ante_total = ante as u64 * num as u64;

    let mut chips = vec![0u64; num];
    chips[num - 2] = settings.handdata.blinds[1];
    chips[num - 1] = settings.handdata.blinds[0];

    for seq in &node.sequence {
        let p = seq.player;
        if p < num && (seq.action_type == "R" || seq.action_type == "C") {
            chips[p] = seq.amount;
        }
    }

    (chips.iter().sum::<u64>() + ante_total) as f64 / bb
}

pub fn compute_current_bet_bb(node: &S3Node, settings: &S3Settings) -> f64 {
    let bb = settings.handdata.blinds[0] as f64;
    let num = settings.handdata.stacks.len();
    let mut max_bet: u64 = settings.handdata.blinds[0];

    for seq in &node.sequence {
        if seq.action_type == "R" && seq.player < num {
            max_bet = max_bet.max(seq.amount);
        }
    }

    max_bet as f64 / bb
}

/// "20.125", "2.500" — always 3 decimal places
fn format_bb_3dec(val: f64) -> String {
    format!("{val:.3}")
}

/// Smart format: "0", "2" for integers, "0.500", "1.000" for fractions (3 dec)
fn format_bb_smart(val: f64) -> String {
    let rounded = (val * 1000.0).round() / 1000.0;
    if rounded == rounded.floor() && rounded.abs() < 1e12 {
        format!("{}", rounded as i64)
    } else {
        format!("{rounded:.3}")
    }
}
