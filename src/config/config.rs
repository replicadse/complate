#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Config {
    pub version: String,
    pub templates: std::collections::BTreeMap<String, Template>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all="snake_case")]
pub struct Template {
    pub content: Content,
    pub values: std::collections::BTreeMap<String, ValueDefinition>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all="snake_case")]
pub enum Content {
    File(String),
    Inline(String),
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all="snake_case")]
pub enum ValueDefinition {
    Static(String),
    Prompt(String),
    Shell(String),
    Select{text: String, options: Vec<String>},
    Check{text: String, options: Vec<String>},
}

impl Config {
    pub fn new(version: String) -> Self {
        Self {
            version: version,
            templates: std::collections::BTreeMap::new(),
        }
    }
}

impl Template {
    pub fn new(content: Content) -> Self {
        Self {
            content: content,
            values: std::collections::BTreeMap::new(),
        }
    }
}
