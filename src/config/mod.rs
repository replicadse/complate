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
pub enum OptionValue {
    Static(String),
    Shell(String),
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Option {
    pub display: String,
    pub value: OptionValue,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VariableDefinition {
    Env(String),
    Static(String),
    Prompt(String),
    Shell(String),
    Select {
        text: String,
        options: std::collections::BTreeMap<String, Option>,
    },
    Check {
        text: String,
        separator: String,
        options: std::collections::BTreeMap<String, Option>,
    },
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

pub async fn default_config() -> String {
    r###"version: 0.10
templates:
    zero:
        content:
            inline: |-
                {{ a.alpha }}
        values:
            a.alpha:
                env: ALPHA
    one:
        content:
            file: ./.complate/templates/arbitraty-template-file.tpl
        values:
            a.summary:
                env: "random summary"
    two:
        content:
            inline: |-
                {{ a.alpha }}
                {{ b.bravo }}
                {{ c.charlie }}
                {{ d.delta }}
                {{ e.echo }}
        values:
            a.alpha:
              prompt: "alpha"
            b.bravo:
              shell: "printf bravo"
            c.charlie:
              static: "charlie"
            d.delta:
                select:
                    text: Select the version level that shall be incremented
                    options:
                      alpha:
                        display: alpha
                        value:
                          static: alpha
                      bravo:
                        display: bravo
                        value:
                          shell: printf bravo
            e.echo:
                check:
                    text: Select the components that are affected
                    separator: ", "
                    options:
                      alpha:
                        display: alpha
                        value:
                          static: alpha
                      bravo:
                        display: bravo
                        value:
                          shell: printf bravo
            f.foxtrot:
                env: "FOXTROT"
"###
    .to_owned()
}
