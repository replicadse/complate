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
    #[serde(default)]
    pub helpers: std::collections::BTreeMap<String, Helper>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Helper {
    pub shell: String,
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

pub async fn default_config() -> String {
    r###"version: 0.11
templates:
  zero:
    content:
      inline: |-
        {{ a.alpha }}
    values:
      a.alpha:
        static: ALPHA
  one:
    content:
      file: ./.complate/templates/arbitraty-template-file.tpl
    values:
      a.pwd:
        env: "PWD"
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
  three:
    content:
      inline: |-
        {{ _decode "dGVzdA==" }}
    helpers:
      "_decode":
        shell: |-
          printf "$(printf $VALUE | base64 -D)"
    values: {}

"###
    .to_owned()
}
