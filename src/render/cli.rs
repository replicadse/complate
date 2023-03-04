use crate::error::Error;

use super::UserInput;
use async_trait::async_trait;
use std::{collections::HashMap, result::Result};

pub struct CLIBackend<'a> {
    shell_trust: &'a super::ShellTrust,
}

impl<'a> CLIBackend<'a> {
    pub fn new(shell_trust: &'a super::ShellTrust) -> Self {
        Self { shell_trust }
    }
}

#[async_trait]
impl<'a> UserInput for CLIBackend<'a> {
    async fn prompt(&self, text: &str) -> Result<String, Box<dyn std::error::Error>> {
        match dialoguer::Input::new().allow_empty(true).with_prompt(text).interact() {
            | Ok(res) => Ok(res),
            | Err(_) => Err(Box::new(Error::InteractAbort)),
        }
    }

    async fn select(
        &self,
        prompt: &str,
        options: &std::collections::BTreeMap<String, super::Option>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let keys = options.keys().cloned().collect::<Vec<String>>();
        let display_vals = options.values().map(|x| x.display.to_owned()).collect::<Vec<String>>();

        let result_idx = dialoguer::Select::new()
            .with_prompt(prompt)
            .items(&display_vals)
            .default(0)
            .paged(false)
            .interact()?;
        match &options[&keys[result_idx]].value {
            | super::OptionValue::Static(x) => Ok(x.into()),
            | super::OptionValue::Shell(cmd) => super::shell(cmd, &HashMap::new(), &self.shell_trust).await,
        }
    }

    async fn check(
        &self,
        prompt: &str,
        separator: &str,
        options: &std::collections::BTreeMap<String, super::Option>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let keys = options.keys().cloned().collect::<Vec<String>>();
        let display_vals = options.values().map(|x| x.display.to_owned()).collect::<Vec<String>>();

        let indices = dialoguer::MultiSelect::new()
            .with_prompt(prompt)
            .items(&display_vals)
            .interact()?;

        match indices.len() {
            | 0usize => Ok("".into()),
            | _ => {
                let mut d = String::new();
                for i in indices {
                    let v = match &options[&keys[i]].value {
                        | super::OptionValue::Static(x) => x.into(),
                        | super::OptionValue::Shell(cmd) => {
                            super::shell(&cmd, &HashMap::new(), &self.shell_trust).await?
                        },
                    };
                    d.push_str(&v);
                    d.push_str(separator);
                }
                d.truncate(d.len() - 2);
                Ok(d)
            },
        }
    }
}
