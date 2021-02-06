use super::UserInput;
use async_trait::async_trait;
use std::io::Result;

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
    async fn prompt(&self, text: &str) -> Result<String> {
        dialoguer::Input::new()
            .allow_empty(true)
            .with_prompt(text)
            .interact()
    }

    async fn shell(&self, command: &str, shell_trust: &super::ShellTrust) -> Result<String> {
        super::shell(command, shell_trust, &super::Backend::CLI).await
    }

    async fn select(
        &self,
        prompt: &str,
        options: &std::collections::BTreeMap<String, super::Option>,
    ) -> Result<String> {
        let keys = options.keys().cloned().collect::<Vec<String>>();
        let display_vals = options
            .values()
            .map(|x| x.display.to_owned())
            .collect::<Vec<String>>();

        let result_idx = dialoguer::Select::new()
            .with_prompt(prompt)
            .items(&display_vals)
            .default(0)
            .paged(false)
            .interact()?;
        match &options[&keys[result_idx]].value {
            super::OptionValue::Static(x) => Ok(x.to_owned()),
            super::OptionValue::Shell(cmd) => self.shell(cmd, &self.shell_trust).await,
        }
    }

    async fn check(
        &self,
        prompt: &str,
        separator: &str,
        options: &std::collections::BTreeMap<String, super::Option>,
    ) -> Result<String> {
        let keys = options.keys().cloned().collect::<Vec<String>>();
        let display_vals = options
            .values()
            .map(|x| x.display.to_owned())
            .collect::<Vec<String>>();

        let indices = dialoguer::MultiSelect::new()
            .with_prompt(prompt)
            .items(&display_vals)
            .interact()
            .unwrap();

        match indices.len() {
            0usize => Ok("".to_owned()),
            _ => {
                let mut d = String::new();
                for i in indices {
                    let v = match &options[&keys[i]].value {
                        super::OptionValue::Static(x) => x.to_owned(),
                        super::OptionValue::Shell(cmd) => {
                            self.shell(&cmd, &self.shell_trust).await?
                        }
                    };
                    d.push_str(&v);
                    d.push_str(separator);
                }
                d.truncate(d.len() - 2);
                Ok(d)
            }
        }
    }
}
