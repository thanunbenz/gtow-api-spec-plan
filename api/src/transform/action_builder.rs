use crate::config::constants::{POSITIONS_8, POSITIONS_9};
use crate::transform::game_state::s3_to_api_seat;
use crate::types::api::ActionDef;
use crate::types::s3::{S3Action, S3Node, S3Settings, SolverTree};

pub fn build_action_def(
    s3_action: &S3Action,
    node: &S3Node,
    settings: &S3Settings,
    tree: &SolverTree,
    pot_bb: f64,
    current_bet_bb: f64,
    remaining_players: usize,
) -> ActionDef {
    let bb = settings.handdata.blinds[0] as f64;
    let position = POSITIONS_9[s3_to_api_seat(node.player)];
    let betsize_bb = s3_action.amount as f64 / bb;

    let is_allin = is_all_in(s3_action, node, settings);
    let code = build_action_code(s3_action, is_allin, betsize_bb);
    let action_type = match s3_action.action_type.as_str() {
        "F" => "FOLD",
        "C" => "CALL",
        "R" => "RAISE",
        other => other,
    };

    let display_name = if is_allin { "ALLIN" } else { action_type };

    let simple_group = match action_type {
        "FOLD" => "FOLD",
        "CALL" => "CALL",
        "RAISE" => "RAISE",
        _ => action_type,
    };

    let advanced_group = match action_type {
        "FOLD" => "FOLD".to_string(),
        "CALL" => "CALL".to_string(),
        "RAISE" => classify_raise_size(betsize_bb, current_bet_bb, pot_bb),
        _ => action_type.to_string(),
    };

    // betsize_by_pot: string with full precision, null for fold/call
    let betsize_by_pot = if action_type == "RAISE" && pot_bb > 0.0 {
        let ratio = (betsize_bb - current_bet_bb) / pot_bb;
        Some(format_ratio(ratio))
    } else {
        None
    };

    let is_hand_end = action_type == "FOLD" && remaining_players <= 2;

    // next_street: true when CALL closes the action (goes to flop)
    let next_street = action_type == "CALL" && is_call_closing_action(s3_action, node, tree);

    let next_position = determine_next_position(s3_action, node, tree);

    ActionDef {
        code,
        position: position.to_string(),
        action_type: action_type.to_string(),
        betsize: format_betsize(betsize_bb),
        allin: is_allin,
        is_hand_end,
        is_showdown: false,
        next_street,
        display_name: display_name.to_string(),
        simple_group: simple_group.to_string(),
        advanced_group,
        betsize_by_pot,
        next_position,
    }
}

fn build_action_code(s3_action: &S3Action, is_allin: bool, betsize_bb: f64) -> String {
    match s3_action.action_type.as_str() {
        "F" => "F".to_string(),
        "C" => "C".to_string(),
        "R" if is_allin => "RAI".to_string(),
        "R" => {
            let rounded = (betsize_bb * 1000.0).round() / 1000.0;
            if rounded == rounded.floor() {
                format!("R{}", rounded as u64)
            } else {
                format!("R{rounded}")
            }
        }
        other => other.to_string(),
    }
}

fn is_all_in(s3_action: &S3Action, node: &S3Node, settings: &S3Settings) -> bool {
    if s3_action.action_type != "R" {
        return false;
    }
    // Method 1: last raise in actions[] is typically all-in
    let raises: Vec<&S3Action> = node.actions.iter().filter(|a| a.action_type == "R").collect();
    if let Some(last_raise) = raises.last() {
        if last_raise.amount == s3_action.amount {
            return true;
        }
    }
    // Method 2: compare with effective stack (stack minus ante)
    if node.player < settings.handdata.stacks.len() {
        let player_stack = settings.handdata.stacks[node.player];
        let ante = if settings.handdata.blinds.len() > 2 { settings.handdata.blinds[2] } else { 0 };
        s3_action.amount >= player_stack.saturating_sub(ante)
    } else {
        false
    }
}

fn is_call_closing_action(s3_action: &S3Action, node: &S3Node, tree: &SolverTree) -> bool {
    // A call closes the action if the child node moves to street > 0
    // or if it's a terminal (heads-up pot going to flop)
    if let Some(child_id) = s3_action.node {
        if let Some(child) = tree.nodes.get(&child_id) {
            return child.street > node.street;
        }
    }
    // Terminal call = goes to showdown/next street
    s3_action.node.is_none() && s3_action.action_type == "C"
}

fn classify_raise_size(betsize_bb: f64, current_bet_bb: f64, pot_bb: f64) -> String {
    if pot_bb <= 0.0 {
        return "BET_SMALL".to_string();
    }
    let ratio = (betsize_bb - current_bet_bb) / pot_bb;
    if ratio <= 0.4 {
        "BET_SMALL".to_string()
    } else if ratio <= 0.75 {
        "BET_MEDIUM".to_string()
    } else if ratio <= 1.0 {
        "BET_LARGE".to_string()
    } else {
        "BET_OVERBET".to_string()
    }
}

pub fn player_to_position(player: usize) -> &'static str {
    if player < POSITIONS_8.len() {
        POSITIONS_8[player]
    } else {
        "UNKNOWN"
    }
}

fn determine_next_position(
    s3_action: &S3Action,
    node: &S3Node,
    tree: &SolverTree,
) -> String {
    // If child node exists, use its player field
    if let Some(child_id) = s3_action.node {
        if let Some(child) = tree.nodes.get(&child_id) {
            let api_seat = s3_to_api_seat(child.player);
            return POSITIONS_9[api_seat].to_string();
        }
    }
    // Terminal — next player in table order (9-seat)
    let api_seat = s3_to_api_seat(node.player);
    let next_api = (api_seat + 1) % 9;
    POSITIONS_9[next_api].to_string()
}

/// Smart betsize format: "0", "2" for integers, "20.000", "6.000" for decimals (3 dec)
fn format_betsize(val: f64) -> String {
    let rounded = (val * 1000.0).round() / 1000.0;
    if rounded == rounded.floor() && rounded.abs() < 1e12 {
        format!("{}", rounded as i64)
    } else {
        format!("{rounded:.3}")
    }
}

/// Format ratio with enough precision (spec: "0.2758620689655172", "1.02752294")
fn format_ratio(val: f64) -> String {
    // Trim trailing zeros but keep at least some precision
    let s = format!("{val:.15}");
    let s = s.trim_end_matches('0');
    let s = s.trim_end_matches('.');
    s.to_string()
}
