use crate::args::{Backend, ShellTrust};
use crate::config::{Config, Content, Option, OptionValue, Template, VariableDefinition};
use async_trait::async_trait;
use std::collections::BTreeMap;
use std::io::{Error, ErrorKind, Result};

pub async fn render(template: &str, values: &BTreeMap<String, String>) -> Result<String> {
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

#[allow(clippy::needless_lifetimes)]
pub async fn select_template<'a>(
    config: &'a Config,
    backend: &Backend,
    shell_trust: &ShellTrust,
) -> Result<&'a Template> {
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
        Some(x) => Ok(x),
        None => Err(std::io::Error::new(std::io::ErrorKind::Other, "failed")),
    }
}

pub async fn populate_variables(
    vars: &std::collections::BTreeMap<String, VariableDefinition>,
    shell_trust: &ShellTrust,
    backend: &Backend,
) -> Result<BTreeMap<String, String>> {
    let mut values = BTreeMap::new();
    for var in vars {
        values.insert(var.0.to_owned(), var.1.execute(shell_trust, backend).await?);
    }
    Ok(values)
}

pub async fn select_and_render(invoke_options: crate::args::RenderArguments) -> Result<String> {
    let cfg: Config = serde_yaml::from_str(&invoke_options.configuration).unwrap();

    let template = match invoke_options.template {
        Some(x) => cfg.templates.get(&x).unwrap(),
        None => select_template(&cfg, &invoke_options.backend, &invoke_options.shell_trust).await?,
    };
    let template_str = match &template.content {
        Content::Inline(x) => x.to_owned(),
        Content::File(x) => std::fs::read_to_string(x)?,
    };
    let values = populate_variables(
        &template.values,
        &invoke_options.shell_trust,
        &invoke_options.backend,
    )
    .await?;
    render(&template_str, &values).await
}

#[async_trait]
pub trait Resolve {
    async fn execute(&self, shell_trust: &ShellTrust, backend: &Backend) -> Result<String>;
}

#[async_trait]
pub trait UserInput: Send + Sync {
    async fn prompt(&self, text: &str) -> Result<String>;
    async fn shell(&self, command: &str, shell_trust: &ShellTrust) -> Result<String>;
    async fn select(&self, prompt: &str, options: &BTreeMap<String, Option>) -> Result<String>;
    async fn check(
        &self,
        prompt: &str,
        separator: &str,
        options: &BTreeMap<String, Option>,
    ) -> Result<String>;
}

impl Backend {
    pub fn to_input<'a>(&self, shell_trust: &'a ShellTrust) -> Result<Box<dyn UserInput + 'a>> {
        Ok(match self {
            #[cfg(feature = "backend+cli")]
            Backend::CLI => Box::new(cli::CLIBackend::new(shell_trust)) as Box<dyn UserInput>,
            #[cfg(feature = "backend+ui")]
            Backend::UI => Box::new(ui::UIBackend::new(shell_trust)) as Box<dyn UserInput>,
        })
    }
}

#[async_trait]
impl Resolve for VariableDefinition {
    async fn execute(&self, shell_trust: &ShellTrust, backend: &Backend) -> Result<String> {
        let backend_impl = backend.to_input(shell_trust)?;

        match self {
            VariableDefinition::Static(v) => Ok(v.to_owned()),
            VariableDefinition::Prompt(v) => backend_impl.prompt(v).await,
            VariableDefinition::Shell(cmd) => backend_impl.shell(cmd, shell_trust).await,
            VariableDefinition::Select { text, options } => {
                backend_impl.select(text, options).await
            }
            VariableDefinition::Check {
                text,
                separator,
                options,
            } => backend_impl.check(text, separator, options).await,
        }
    }
}

