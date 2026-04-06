use crate::config::constants::combo_count;
use crate::services::node_traversal::TraversalResult;
use crate::transform::action_builder::build_action_def;
use crate::transform::game_state::{build_spot_game, compute_current_bet_bb, compute_pot_bb, count_remaining_players};
use crate::transform::hand_index::hand_name_to_index;
use crate::transform::player_info::build_players_info;
use crate::types::api::{ActionSolution, SpotSolutionResponse};
use crate::types::s3::{S3Settings, SolverTree};

pub fn build_spot_solution(
    traversal: &TraversalResult,
    settings: &S3Settings,
    tree: &SolverTree,
) -> SpotSolutionResponse {
    let node = &traversal.node;
    let num_players = settings.handdata.stacks.len();
    let pot_bb = compute_pot_bb(node, settings);
    let current_bet_bb = compute_current_bet_bb(node, settings);
    let remaining = count_remaining_players(node, num_players);

    let mut action_solutions = Vec::with_capacity(node.actions.len());

    for (i, s3_action) in node.actions.iter().enumerate() {
        let mut strategy = vec![0.0; 169];
        let mut evs = vec![0.0; 169];

        for (name, hand) in &node.hands {
            if let Some(idx) = hand_name_to_index(name) {
                if i < hand.played.len() {
                    strategy[idx] = hand.played[i];
                }
                if i < hand.evs.len() {
                    evs[idx] = hand.evs[i];
                }
            }
        }

        let mut total_weighted_freq = 0.0;
        let mut total_combos = 0.0;
        let mut total_ev_sum = 0.0;
        let mut total_range_combos = 0.0;

        for idx in 0..169 {
            let combos = combo_count(idx);
            let weight = get_hand_weight_by_index(node, idx);
            let freq = strategy[idx];
            let ev = evs[idx];

            total_weighted_freq += freq * combos * weight;
            total_combos += freq * combos * weight;
            total_ev_sum += ev * freq * combos * weight;
            total_range_combos += combos * weight;
        }

        let total_frequency = if total_range_combos > 0.0 {
            total_weighted_freq / total_range_combos
        } else {
            0.0
        };

        let total_ev = if total_combos > 0.0 {
            total_ev_sum / total_combos
        } else {
            0.0
        };

        let action_def = build_action_def(
            s3_action, node, settings, tree, pot_bb, current_bet_bb, remaining,
        );

        action_solutions.push(ActionSolution {
            action: action_def,
            total_frequency,
            total_ev,
            total_combos: (total_combos * 100.0).round() / 100.0,
            strategy,
            evs,
            hand_categories: vec![],
            draw_categories: vec![],
            equity_buckets: None,
            equity_buckets_advanced: None,
            tournament_evs_converter: None,
        });
    }

    let players_info = build_players_info(
        node,
        settings,
        traversal.parent_node.as_ref(),
        traversal.parent_action_index,
    );

    let game = build_spot_game(node, settings);

    SpotSolutionResponse {
        action_solutions,
        players_info,
        hand_categories_range: vec![],
        draw_categories_range: vec![],
        blocker_rate: vec![],
        unblocker_rate: vec![],
        blockers_frequencies: None,
        game,
        usage: None,
        warning: None,
        hands_locked: None,
    }
}

fn get_hand_weight_by_index(node: &crate::types::s3::S3Node, target_idx: usize) -> f64 {
    for (name, hand) in &node.hands {
        if let Some(idx) = hand_name_to_index(name) {
            if idx == target_idx {
                return hand.weight;
            }
        }
    }
    0.0
}
