use crate::services::node_traversal::TraversalResult;
use crate::transform::action_builder::build_action_def;
use crate::transform::game_state::{build_next_game, compute_current_bet_bb, compute_pot_bb, count_remaining_players};
use crate::types::api::{AvailableAction, NextActionsNode, NextActionsResponse};
use crate::types::s3::{S3Settings, SolverTree};

pub fn build_next_actions(
    traversal: &TraversalResult,
    settings: &S3Settings,
    tree: &SolverTree,
) -> NextActionsResponse {
    let node = &traversal.node;
    let num_players = settings.handdata.stacks.len();
    let pot_bb = compute_pot_bb(node, settings);
    let current_bet_bb = compute_current_bet_bb(node, settings);
    let remaining = count_remaining_players(node, num_players);

    let mut available_actions = Vec::with_capacity(node.actions.len());

    for s3_action in &node.actions {
        let action_def = build_action_def(
            s3_action, node, settings, tree, pot_bb, current_bet_bb, remaining,
        );

        let is_solution_end = s3_action.node.is_none();
        let can_be_solved_by_ai = action_def.next_street;
        let next_position = action_def.next_position.clone();

        available_actions.push(AvailableAction {
            action: action_def,
            frequency: None,
            is_solution_end,
            can_be_solved_by_ai,
            next_position,
            selected: false,
        });
    }

    let preset_action_code = available_actions
        .first()
        .map(|a| a.action.code.clone());

    let game = build_next_game(node, settings);

    NextActionsResponse {
        next_actions: NextActionsNode {
            game,
            available_actions,
            custom_solution_id: None,
            is_node_locked: false,
            is_edited: false,
            is_editable: false,
            forced_fold: false,
            available_node_edits: None,
            merged_actions: vec![],
            preset_action_code,
        },
        future_actions: vec![],
    }
}
