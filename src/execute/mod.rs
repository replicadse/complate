use crate::args::ShellTrust;
use crate::config::ValueDefinition;
use async_trait::async_trait;
use std::io::{Error, ErrorKind, Result};

#[async_trait]
pub trait Execute {
    async fn execute(&self, shell_trust: &ShellTrust) -> Result<String>;
}

#[async_trait]
impl Execute for ValueDefinition {
    async fn execute(&self, shell_trust: &ShellTrust) -> Result<String> {
        match self {
            ValueDefinition::Static(v) => Ok(v.to_owned()),
            ValueDefinition::Prompt(v) => prompt(v).await,
            ValueDefinition::Shell(cmd) => shell(cmd, shell_trust).await,
            ValueDefinition::Select { text, options } => select(text, options).await,
            ValueDefinition::Check { text, options } => check(text, options).await,
        }
    }
}

async fn prompt(text: &str) -> Result<String> {
    dialoguer::Input::new()
        .allow_empty(true)
        .with_prompt(text)
        .interact()
}

async fn shell(command: &str, shell_trust: &ShellTrust) -> Result<String> {
    match shell_trust {
        ShellTrust::None => return Err(Error::new(ErrorKind::Other, "no shell trust")),
        ShellTrust::Prompt => {
            let exec = dialoguer::Confirmation::new()
                .with_text(&format!("You are about to run a shell command. The command is:\n{}\nDo you confirm the execution?", command))
                .interact()?;
            if !exec {
                return Err(Error::new(
                    ErrorKind::Other,
                    "user declined command execution",
                ));
            }
        }
        ShellTrust::Ultimate => {}
    }

    let output = std::process::Command::new("sh")
        .arg("-c")
        .arg(command)
        .output()?;
    if output.status.code().unwrap() != 0 {
        return Err(Error::new(ErrorKind::Other, "failed to run command"));
    }
    Ok(String::from_utf8(output.stdout).unwrap())
}

async fn select(prompt: &str, options: &[String]) -> Result<String> {
    let idx = dialoguer::Select::new()
        .with_prompt(prompt)
        .items(options)
        .default(0)
        .interact()?;
    Ok(options[idx].to_owned())
}

async fn check(prompt: &str, options: &[String]) -> Result<String> {
    let indices = dialoguer::Checkboxes::new()
        .with_prompt(prompt)
        .items(options)
        .interact()
        .unwrap();

    match indices.len() {
        0usize => Ok("".to_owned()),
        _ => {
            let mut d = String::new();
            for i in indices {
                d.push_str(&options[i]);
                d.push_str(", ");
            }
            d.truncate(d.len() - 2);
            Ok(d)
        }
    }
}
