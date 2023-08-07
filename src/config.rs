use {
    indoc::indoc,
    std::collections::{
        BTreeMap,
        HashMap,
    },
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

pub async fn default_config() -> String {
    indoc! {
      r#"version: 0.13
    templates:
      zero:
        content:
          inline: |-
            {{ a.alpha }}
            {{ b.bravo }}
        variables:
          a.alpha:
            static: alpha
          b.bravo: arg

      one:
        content:
          file: ./.complate/templates/arbitraty-template-file.tpl
        variables:
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
        variables:
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
            {{ test }}
            {{ _decode "dGVzdA==" }}
        helpers:
          "_decode": printf "$(printf $VALUE | base64 -D)"
        variables:
          test:
            static: "test"
"#

    }
    .into()
}
