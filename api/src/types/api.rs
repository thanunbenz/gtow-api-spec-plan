use serde::Serialize;
use std::collections::HashMap;

// --- Spot Solution Response ---

#[derive(Debug, Serialize, Clone)]
pub struct SpotSolutionResponse {
    pub action_solutions: Vec<ActionSolution>,
    pub players_info: Vec<PlayerInfo>,
    pub hand_categories_range: Vec<i32>,
    pub draw_categories_range: Vec<i32>,
    pub blocker_rate: Vec<f64>,
    pub unblocker_rate: Vec<f64>,
    pub blockers_frequencies: Option<()>,
    pub game: SpotGame,
    pub usage: Option<()>,
    pub warning: Option<String>,
    pub hands_locked: Option<()>,
}

#[derive(Debug, Serialize, Clone)]
pub struct ActionSolution {
    pub action: ActionDef,
    pub total_frequency: f64,
    pub total_ev: f64,
    pub total_combos: f64,
    pub strategy: Vec<f64>,
    pub evs: Vec<f64>,
    pub hand_categories: Vec<i32>,
    pub draw_categories: Vec<i32>,
    pub equity_buckets: Option<()>,
    pub equity_buckets_advanced: Option<()>,
    pub tournament_evs_converter: Option<()>,
}

#[derive(Debug, Serialize, Clone)]
pub struct ActionDef {
    pub code: String,
    pub position: String,
    #[serde(rename = "type")]
    pub action_type: String,
    pub betsize: String,
    pub allin: bool,
    pub is_hand_end: bool,
    pub is_showdown: bool,
    pub next_street: bool,
    pub display_name: String,
    pub simple_group: String,
    pub advanced_group: String,
    pub betsize_by_pot: Option<String>,
    pub next_position: String,
}

// --- PlayerInfo (spot-solution only) ---

#[derive(Debug, Serialize, Clone)]
pub struct PlayerInfo {
    pub player: PlayerDef,
    pub range: Vec<f64>,
    pub hand_evs: Vec<f64>,
    pub hand_eqs: Vec<f64>,
    pub hand_eqrs: Vec<f64>,
    pub total_ev: Option<f64>,
    pub total_eq: Option<()>,
    pub total_eqr: Option<()>,
    pub pot_share: f64,
    pub total_combos: f64,
    pub simple_hand_counters: HashMap<String, f64>,
    pub equity_buckets_range: Vec<i32>,
    pub equity_buckets_advanced_range: Vec<i32>,
    pub equity_buckets: Vec<i32>,
    pub equity_buckets_advanced: Vec<i32>,
    pub hand_categories: Vec<i32>,
    pub draw_categories: Vec<i32>,
    pub relative_postflop_position: String,
    pub eq_percentile: Vec<f64>,
    pub tournament_evs_converter: Option<()>,
}

#[derive(Debug, Serialize, Clone)]
pub struct PlayerDef {
    pub relative_postflop_position: Option<String>,
    pub hand: Option<String>,
    pub is_dealer: bool,
    pub is_folded: bool,
    pub is_hero: bool,
    pub is_active: bool,
    pub stack: String,
    pub current_stack: String,
    pub chips_on_table: String,
    pub bounty: Option<String>,
    pub profile: Option<String>,
    pub position: String,
    pub bounty_in_bb: Option<String>,
    pub name: String,
    pub seat: usize,
}

// --- Game (spot-solution): field order starts with relative_postflop_position ---

#[derive(Debug, Serialize, Clone)]
pub struct SpotGame {
    pub players: Vec<SpotGamePlayer>,
    pub current_street: StreetInfo,
    pub pot: String,
    pub pot_odds: Option<String>,
    pub active_position: String,
    pub board: String,
    pub bet_display_name: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct SpotGamePlayer {
    pub relative_postflop_position: Option<String>,
    pub hand: Option<String>,
    pub is_dealer: bool,
    pub is_folded: bool,
    pub is_hero: bool,
    pub is_active: bool,
    pub stack: String,
    pub current_stack: String,
    pub chips_on_table: String,
    pub bounty: Option<String>,
    pub profile: Option<String>,
    pub position: String,
    pub bounty_in_bb: Option<String>,
}

// --- Game (next-actions): field order starts with position ---

#[derive(Debug, Serialize, Clone)]
pub struct NextGame {
    pub players: Vec<NextGamePlayer>,
    pub current_street: StreetInfo,
    pub pot: String,
    pub pot_odds: Option<String>,
    pub active_position: String,
    pub board: String,
    pub bet_display_name: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct NextGamePlayer {
    pub position: String,
    pub relative_postflop_position: Option<String>,
    pub hand: Option<String>,
    pub is_dealer: bool,
    pub is_folded: bool,
    pub is_hero: bool,
    pub is_active: bool,
    pub stack: String,
    pub current_stack: String,
    pub chips_on_table: String,
    pub bounty: Option<String>,
    pub bounty_in_bb: Option<String>,
    pub profile: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct StreetInfo {
    #[serde(rename = "type")]
    pub street_type: String,
    pub start_pot: String,
    pub end_pot: String,
}

// --- Next Actions Response ---

#[derive(Debug, Serialize, Clone)]
pub struct NextActionsResponse {
    pub next_actions: NextActionsNode,
    pub future_actions: Vec<()>,
}

#[derive(Debug, Serialize, Clone)]
pub struct NextActionsNode {
    pub game: NextGame,
    pub available_actions: Vec<AvailableAction>,
    pub custom_solution_id: Option<String>,
    pub is_node_locked: bool,
    pub is_edited: bool,
    pub is_editable: bool,
    pub forced_fold: bool,
    pub available_node_edits: Option<()>,
    pub merged_actions: Vec<String>,
    pub preset_action_code: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct AvailableAction {
    pub action: ActionDef,
    pub frequency: Option<f64>,
    pub is_solution_end: bool,
    pub can_be_solved_by_ai: bool,
    pub next_position: String,
    pub selected: bool,
}
