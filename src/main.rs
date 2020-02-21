extern crate clap;
extern crate dialoguer;
extern crate handlebars;
extern crate serde;
extern crate serde_json;
extern crate serde_yaml;

use std::io::{Result, Write};
use std::collections::{BTreeMap};
use futures::executor::{block_on};

pub mod args;
use args::args::ShellTrust;
pub mod config;
use config::config::{Config, Template, Content};
pub mod execute;
use execute::execute::{Execute};

async fn select_template<'a>(config: &'a Config) -> Result<&'a Template> {
    let keys: Vec<String> = config.templates.keys().map(
        |t| t.to_owned()
    ).collect();
    let selection = dialoguer::Select::new()
        .items(keys.as_slice())
        .default(0)
        .paged(false)
        .interact()?;
    
    match config.templates.get(&keys[selection]) {
        Some(x) => Ok(x),
        None => Err(std::io::Error::new(std::io::ErrorKind::Other, "failed")),
    }
}

async fn get_values(template: &Template, shell_trust: &ShellTrust) -> Result<BTreeMap<String, String>> {
    let mut values = BTreeMap::new();
    for value in &template.values {
        values.insert(value.0.to_owned(), value.1.execute(shell_trust).await?);
    }
    Ok(values)
}

async fn replace(template: &str, values: &BTreeMap<String, String>) -> Result<String> {
    fn recursive_add(namespace: &mut std::collections::VecDeque<String>, parent: &mut serde_json::Value, value: &str) {
        let current_namespace = namespace.pop_front().unwrap();
        match namespace.len() {
            0 => {
                parent.as_object_mut().unwrap().entry(&current_namespace)
                    .or_insert(serde_json::Value::String(value.to_owned()));
            }
            _ => {
                let p = parent.as_object_mut().unwrap().entry(&current_namespace)
                    .or_insert(serde_json::Value::Object(serde_json::Map::new()));
                recursive_add(
                    namespace, p, value);
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