async fn shell(command: &str, shell_trust: &ShellTrust, backend: &Backend) -> Result<String> {
    match shell_trust {
        ShellTrust::None => return Err(Error::new(ErrorKind::Other, "no shell trust")),
        ShellTrust::Prompt => {
            let be = backend.to_input(shell_trust)?;
            let mut yesno = BTreeMap::new();
            yesno.insert(
                "0".to_owned(),
                Option {
                    display: "yes".to_owned(),
                    value: OptionValue::Static("yes".to_owned()),
                },
            );
            yesno.insert(
                "1".to_owned(),
                Option {
                    display: "no".to_owned(),
                    value: OptionValue::Static("no".to_owned()),
                },
            );

            let sel = be.select(&format!("You are about to run a shell command. The command is:\n{}\nDo you confirm the execution?", command), &yesno).await;
            if sel.unwrap_or_default() == "yes" {
            } else {
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

#[cfg(feature = "backend+cli")]
mod cli {
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
}

#[cfg(feature = "backend+ui")]
mod ui {
    use super::UserInput;
    use async_trait::async_trait;
    use cursive::traits::*;
    use cursive::views::Dialog;
    use std::collections::HashSet;
    use std::io::{Error, ErrorKind, Result};
    use std::ops::Deref;

    pub struct UIBackend<'a> {
        shell_trust: &'a super::ShellTrust,
    }

    impl<'a> UIBackend<'a> {
        pub fn new(shell_trust: &'a super::ShellTrust) -> Self {
            Self { shell_trust }
        }
    }

    #[async_trait]
    impl<'a> UserInput for UIBackend<'a> {
        async fn prompt(&self, text: &str) -> Result<String> {
            let v = std::rc::Rc::new(std::cell::Cell::new(None));
            let vx = v.clone();

            let mut siv = cursive::default();
            let form = fui::form::FormView::new()
                .field(fui::fields::Text::new(text))
                .on_submit(move |s, x| {
                    vx.set(Some(x.to_string()));
                    s.quit()
                });

            siv.add_layer(Dialog::around(form).full_screen());
            siv.run();

            v.take()
                .ok_or_else(|| Error::new(ErrorKind::Other, "user abort"))
        }

        async fn shell(&self, command: &str, shell_trust: &super::ShellTrust) -> Result<String> {
            super::shell(command, shell_trust, &super::Backend::UI).await
        }

        async fn select(
            &self,
            prompt: &str,
            options: &std::collections::BTreeMap<String, super::Option>,
        ) -> Result<String> {
            let keys = options.keys().cloned().collect::<Vec<String>>();
            let mut index_display = 0usize;
            let display_vals = options
                .values()
                .map(|x| {
                    let mut v = String::from("(");
                    v.push_str(&index_display.to_string());
                    index_display += 1;
                    v.push_str(") ");
                    v.push_str(&x.display.to_owned());
                    v
                })
                .collect::<Vec<String>>();

            let sel = std::cell::Cell::new(String::new());
            {
                let v = std::rc::Rc::new(std::cell::Cell::new(None));
                let vx = v.clone();
                let mut siv = cursive::default();
                siv.add_global_callback(cursive::event::Event::CtrlChar('c'), |s| {
                    s.quit();
                });
                let mut select = cursive::views::SelectView::<String>::new()
                    .h_align(cursive::align::HAlign::Left)
                    .autojump()
                    .on_submit(move |s, x: &str| {
                        vx.set(Some(x.to_owned()));
                        s.quit();
                    });
                select.add_all_str(&display_vals);
                siv.add_layer(
                    cursive::views::Dialog::around(select.scrollable().fixed_size((20, 10)))
                        .title(prompt),
                );
                siv.run();
                sel.set(v.take().unwrap());
            }

            let sel_value = sel.take();
            let selection =
                &options[&keys[display_vals.iter().position(|x| *x == sel_value).unwrap()]];
            match &selection.value {
                super::OptionValue::Static(x) => Ok(x.to_owned()),
                super::OptionValue::Shell(cmd) => self.shell(cmd, &self.shell_trust).await,
            }
        }

        async fn check(
            &self,
            _: &str,
            separator: &str,
            options: &std::collections::BTreeMap<String, super::Option>,
        ) -> Result<String> {
            let mut opts = Vec::new();
            {
                let ok_pressed = std::sync::Arc::new(std::cell::Cell::new(false));
                let ok_pressed_siv = ok_pressed.clone();
                let items = std::sync::Arc::new(std::sync::RwLock::new(HashSet::<
                    std::string::String,
                >::new()));
                let items_view = items.clone();
                let items_view2 = items.clone();

                let keys = options.keys().cloned().collect::<Vec<String>>();
                let mut index_display = 0usize;
                let display_vals = options
                    .values()
                    .map(|x| {
                        let mut v = String::from("(");
                        v.push_str(&index_display.to_string());
                        index_display += 1;
                        v.push_str(") ");
                        v.push_str(&x.display.to_owned());
                        v
                    })
                    .collect::<Vec<String>>();

                let mut siv = cursive::default();
                siv.add_global_callback(cursive::event::Event::CtrlChar('c'), |s| {
                    s.quit();
                });

                let view = fui::views::Multiselect::new(ArrOptions::new(&display_vals))
                    .on_select(move |_, v| {
                        items_view.try_write().unwrap().insert(v.deref().to_owned());
                    })
                    .on_deselect(move |_, v| {
                        items_view2.try_write().unwrap().remove(v.deref());
                    });
                let dlg = cursive::views::Dialog::around(view).button("Ok", move |s| {
                    ok_pressed_siv.set(true);
                    s.quit();
                });
                siv.add_layer(dlg);
                siv.run();

                if !ok_pressed.take() {
                    return Err(Error::new(ErrorKind::Other, "user abort"));
                }
                for x in items.try_read().unwrap().iter() {
                    let pos = display_vals.iter().position(|v| x == v).unwrap();
                    let selection = &options[&keys[pos]];
                    opts.push(&selection.value);
                }
            }
            let mut data = Vec::new();
            for opt in opts {
                data.push(match opt {
                    super::OptionValue::Static(x) => x.to_owned(),
                    super::OptionValue::Shell(cmd) => {
                        self.shell(&cmd, &self.shell_trust).await.unwrap()
                    }
                });
            }
            Ok(data.join(separator))
        }
    }

    struct ArrOptions {
        opts: Vec<String>,
    }
    impl ArrOptions {
        pub fn new(opts: &[String]) -> Self {
            Self {
                opts: Vec::from(opts),
            }
        }
    }
    impl fui::feeders::Feeder for ArrOptions {
        fn query(&self, _: &str, position: usize, items_count: usize) -> Vec<String> {
            self.opts
                .iter()
                .skip(position)
                .take(items_count)
                .map(|x| x.to_owned())
                .collect()
        }
    }
}
