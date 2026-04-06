use std::collections::HashMap;

use crate::config::constants::{combo_count, POSITIONS_8};
use crate::transform::game_state::s3_to_api_seat;
use crate::transform::hand_index::{hand_name_to_index, index_to_hand_name};
use crate::types::api::{PlayerDef, PlayerInfo};
use crate::types::s3::{S3Node, S3Settings};

pub fn build_players_info(
    node: &S3Node,
    settings: &S3Settings,
    parent_node: Option<&S3Node>,
    parent_action_index: Option<usize>,
) -> Vec<PlayerInfo> {
    let bb = settings.handdata.blinds[0] as f64;
    let mut result = Vec::new();

    // If facing a raise, include the last aggressor's range
    if let (Some(parent), Some(action_idx)) = (parent_node, parent_action_index) {
        let parent_action = &parent.actions[action_idx];
        if parent_action.action_type == "R" {
            let aggressor_range = compute_range_after_action(parent, action_idx);
            let aggressor_evs = compute_evs_for_action(parent, action_idx);
            let aggressor_s3 = parent.player;
            let aggressor_api = s3_to_api_seat(aggressor_s3);
            let position = POSITIONS_8[aggressor_s3];
            let stack_bb = settings.handdata.stacks[aggressor_s3] as f64 / bb;
            let chips_committed = parent_action.amount as f64 / bb;
            let current_stack = stack_bb - chips_committed;
            let btn_s3 = settings.handdata.stacks.len() - 3;

            let rel_pos = if aggressor_s3 == btn_s3 { "IP" } else { "OOP" };

            let total_combos_val = compute_total_combos(&aggressor_range);
            let total_ev_val = compute_total_ev(&aggressor_range, &aggressor_evs);
            let counters = build_simple_hand_counters(&aggressor_range);

            result.push(build_player_info(
                PlayerDef {
                    relative_postflop_position: Some(rel_pos.to_string()),
                    hand: None,
                    is_dealer: aggressor_s3 == btn_s3,
                    is_folded: false,
                    is_hero: false,
                    is_active: false,
                    stack: format!("{stack_bb:.3}"),
                    current_stack: format!("{current_stack:.3}"),
                    chips_on_table: format_bb_smart(chips_committed),
                    bounty: None,
                    profile: None,
                    position: position.to_string(),
                    bounty_in_bb: None,
                    name: position.to_string(),
                    seat: aggressor_api,
                },
                aggressor_range,
                aggressor_evs,
                total_ev_val,
                total_combos_val,
                counters,
                rel_pos.to_string(),
            ));
        }
    }

    // Always include the hero (active player)
    let hero_s3 = node.player;
    let hero_api = s3_to_api_seat(hero_s3);
    let position = POSITIONS_8[hero_s3];
    let stack_bb = settings.handdata.stacks[hero_s3] as f64 / bb;
    let btn_s3 = settings.handdata.stacks.len() - 3;

    let hero_range = build_hero_range(node);
    let hero_evs = compute_hero_evs(node);

    let mut hero_chips = 0u64;
    for seq in &node.sequence {
        if seq.player == hero_s3 && (seq.action_type == "R" || seq.action_type == "C") {
            hero_chips = seq.amount;
        }
    }
    let chips_bb = hero_chips as f64 / bb;
    let current_stack = stack_bb - chips_bb;
    let is_btn = hero_s3 == btn_s3;
    let rel_pos = if is_btn { "IP" } else { "OOP" };

    let total_combos_val = compute_total_combos(&hero_range);
    let total_ev_val = compute_total_ev(&hero_range, &hero_evs);
    let counters = build_simple_hand_counters(&hero_range);

    result.push(build_player_info(
        PlayerDef {
            relative_postflop_position: Some(rel_pos.to_string()),
            hand: None,
            is_dealer: is_btn,
            is_folded: false,
            is_hero: true,
            is_active: true,
            stack: format!("{stack_bb:.3}"),
            current_stack: format!("{current_stack:.3}"),
            chips_on_table: format_bb_smart(chips_bb),
            bounty: None,
            profile: None,
            position: position.to_string(),
            bounty_in_bb: None,
            name: position.to_string(),
            seat: hero_api,
        },
        hero_range,
        hero_evs,
        total_ev_val,
        total_combos_val,
        counters,
        rel_pos.to_string(),
    ));

    result
}

