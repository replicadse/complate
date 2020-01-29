#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Config {
    pub templates: std::collections::HashMap<String, Template>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Template {
    #[serde(rename="content")]
    pub content: Content,
    #[serde(rename="values")]
    pub values: std::collections::HashMap<String, ValueProvider>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum Content {
    #[serde(rename="file")]
    File(String),
    #[serde(rename="inline")]
    Inline(String),
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum ValueProvider {
    #[serde(rename="static")]
    Static(String),
    #[serde(rename="prompt")]
    Prompt(String),
    #[serde(rename="shell")]
    Shell(String),
}

impl Config {
    pub fn new() -> Self {
        Self {
            templates: std::collections::HashMap::new(),
        }
    }
}

impl Template {
    pub fn new(content: Content) -> Self {
        Self {
            content: content,
            values: std::collections::HashMap::new(),
        }
    }
}
