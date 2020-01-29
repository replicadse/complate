extern crate clap;
extern crate dialoguer;
extern crate handlebars;
extern crate serde;
extern crate serde_json;
extern crate serde_yaml;

use std::io::Write;
use futures::executor::{block_on};

pub mod args;
use args::args::ShellMode;
pub mod config;
use config::config::{Config, Template, Content, ValueProvider};

fn select_template<'a>(config: &'a Config) -> std::io::Result<&'a Template> {
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

fn get_values(template: &Template, shell_mode: &ShellMode) -> std::io::Result<std::collections::HashMap<String, String>> {
    let mut values = std::collections::HashMap::new();
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
            ValueProvider::Shell(v) => {
                match shell_mode {
                    ShellMode::Enabled => {
                        let output = std::process::Command::new("sh")
                            .arg("-c")
                            .arg(v)
                            .output()?;
                        if output.status.code().unwrap() != 0 {
                            panic!("failed to run command {}", v);
                        }
                        values.insert(value.0.to_owned(), String::from_utf8(output.stdout).unwrap());
                    },
                    ShellMode::Disabled => {
                        panic!("tried to execute shell command with disabled shell mode for value {}", value.0);
                    },
                }
            },
        }
    }
    Ok(values)
}

fn replace(template: &str, values: &std::collections::HashMap<String, String>) -> std::io::Result<String> {
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

fn main() -> std::io::Result<()> {
    let args = block_on(crate::args::args::ClapArgumentLoader::load_from_cli())?; 
    let cfg: Config = serde_yaml::from_str(&args.configuration).unwrap();

    let template = select_template(&cfg)?;
    let template_str = match &template.content {
        Content::Inline(x) => x.to_owned(),
        Content::File(x) => std::fs::read_to_string(x)?,
    };
    let values = get_values(&template, &args.shell_mode)?;
    let rendered = replace(&template_str, &values)?;

    std::io::stdout().write_all(rendered.as_bytes())?;
    Ok(())
}
