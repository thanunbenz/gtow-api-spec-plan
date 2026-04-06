use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize, Clone)]
pub struct S3Node {
    pub player: usize,
    pub street: u32,
    pub children: u32,
    #[serde(default)]
    pub sequence: Vec<S3SequenceAction>,
    #[serde(default)]
    pub actions: Vec<S3Action>,
    pub hands: HashMap<String, S3Hand>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct S3SequenceAction {
    pub player: usize,
    #[serde(rename = "type")]
    pub action_type: String,
    pub amount: u64,
    pub street: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct S3Action {
    #[serde(rename = "type")]
    pub action_type: String,
    pub amount: u64,
    pub node: Option<u32>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct S3Hand {
    pub weight: f64,
    pub played: Vec<f64>,
    pub evs: Vec<f64>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct S3Settings {
    pub handdata: HandData,
    pub eqmodel: EqModel,
}

#[derive(Debug, Deserialize, Clone)]
pub struct HandData {
    pub stacks: Vec<u64>,
    pub blinds: Vec<u64>,
    #[serde(rename = "skipSb", default)]
    pub skip_sb: bool,
    #[serde(rename = "anteType", default)]
    pub ante_type: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct EqModel {
    pub id: String,
    #[serde(default)]
    pub structure: Option<EqStructure>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct EqStructure {
    pub name: Option<String>,
    pub chips: Option<f64>,
    pub prizes: Option<HashMap<String, f64>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct S3Equity {
    #[serde(rename = "equityUnit")]
    pub equity_unit: String,
    #[serde(rename = "conversionFactors")]
    pub conversion_factors: ConversionFactors,
    #[serde(rename = "preHandEquity")]
    pub pre_hand_equity: Vec<f64>,
    #[serde(rename = "bubbleFactors")]
    pub bubble_factors: Vec<Vec<f64>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ConversionFactors {
    #[serde(rename = "toUSD")]
    pub to_usd: f64,
    #[serde(rename = "toRegularPrizePercent")]
    pub to_regular_prize_percent: f64,
}

#[derive(Debug, Clone)]
pub struct SolverTree {
    pub settings: S3Settings,
    pub equity: S3Equity,
    pub nodes: HashMap<u32, S3Node>,
}