fn build_player_info(
    player: PlayerDef,
    range: Vec<f64>,
    hand_evs: Vec<f64>,
    total_ev: Option<f64>,
    total_combos: f64,
    simple_hand_counters: HashMap<String, f64>,
    relative_postflop_position: String,
) -> PlayerInfo {
    // eq_percentile: placeholder zeros for preflop
    let eq_percentile = vec![0.0; 169];

    PlayerInfo {
        player,
        range,
        hand_evs,
        hand_eqs: vec![0.0; 169],
        hand_eqrs: vec![],
        total_ev,
        total_eq: None,
        total_eqr: None,
        pot_share: 0.0,
        total_combos,
        simple_hand_counters,
        equity_buckets_range: vec![],
        equity_buckets_advanced_range: vec![],
        equity_buckets: vec![0; 4],
        equity_buckets_advanced: vec![0; 7],
        hand_categories: vec![0; 17],
        draw_categories: vec![0; 8],
        relative_postflop_position,
        eq_percentile,
        tournament_evs_converter: None,
    }
}

fn build_hero_range(node: &S3Node) -> Vec<f64> {
    let mut range = vec![0.0; 169];
    for (name, hand) in &node.hands {
        if let Some(idx) = hand_name_to_index(name) {
            range[idx] = hand.weight;
        }
    }
    range
}

fn compute_hero_evs(node: &S3Node) -> Vec<f64> {
    // Weighted average EV across all actions for each hand
    let mut evs = vec![0.0; 169];
    for (name, hand) in &node.hands {
        if let Some(idx) = hand_name_to_index(name) {
            let mut ev = 0.0;
            for (i, &freq) in hand.played.iter().enumerate() {
                if i < hand.evs.len() {
                    ev += freq * hand.evs[i];
                }
            }
            evs[idx] = ev;
        }
    }
    evs
}

fn compute_range_after_action(node: &S3Node, action_index: usize) -> Vec<f64> {
    let mut range = vec![0.0; 169];
    for (name, hand) in &node.hands {
        if let Some(idx) = hand_name_to_index(name) {
            if action_index < hand.played.len() {
                range[idx] = hand.weight * hand.played[action_index];
            }
        }
    }
    range
}

fn compute_evs_for_action(node: &S3Node, action_index: usize) -> Vec<f64> {
    let mut evs = vec![0.0; 169];
    for (name, hand) in &node.hands {
        if let Some(idx) = hand_name_to_index(name) {
            if action_index < hand.evs.len() {
                evs[idx] = hand.evs[action_index];
            }
        }
    }
    evs
}

fn compute_total_combos(range: &[f64]) -> f64 {
    let mut total = 0.0;
    for (i, &w) in range.iter().enumerate() {
        total += w * combo_count(i);
    }
    (total * 100.0).round() / 100.0
}

fn compute_total_ev(range: &[f64], evs: &[f64]) -> Option<f64> {
    let mut sum_ev = 0.0;
    let mut sum_weight = 0.0;
    for i in 0..169 {
        let w = range[i] * combo_count(i);
        sum_ev += w * evs[i];
        sum_weight += w;
    }
    if sum_weight > 0.0 { Some(sum_ev / sum_weight) } else { None }
}

fn build_simple_hand_counters(range: &[f64]) -> HashMap<String, f64> {
    let mut counters = HashMap::new();
    for i in 0..169 {
        if let Some(name) = index_to_hand_name(i) {
            counters.insert(name.to_string(), range[i] * combo_count(i));
        }
    }
    counters
}

fn format_bb_smart(val: f64) -> String {
    let rounded = (val * 1000.0).round() / 1000.0;
    if rounded == rounded.floor() && rounded.abs() < 1e12 {
        format!("{}", rounded as i64)
    } else {
        format!("{rounded:.3}")
    }
}
