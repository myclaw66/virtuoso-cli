use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CornerConfig {
    pub simulator: Option<String>,
    pub design: DesignTarget,
    pub model_file: String,
    pub analysis: AnalysisConfig,
    pub corners: Vec<Corner>,
    pub measures: Vec<Measure>,
}

#[derive(Debug, Deserialize)]
pub struct DesignTarget {
    pub lib: String,
    pub cell: String,
    #[serde(default = "default_view")]
    pub view: String,
}

fn default_view() -> String {
    "schematic".into()
}

#[derive(Debug, Deserialize)]
pub struct AnalysisConfig {
    #[serde(rename = "type")]
    pub analysis_type: String,
    #[serde(flatten)]
    pub params: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct Corner {
    pub name: String,
    pub section: String,
    pub temp: f64,
    #[serde(flatten)]
    pub vars: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct Measure {
    pub name: String,
    pub expr: String,
}
