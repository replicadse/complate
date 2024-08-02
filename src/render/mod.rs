use {
    crate::config::{
        Config,
        Content,
        OptionValue,
        Template,
        VariableDefinition,
    },
    anyhow::Result,
    async_trait::async_trait,
    fancy_regex::Regex,
    handlebars::RenderError,
    std::{
        collections::{
            BTreeMap,
            HashMap,
        },
        env,
    },
};

#[cfg(feature = "backend+cli")]
pub mod cli;
pub mod headless;

#[derive(Debug)]
pub enum Backend {
    Headless,
    #[cfg(feature = "backend+cli")]
    CLI,
}

#[derive(Debug)]
pub struct RenderArguments {
    pub configuration: String,
    pub template: Option<String>,
    pub value_overrides: HashMap<String, String>,
    pub shell_trust: ShellTrust,
    pub loose: bool,
    pub backend: Backend,
}

#[derive(Debug, Eq, PartialEq)]
pub enum ShellTrust {
    None,
    Ultimate,
}

pub async fn make_handlebars<'a>(
    variable_values: &HashMap<String, String>,
    helpers: &'a std::option::Option<HashMap<String, String>>,
    shell_trust: &ShellTrust,
    strict: bool,
) -> Result<(handlebars::Handlebars<'a>, serde_json::Value)> {
    fn recursive_add(namespace: &mut std::collections::VecDeque<String>, parent: &mut serde_json::Value, value: &str) {
        let current_namespace = namespace.pop_front().unwrap();
        match namespace.len() {
            | 0 => {
                parent
                    .as_object_mut()
                    .unwrap()
                    .entry(&current_namespace)
                    .or_insert(serde_json::Value::String(value.into()));
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
    for val in variable_values {
        let namespaces_vec: Vec<String> = val.0.split('.').map(|s| s.to_string()).collect();
        let mut namespaces = std::collections::VecDeque::from(namespaces_vec);
        recursive_add(&mut namespaces, &mut values_json, val.1);
    }

    let mut hb = handlebars::Handlebars::new();
    hb.register_escape_fn(|s| s.into());
    hb.set_strict_mode(strict);

    if let Some(helpers) = helpers {
        if helpers.len() > 0 && shell_trust != &ShellTrust::Ultimate {
            return Err(anyhow::anyhow!("need trust for executing helper functions").into());
        }

        for helper in helpers {
            let h_func = move |h: &handlebars::Helper,
                               _: &handlebars::Handlebars,
                               _: &handlebars::Context,
                               _: &mut handlebars::RenderContext,
                               out: &mut dyn handlebars::Output|
                  -> handlebars::HelperResult {
                let param = h.param(0).ok_or(RenderError::new("parameter is not a string"))?;
                // dbg!(param);
                let cmd = helper.1;

                let output = std::process::Command::new("sh")
                    .arg("-c")
                    .arg(cmd)
                    .env(
                        "VALUE",
                        param
                            .value()
                            .as_str()
                            .ok_or(RenderError::new("parameter is not a string"))?,
                    )
                    .output()?;
                if output.status.code().unwrap() != 0 {
                    return Err(handlebars::RenderError::new("failed to get command status"));
                }

                out.write(String::from_utf8(output.stdout)?.as_str())?;
                Ok(())
            };
            hb.register_helper(helper.0, Box::new(h_func))
        }
    }

    Ok((hb, values_json))
}

pub async fn select_template<'a>(
    config: &'a Config,
    backend: &Backend,
    shell_trust: &ShellTrust,
) -> Result<&'a Template> {
    let templates = config.templates.keys().cloned().collect::<Vec<String>>();
    let mut template_map = BTreeMap::new();
    for t in templates {
        template_map.insert(t.to_owned(), crate::config::Option {
            display: t.to_owned(),
            value: OptionValue::Static(t.into()),
        });
    }

    let be = backend.to_input(shell_trust)?;
    let selection = be.select("", &template_map).await?;

    match config.templates.get(&selection) {
        | Some(x) => Ok(x),
        | None => Err(anyhow::anyhow!("invalid template selection")),
    }
}

pub async fn populate_variables(
    vars: &std::collections::HashMap<String, VariableDefinition>,
    value_overrides: &std::collections::HashMap<String, String>,
    shell_trust: &ShellTrust,
    backend: &Backend,
    prefix: Option<String>,
) -> Result<HashMap<String, String>> {
    let mut values = HashMap::<String, String>::new();
    for v_override in value_overrides {
        values.insert(v_override.0.into(), v_override.1.into());
    }

    for var in vars {
        if None == values.get(var.0) {
            values.insert(var.0.into(), var.1.execute(shell_trust, backend).await?);
        }
    }

    let values = values
        .iter()
        .map(|(k, v)| {
            let mut key = k.clone();
            if let Some(p) = &prefix {
                key = format!("{}.{}", p, key);
            }
            (key, v.clone())
        })
        .collect::<HashMap<String, String>>();
    Ok(values)
}

pub async fn render_template(
    template: &Template,
    value_overrides: &HashMap<String, String>,
    shell_trust: &ShellTrust,
    backend: &Backend,
    strict: bool,
) -> Result<String> {
    let template_str = match &template.content {
        | Content::Inline(x) => x.into(),
        | Content::File(x) => std::fs::read_to_string(x)?,
    };

    let values = if let Some(variables) = &template.variables {
        populate_variables(variables, value_overrides, shell_trust, backend, None).await?
    } else {
        HashMap::<_, _>::new()
    };

    let hb = make_handlebars(&values, &template.helpers, shell_trust, strict).await?;
    hb.0.render_template(&template_str, &hb.1)
        .map_err(|e| anyhow::anyhow!(e))
}

pub async fn select_and_render(invoke_options: RenderArguments) -> Result<String> {
    #[derive(serde::Deserialize)]
    struct WithVersion {
        version: String,
    }
    let version_check: WithVersion = serde_yaml::from_str(&invoke_options.configuration)
        .or::<anyhow::Error>(Err(anyhow::anyhow!("config missing version field")))?;

    let version_regex = Regex::new("^([0-9]+)\\.([0-9]+)$")?;
    if !version_regex.is_match(&version_check.version)? {
        return Err(anyhow::anyhow!("invalid version: {}", version_check.version));
    }
    let expected_version = env!("CARGO_PKG_VERSION").split(".").collect::<Vec<_>>()[..2].join(".");
    if env!("CARGO_PKG_VERSION") != "0.0.0" {
        if &version_check.version != &expected_version {
            return Err(anyhow::anyhow!("config file version mismatch to binary"));
        }
    }

    let cfg: Config = serde_yaml::from_str(&invoke_options.configuration)?;
    let template = match &invoke_options.template {
        | Some(x) => {
            cfg.templates
                .get(x)
                .ok_or_else(|| anyhow::anyhow!("template not found"))?
        },
        | None => select_template(&cfg, &invoke_options.backend, &invoke_options.shell_trust).await?,
    };

    render_template(
        template,
        &invoke_options.value_overrides,
        &invoke_options.shell_trust,
        &invoke_options.backend,
        !invoke_options.loose,
    )
    .await
}

#[async_trait]
pub trait Resolve {
    async fn execute(&self, shell_trust: &ShellTrust, backend: &Backend) -> Result<String>;
}

#[async_trait]
pub trait UserInput: Send+Sync {
    async fn prompt(&self, text: &str) -> Result<String>;
    async fn select(&self, prompt: &str, options: &BTreeMap<String, crate::config::Option>) -> Result<String>;
    async fn check(
        &self,
        prompt: &str,
        separator: &str,
        options: &BTreeMap<String, crate::config::Option>,
    ) -> Result<String>;
}

impl Backend {
    pub fn to_input<'a>(&self, shell_trust: &'a ShellTrust) -> Result<Box<dyn UserInput+'a>> {
        Ok(match self {
            | Backend::Headless => Box::new(headless::HeadlessBackend::new()) as Box<dyn UserInput>,
            #[cfg(feature = "backend+cli")]
            | Backend::CLI => Box::new(cli::CLIBackend::new(shell_trust)) as Box<dyn UserInput>,
        })
    }
}

#[async_trait]
impl Resolve for VariableDefinition {
    async fn execute(&self, shell_trust: &ShellTrust, backend: &Backend) -> Result<String> {
        let backend_impl = backend.to_input(shell_trust)?;

        match self {
            | VariableDefinition::Arg => Err(anyhow::anyhow!("variable missing")),
            | VariableDefinition::Env(v) => Ok(env::var(v)?),
            | VariableDefinition::Static(v) => Ok(v.into()),
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

async fn shell(command: &str, env: &HashMap<String, String>, shell_trust: &ShellTrust) -> Result<String> {
    match shell_trust {
        | ShellTrust::None => return Err(anyhow::anyhow!("need trust for executing shell commands")),
        | ShellTrust::Ultimate => {},
    }

    let output = std::process::Command::new("sh")
        .arg("-c")
        .arg(command)
        .envs(env)
        .output()?;
    if output.status.code().unwrap() != 0 {
        return Err(anyhow::anyhow!("shell command error:\n{}", command));
    }
    Ok(String::from_utf8(output.stdout)?)
}
