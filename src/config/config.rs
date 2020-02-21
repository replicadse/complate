#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Config {
    pub version: String,
    pub templates: std::collections::BTreeMap<String, Template>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Template {
    #[serde(rename="content")]
    pub content: Content,
    #[serde(rename="values")]
    pub values: std::collections::BTreeMap<String, ValueDefinition>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum Content {
    #[serde(rename="file")]
    File(String),
    #[serde(rename="inline")]
    Inline(String),
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum ValueDefinition {
    #[serde(rename="static")]
    Static(String),
    #[serde(rename="prompt")]
    Prompt(String),
    #[serde(rename="shell")]
    Shell(String),
    #[serde(rename="select")]
    Select{text: String, options: Vec<String>},
    #[serde(rename="check")]
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
