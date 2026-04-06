use actix_web::{web, HttpResponse};
use serde::Deserialize;

use crate::services::node_traversal::{parse_preflop_actions, traverse};
use crate::services::tree_loader::{load_tree, stacks_to_folder};
use crate::transform::spot_solution::build_spot_solution;
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct SpotSolutionQuery {
    pub gametype: String,
    pub depth: String,
    pub stacks: String,
    #[serde(default)]
    pub preflop_actions: String,
}

pub async fn handle_spot_solution(
    query: web::Query<SpotSolutionQuery>,
    state: web::Data<AppState>,
) -> HttpResponse {
    // Parse stacks
    let stacks_bb: Vec<f64> = query
        .stacks
        .split('-')
        .filter_map(|s| s.parse().ok())
        .collect();

    if stacks_bb.is_empty() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Invalid stacks format"
        }));
    }

    let folder = stacks_to_folder(&stacks_bb);

    // Load tree
    let tree = match load_tree(&state.data_source, &folder).await {
        Ok(t) => t,
        Err(e) => {
            return HttpResponse::NotFound().json(serde_json::json!({
                "error": format!("Solution not found: {e}")
            }));
        }
    };

    // Parse actions and traverse
    let actions = parse_preflop_actions(&query.preflop_actions);
    let traversal = match traverse(&tree, &actions) {
        Ok(t) => t,
        Err(e) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": format!("Traversal failed: {e}")
            }));
        }
    };

    // Build response
    let response = build_spot_solution(&traversal, &tree.settings, &tree);

    HttpResponse::Ok()
        .insert_header(("Cache-Control", "public, max-age=86400, immutable"))
        .json(response)
}
