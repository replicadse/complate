#[cfg(not(feature = "backend::cli"))]
#[cfg(not(feature = "backend::ui"))]
extern crate DO_NOT_COMPILE_WITHOUT_ANY_ENABLED_BACKEND;

extern crate clap;
#[cfg(feature = "backend::ui")]
extern crate cursive;
#[cfg(feature = "backend::cli")]
extern crate dialoguer;
#[cfg(feature = "backend::ui")]
extern crate fui;
extern crate handlebars;
extern crate serde;
extern crate serde_json;
extern crate serde_yaml;

use futures::executor::block_on;
use std::collections::BTreeMap;
use std::io::{Result, Write};

pub mod args;
use args::{Backend, ShellTrust};
pub mod config;
use config::{Config, Content, Template};
pub mod print;
use print::Print;

#[allow(clippy::needless_lifetimes)]
async fn select_template<'a>(config: &'a Config, backend: &Backend) -> Result<&'a Template> {
    let keys: Vec<String> = config.templates.keys().map(|t| t.to_owned()).collect();

    let be = backend.to_input()?;
    let selection = be.select("", keys.as_slice()).await?;

    match config.templates.get(&selection) {
        Some(x) => Ok(x),
        None => Err(std::io::Error::new(std::io::ErrorKind::Other, "failed")),
    }
}

async fn get_values(
    template: &Template,
    shell_trust: &ShellTrust,
    backend: &Backend,
) -> Result<BTreeMap<String, String>> {
    let mut values = BTreeMap::new();
    for value in &template.values {
        values.insert(
            value.0.to_owned(),
            value.1.execute(shell_trust, backend).await?,
        );
    }
    Ok(values)
}

async fn replace(template: &str, values: &BTreeMap<String, String>) -> Result<String> {
    fn recursive_add(
        namespace: &mut std::collections::VecDeque<String>,
        parent: &mut serde_json::Value,
        value: &str,
    ) {
        let current_namespace = namespace.pop_front().unwrap();
        match namespace.len() {
            0 => {
                parent
                    .as_object_mut()
                    .unwrap()
                    .entry(&current_namespace)
                    .or_insert(serde_json::Value::String(value.to_owned()));
            }
            _ => {
                let p = parent
                    .as_object_mut()
                    .unwrap()
                    .entry(&current_namespace)
                    .or_insert(serde_json::Value::Object(serde_json::Map::new()));
                recursive_add(namespace, p, value);
            }
        }
    }

    let mut hb = handlebars::Handlebars::new();
    hb.set_strict_mode(true);

    let mut values_json = serde_json::Value::Object(serde_json::Map::new());
    for val in values {
        let namespaces_vec: Vec<String> = val.0.split('.').map(|s| s.to_string()).collect();
        let mut namespaces = std::collections::VecDeque::from(namespaces_vec);
        recursive_add(&mut namespaces, &mut values_json, val.1);
    }

    let rendered_template = hb.render_template(template, &values_json).unwrap();
    Ok(rendered_template)
}

async fn print(invoke_options: args::PrintArguments) -> Result<()> {
    let cfg: Config = serde_yaml::from_str(&invoke_options.configuration).unwrap();

    let template = select_template(&cfg, &invoke_options.backend).await?;
    let template_str = match &template.content {
        Content::Inline(x) => x.to_owned(),
        Content::File(x) => std::fs::read_to_string(x)?,
    };
    let values = get_values(
        &template,
        &invoke_options.shell_trust,
        &invoke_options.backend,
    )
    .await?;
    let rendered = replace(&template_str, &values).await?;

    std::io::stdout().write_all(rendered.as_bytes())?;
    Ok(())
}

async fn default_config() -> String {
    r###"version: 0.5
templates:
    default:
        content:
            inline: |-
                {{ a.summary }} | {{ e.version }}
                Components: [{{ f.components }}]
                Author: {{ b.author.name }} | {{ c.author.account }}
                
                Files:
                {{ d.git.staged.files }}
        values:
            a.summary:
                prompt: "Enter the summary"
            b.author.name:
                shell: "git config user.name | tr -d '\n'"
            c.author.account:
                shell: "whoami | tr -d '\n'"
            d.git.staged.files:
                shell: "git diff --name-status --cached"
            e.version:
                select:
                    text: Select the version level that shall be incremented
                    options:
                        - "#patch"
                        - "#minor"
                        - "#major"
            f.components:
                check:
                    text: Select the components that are affected
                    options:
                        - security
                        - command::print
                        - backend::cli
                        - backend::ui
                        - misc
    
"###
    .to_owned()
}

async fn async_main() -> Result<()> {
    let cmd = crate::args::ClapArgumentLoader::load_from_cli().await?;
    cmd.validate().await?;

    match cmd.command {
        crate::args::Command::Init => {
            std::fs::create_dir_all("./.complate")?;
            std::fs::write("./.complate/config.yml", default_config().await)?;
            Ok(())
        }
        crate::args::Command::Print(x) => print(x).await,
    }
}

fn main() -> Result<()> {
    block_on(async_main())
}
