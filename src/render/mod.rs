use crate::args::{Backend, ShellTrust};
use crate::config::{Config, Content, Option, OptionValue, Template, VariableDefinition};
use crate::error::Error;
use async_trait::async_trait;
use std::collections::{BTreeMap, HashMap};
use std::env;
use std::result::Result;

#[cfg(feature = "backend+cli")]
mod cli;
#[cfg(feature = "backend+ui")]
mod ui;

pub async fn render(
    args: &crate::args::RenderArguments,
    template: &str,
    values: &BTreeMap<String, String>,
    helpers: &BTreeMap<String, crate::config::Helper>,
) -> Result<String, Box<dyn std::error::Error>> {
    fn recursive_add(namespace: &mut std::collections::VecDeque<String>, parent: &mut serde_json::Value, value: &str) {
        let current_namespace = namespace.pop_front().unwrap();
        match namespace.len() {
            | 0 => {
                parent
                    .as_object_mut()
                    .unwrap()
                    .entry(&current_namespace)
                    .or_insert(serde_json::Value::String(value.to_owned()));
            },
            | _ => {
                let p = parent
                    .as_object_mut()
                    .unwrap()
                    .entry(&current_namespace)
                    .or_insert(serde_json::Value::Object(serde_json::Map::new()));
                recursive_add(namespace, p, value);
            },
        }
    }

    let mut values_json = serde_json::Value::Object(serde_json::Map::new());
    for val in values {
        let namespaces_vec: Vec<String> = val.0.split('.').map(|s| s.to_string()).collect();
        let mut namespaces = std::collections::VecDeque::from(namespaces_vec);
        recursive_add(&mut namespaces, &mut values_json, val.1);
    }

    let mut hb = handlebars::Handlebars::new();
    hb.register_escape_fn(|s| s.to_owned());
    hb.set_strict_mode(!args.loose);

    if helpers.len() > 0 && args.shell_trust != ShellTrust::Ultimate {
        return Err(Box::new(Error::NoTrust(
            "need trust for executing helper functions".into(),
        )));
    }
    for helper in helpers {
        let h_func = move |h: &handlebars::Helper,
                           _: &handlebars::Handlebars,
                           _: &handlebars::Context,
                           _: &mut handlebars::RenderContext,
                           out: &mut dyn handlebars::Output|
              -> handlebars::HelperResult {
            let param = h
                .param(0)
                .ok_or_else(|| Error::Helper("no parameter to helper function found".into()))
                .unwrap();
            // dbg!(param);
            let cmd = helper.1.shell.to_owned();

            let output = std::process::Command::new("sh")
                .arg("-c")
                .arg(cmd)
                .env(
                    "VALUE",
                    param
                        .value()
                        .as_str()
                        .ok_or_else(|| Error::Helper("parameter is not a string".into()))
                        .unwrap(),
                )
                .output()?;
            if output.status.code().unwrap() != 0 {
                return Err(handlebars::RenderError::new("failed to get command status"));
            }

            out.write(String::from_utf8(output.stdout).unwrap().as_str())?;
            Ok(())
        };
        hb.register_helper(helper.0, Box::new(h_func))
    }

    Ok(hb.render_template(template, &values_json)?)
}

#[allow(clippy::needless_lifetimes)]
pub async fn select_template<'a>(
    config: &'a Config,
    backend: &Backend,
    shell_trust: &ShellTrust,
) -> Result<&'a Template, Box<dyn std::error::Error>> {
    let templates = config.templates.keys().cloned().collect::<Vec<String>>();
    let mut template_map = BTreeMap::new();
    for t in templates {
        template_map.insert(
            t.to_owned(),
            Option {
                display: t.to_owned(),
                value: OptionValue::Static(t.to_owned()),
            },
        );
    }

    let be = backend.to_input(shell_trust)?;
    let selection = be.select("", &template_map).await?;

    match config.templates.get(&selection) {
        | Some(x) => Ok(x),
        | None => Err(Box::new(Error::Generic("invalid template selection".into()))),
    }
}

