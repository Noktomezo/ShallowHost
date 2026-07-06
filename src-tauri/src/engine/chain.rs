#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChainItem {
    pub id: String,
    pub name: String,
    pub vendor: String,
    pub format: String,
    pub bypassed: bool,
    pub unique_id: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ParamInfo {
    pub index: usize,
    pub name: String,
    pub unit: String,
    pub min: f64,
    pub max: f64,
    pub default: f64,
    pub step_count: u32,
    pub value: f64,
}
