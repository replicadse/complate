#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Config {
    pub version: String,
    #[serde(default)]
    pub templates: std::collections::BTreeMap<String, Template>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Template {
    pub content: Content,
    #[serde(default)]
    pub values: std::collections::BTreeMap<String, VariableDefinition>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Content {
    File(String),
    Inline(String),
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Option {
    pub display: String,
    pub value: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VariableDefinition {
    Static(String),
    Prompt(String),
    Shell(String),
    Select { text: String, options: Vec<Option> },
    Check { text: String, options: Vec<Option> },
}

impl Config {
    pub fn new(version: String) -> Self {
        Self {
            version,
            templates: std::collections::BTreeMap::new(),
        }
    }
}

impl Template {
    pub fn new(content: Content) -> Self {
        Self {
            content,
            values: std::collections::BTreeMap::new(),
        }
    }
}
