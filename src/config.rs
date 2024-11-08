use std::collections::{
    BTreeMap,
    HashMap,
};

#[derive(Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    pub version: String,
    #[serde(with = "serde_yaml::with::singleton_map_recursive")]
    #[schemars(with = "BTreeMap<String, Template>")]
    pub templates: BTreeMap<String, Template>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub struct Template {
    pub content: Content,
    #[schemars(with = "std::option::Option<HashMap<String, VariableDefinition>>")]
    pub variables: std::option::Option<HashMap<String, VariableDefinition>>,
    #[schemars(with = "std::option::Option<HashMap<String, String>>")]
    pub helpers: std::option::Option<HashMap<String, String>>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum Content {
    File(String),
    Inline(String),
}

#[derive(Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum OptionValue {
    Static(String),
    Shell(String),
}

#[derive(Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub struct Option {
    pub display: String,
    pub value: OptionValue,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum VariableDefinition {
    Arg,
    Env(String),
    Static(String),
    Prompt(String),
    Shell(String),
    Select {
        text: String,
        #[schemars(with = "BTreeMap<String, Option>")]
        options: BTreeMap<String, Option>,
    },
    Check {
        text: String,
        separator: String,
        #[schemars(with = "BTreeMap<String, Option>")]
        options: BTreeMap<String, Option>,
    },
}

pub async fn default_config() -> &'static str {
    include_str!("../.complate/config.yaml")
}
