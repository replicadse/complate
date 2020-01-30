extern crate clap;
extern crate dialoguer;
extern crate handlebars;
extern crate serde;
extern crate serde_json;
extern crate serde_yaml;

use std::io::{Result, Write};
use std::collections::{HashMap};
use futures::executor::{block_on};

pub mod args;
use args::args::ShellTrust;
pub mod config;
use config::config::{Config, Template, Content, ValueProvider};

async fn select_template<'a>(config: &'a Config) -> Result<&'a Template> {
    let keys: Vec<String> = config.templates.keys().map(
        |t| t.to_owned()
    ).collect();
    let selection = dialoguer::Select::new()
        .items(keys.as_slice())
        .default(0)
        .paged(false)
        .interact()?;
    Ok(config.templates.get(&keys[selection]).unwrap())
}

async fn get_values(template: &Template, shell_trust: &ShellTrust) -> Result<HashMap<String, String>> {
    let mut values = HashMap::new();
    for value in &template.values {
        match value.1 {
            ValueProvider::Static(v) => { values.insert(value.0.to_owned(), v.to_owned()); },
            ValueProvider::Prompt(v) => {
                values.insert(
                    value.0.to_owned(), 
                    dialoguer::Input::new()
                        .allow_empty(true)
                        .with_prompt(&v)
                        .interact()
                        .unwrap());
            },
            ValueProvider::Shell(command) => {
                match shell_trust {
                    ShellTrust::None => {
                        panic!("could not execute shell command due to the trust level\n{}", command);
                    },
                    ShellTrust::Prompt => {
                        let exec = dialoguer::Confirmation::new()
                            .with_text(&format!("You are about to run a shell command. The command is:\n{}\nDo you confirm the execution?", command))
                            .interact().unwrap();
                            match exec {
                                false => {
                                    panic!("user declined command execution for command {}", command);
                                }
                                true => {},
                            }
                    },
                    ShellTrust::Ultimate => {},
                }
                let output = std::process::Command::new("sh")
                    .arg("-c")
                    .arg(command)
                    .output()?;
                if output.status.code().unwrap() != 0 {
                    panic!("failed to run command {}", command);
                }
                values.insert(value.0.to_owned(), String::from_utf8(output.stdout).unwrap());
            },
            ValueProvider::Select(items) => {
                values.insert(value.0.to_owned(),
                    items[dialoguer::Select::new()
                        .with_prompt(value.0)
                        .items(items.as_slice())
                        .default(0)
                        .interact()
                        .unwrap()].to_owned());
            },
            ValueProvider::Check(items) => {
                let indices = dialoguer::Checkboxes::new()
                    .with_prompt(value.0)
                    .items(items)
                    .interact().unwrap();

                values.insert(value.0.to_owned(),
                    match indices.len() {
                        0usize => "".to_owned(),
                        _ => {
                            let mut d = String::new();
                            for i in indices {
                                d.push_str(&items[i]);
                                d.push_str(", ");
                            }
                            d.truncate(d.len() - 2);
                            d
                        }
                    }
                );
            }
        }
    }
    Ok(values)
}

async fn replace(template: &str, values: &HashMap<String, String>) -> Result<String> {
    fn recursive_add(namespace: &mut std::collections::VecDeque<String>, parent: &mut serde_json::Value, value: &str) {
        let current_namespace = namespace.pop_front().unwrap();
        match namespace.len() {
            0 => {
                parent.as_object_mut().unwrap().insert(current_namespace.to_owned(), serde_json::Value::String(value.to_owned()));
            }
            _ => {
                parent.as_object_mut().unwrap().insert(current_namespace.to_owned(), serde_json::Value::Object(serde_json::Map::new()));
                recursive_add(namespace, parent.as_object_mut().unwrap().get_mut(&current_namespace).unwrap(), value);
            }
        }
    }

    let mut hb = handlebars::Handlebars::new();
    hb.set_strict_mode(true);

    let mut values_json = serde_json::Value::Object(serde_json::Map::new());
    for val in values {
        let namespaces_vec: Vec<String> = val.0.split(".").map(|s| s.to_string()).collect();
        let mut namespaces = std::collections::VecDeque::from(namespaces_vec);
        recursive_add(&mut namespaces, &mut values_json, val.1);
    }

    let rendered_template = hb.render_template(template, &values_json).unwrap();
    Ok(rendered_template)
}

async fn async_main() -> Result<()> {
    let args = crate::args::args::ClapArgumentLoader::load_from_cli().await?;
    let cfg: Config = serde_yaml::from_str(&args.configuration).unwrap();

    let template = select_template(&cfg).await?;
    let template_str = match &template.content {
        Content::Inline(x) => x.to_owned(),
        Content::File(x) => std::fs::read_to_string(x)?,
    };
    let values = get_values(&template, &args.shell_trust).await?;
    let rendered = replace(&template_str, &values).await?;

    std::io::stdout().write_all(rendered.as_bytes())?;
    Ok(())
}

fn main() -> Result<()> {
    block_on(async_main())
}
