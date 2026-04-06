use crate::config::constants::BB_CHIP_VALUE;
use crate::types::s3::{S3Action, S3Node, SolverTree};

pub struct TraversalResult {
    pub node: S3Node,
    pub node_id: u32,
    pub parent_node: Option<S3Node>,
    pub parent_action_index: Option<usize>,
}

pub fn traverse(tree: &SolverTree, preflop_actions: &[String]) -> Result<TraversalResult, String> {
    let mut current_id: u32 = 0;
    let mut current_node = tree
        .nodes
        .get(&0)
        .ok_or("Root node (0) not found")?
        .clone();
    let mut parent_node: Option<S3Node> = None;
    let mut parent_action_idx: Option<usize> = None;

    for (step, action_code) in preflop_actions.iter().enumerate() {
        let matched = find_matching_action(&current_node.actions, action_code, &tree.settings)?;

        parent_node = Some(current_node.clone());
        parent_action_idx = Some(matched.0);

        let next_id = matched
            .1
            .node
            .ok_or_else(|| format!("Terminal node reached at step {step} (action {action_code})"))?;

        current_node = tree
            .nodes
            .get(&next_id)
            .ok_or_else(|| format!("Node {next_id} not found"))?
            .clone();
        current_id = next_id;
    }

    Ok(TraversalResult {
        node: current_node,
        node_id: current_id,
        parent_node,
        parent_action_index: parent_action_idx,
    })
}

fn find_matching_action(
    actions: &[S3Action],
    api_code: &str,
    settings: &crate::types::s3::S3Settings,
) -> Result<(usize, S3Action), String> {
    match api_code {
        "F" => {
            for (i, a) in actions.iter().enumerate() {
                if a.action_type == "F" {
                    return Ok((i, a.clone()));
                }
            }
            Err("No fold action found".into())
        }
        "C" => {
            for (i, a) in actions.iter().enumerate() {
                if a.action_type == "C" {
                    return Ok((i, a.clone()));
                }
            }
            Err("No call action found".into())
        }
        code if code.starts_with("RAI") => {
            let raises: Vec<(usize, &S3Action)> = actions
                .iter()
                .enumerate()
                .filter(|(_, a)| a.action_type == "R")
                .collect();
            raises
                .last()
                .map(|(i, a)| (*i, (*a).clone()))
                .ok_or_else(|| "No raise actions for RAI".into())
        }
        code if code.starts_with('R') => {
            let size_str = &code[1..];
            let size_bb: f64 = size_str
                .parse()
                .map_err(|_| format!("Invalid raise size: {code}"))?;
            let target_amount = (size_bb * BB_CHIP_VALUE as f64).round() as u64;

            for (i, a) in actions.iter().enumerate() {
                if a.action_type == "R" && a.amount == target_amount {
                    return Ok((i, a.clone()));
                }
            }

            // Fallback: try with tolerance
            let tolerance = BB_CHIP_VALUE / 100;
            for (i, a) in actions.iter().enumerate() {
                if a.action_type == "R" && a.amount.abs_diff(target_amount) <= tolerance {
                    return Ok((i, a.clone()));
                }
            }

            Err(format!(
                "No matching raise for {code} (target {target_amount}), available: {:?}",
                actions
                    .iter()
                    .filter(|a| a.action_type == "R")
                    .map(|a| a.amount)
                    .collect::<Vec<_>>()
            ))
        }
        _ => Err(format!("Unknown action code: {api_code}")),
    }
}

pub fn parse_preflop_actions(actions_str: &str) -> Vec<String> {
    if actions_str.is_empty() {
        return vec![];
    }
    actions_str.split('-').map(|s| s.to_string()).collect()
}