pub async fn populate_variables(
    vars: &std::collections::BTreeMap<String, VariableDefinition>,
    value_overrides: &std::collections::HashMap<String, String>,
    shell_trust: &ShellTrust,
    backend: &Backend,
) -> Result<BTreeMap<String, String>, Box<dyn std::error::Error>> {
    let mut values = BTreeMap::new();
    for var in vars {
        if let Some(v) = value_overrides.get(var.0) {
            values.insert(var.0.to_owned(), v.to_owned());
        } else {
            values.insert(var.0.to_owned(), var.1.execute(shell_trust, backend).await?);
        }
    }
    Ok(values)
}

pub async fn select_and_render(
    invoke_options: crate::args::RenderArguments,
) -> Result<String, Box<dyn std::error::Error>> {
    let cfg: Config = serde_yaml::from_str(&invoke_options.configuration).unwrap();

    let template = match &invoke_options.template {
        | Some(x) => cfg
            .templates
            .get(x)
            .ok_or_else(|| Error::Generic("template not found".into()))?,
        | None => select_template(&cfg, &invoke_options.backend, &invoke_options.shell_trust).await?,
    };
    let template_str = match &template.content {
        | Content::Inline(x) => x.to_owned(),
        | Content::File(x) => std::fs::read_to_string(x)?,
    };
    let values = populate_variables(
        &template.values,
        &invoke_options.value_overrides,
        &invoke_options.shell_trust,
        &invoke_options.backend,
    )
    .await?;
    render(&invoke_options, &template_str, &values, &template.helpers).await
}

#[async_trait]
pub trait Resolve {
    async fn execute(&self, shell_trust: &ShellTrust, backend: &Backend) -> Result<String, Box<dyn std::error::Error>>;
}

#[async_trait]
pub trait UserInput: Send + Sync {
    async fn prompt(&self, text: &str) -> Result<String, Box<dyn std::error::Error>>;
    async fn select(
        &self,
        prompt: &str,
        options: &BTreeMap<String, Option>,
    ) -> Result<String, Box<dyn std::error::Error>>;
    async fn check(
        &self,
        prompt: &str,
        separator: &str,
        options: &BTreeMap<String, Option>,
    ) -> Result<String, Box<dyn std::error::Error>>;
}

impl Backend {
    pub fn to_input<'a>(
        &self,
        shell_trust: &'a ShellTrust,
    ) -> Result<Box<dyn UserInput + 'a>, Box<dyn std::error::Error>> {
        Ok(match self {
            #[cfg(feature = "backend+cli")]
            | Backend::CLI => Box::new(cli::CLIBackend::new(shell_trust)) as Box<dyn UserInput>,
            #[cfg(feature = "backend+ui")]
            | Backend::UI => Box::new(ui::UIBackend::new(shell_trust)) as Box<dyn UserInput>,
        })
    }
}

#[async_trait]
impl Resolve for VariableDefinition {
    async fn execute(&self, shell_trust: &ShellTrust, backend: &Backend) -> Result<String, Box<dyn std::error::Error>> {
        let backend_impl = backend.to_input(shell_trust)?;

        match self {
            | VariableDefinition::Env(v) => Ok(env::var(v)?),
            | VariableDefinition::Static(v) => Ok(v.to_owned()),
            | VariableDefinition::Prompt(v) => backend_impl.prompt(v).await,
            | VariableDefinition::Shell(cmd) => shell(cmd, &HashMap::new(), shell_trust).await,
            | VariableDefinition::Select { text, options } => backend_impl.select(text, options).await,
            | VariableDefinition::Check {
                text,
                separator,
                options,
            } => backend_impl.check(text, separator, options).await,
        }
    }
}

async fn shell(
    command: &str,
    env: &HashMap<String, String>,
    shell_trust: &ShellTrust,
) -> Result<String, Box<dyn std::error::Error>> {
    match shell_trust {
        | ShellTrust::None => {
            return Err(Box::new(Error::NoTrust(
                "need trust for executing shell commands".into(),
            )))
        },
        | ShellTrust::Ultimate => {},
    }

    let output = std::process::Command::new("sh")
        .arg("-c")
        .arg(command)
        .envs(env)
        .output()?;
    if output.status.code().unwrap() != 0 {
        return Err(Box::new(Error::ShellCommand(command.into())));
    }
    Ok(String::from_utf8(output.stdout)?)
}
