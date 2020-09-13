use crate::args::{Backend, ShellTrust};
use crate::config::ValueDefinition;
use async_trait::async_trait;
use std::io::{Error, ErrorKind, Result};

#[async_trait]
pub trait Print {
    async fn execute(&self, shell_trust: &ShellTrust, backend: &Backend) -> Result<String>;
}

#[async_trait]
pub trait UserInput: Send + Sync {
    async fn prompt(&self, text: &str) -> Result<String>;
    async fn shell(&self, command: &str, shell_trust: &ShellTrust) -> Result<String>;
    async fn select(&self, prompt: &str, options: &[String]) -> Result<String>;
    async fn check(&self, prompt: &str, options: &[String]) -> Result<String>;
}

impl Backend {
    pub fn to_input(&self) -> Result<Box<dyn UserInput>> {
        Ok(match self {
            #[cfg(feature = "backend::cli")]
            Backend::CLI => Box::new(cli::CLIBackend::new()) as Box<dyn UserInput>,
            #[cfg(feature = "backend::ui")]
            Backend::UI => Box::new(ui::UIBackend::new()) as Box<dyn UserInput>,
        })
    }
}

#[async_trait]
impl Print for ValueDefinition {
    async fn execute(&self, shell_trust: &ShellTrust, backend: &Backend) -> Result<String> {
        let backend_impl = backend.to_input()?;

        match self {
            ValueDefinition::Static(v) => Ok(v.to_owned()),
            ValueDefinition::Prompt(v) => backend_impl.prompt(v).await,
            ValueDefinition::Shell(cmd) => backend_impl.shell(cmd, shell_trust).await,
            ValueDefinition::Select { text, options } => backend_impl.select(text, options).await,
            ValueDefinition::Check { text, options } => backend_impl.check(text, options).await,
        }
    }
}

async fn shell(command: &str, shell_trust: &ShellTrust, backend: &Backend) -> Result<String> {
    match shell_trust {
        ShellTrust::None => return Err(Error::new(ErrorKind::Other, "no shell trust")),
        ShellTrust::Prompt => {
            let be = backend.to_input()?;
            let sel = be.select(&format!("You are about to run a shell command. The command is:\n{}\nDo you confirm the execution?", command), &["yes".to_owned(), "no".to_owned()]).await;
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

#[cfg(feature = "backend::cli")]
mod cli {
    use super::UserInput;
    use async_trait::async_trait;
    use std::io::Result;

    pub struct CLIBackend {}

    impl CLIBackend {
        pub fn new() -> Self {
            Self {}
        }
    }

    #[async_trait]
    impl UserInput for CLIBackend {
        async fn prompt(&self, text: &str) -> Result<String> {
            dialoguer::Input::new()
                .allow_empty(true)
                .with_prompt(text)
                .interact()
        }

        async fn shell(&self, command: &str, shell_trust: &super::ShellTrust) -> Result<String> {
            super::shell(command, shell_trust, &super::Backend::CLI).await
        }

        async fn select(&self, prompt: &str, options: &[String]) -> Result<String> {
            let idx = dialoguer::Select::new()
                .with_prompt(prompt)
                .items(options)
                .default(0)
                .paged(false)
                .interact()?;
            Ok(options[idx].to_owned())
        }

        async fn check(&self, prompt: &str, options: &[String]) -> Result<String> {
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
    }
}

#[cfg(feature = "backend::ui")]
mod ui {
    use super::UserInput;
    use async_trait::async_trait;
    use cursive::traits::*;
    use fui::views::Multiselect;
    use std::collections::HashSet;
    use std::io::{Error, ErrorKind, Result};

    pub struct UIBackend {}

    impl UIBackend {
        pub fn new() -> Self {
            Self {}
        }
    }

    #[async_trait]
    impl UserInput for UIBackend {
        async fn prompt(&self, text: &str) -> Result<String> {
            let mut siv = cursive::Cursive::default();
            siv.add_global_callback(cursive::event::Event::CtrlChar('c'), |s| {
                s.quit();
            });
            let v = std::rc::Rc::new(std::cell::Cell::new(None));
            let vx = v.clone();

            siv.add_layer(
                cursive::views::Dialog::new().title(text).content(
                    cursive::views::EditView::new()
                        .on_submit(move |s, x| {
                            vx.set(Some(x.to_string()));
                            s.quit();
                        })
                        .fixed_width(40),
                ),
            );
            siv.run();

            v.take()
                .ok_or_else(|| Error::new(ErrorKind::Other, "user abort"))
        }

        async fn shell(&self, command: &str, shell_trust: &super::ShellTrust) -> Result<String> {
            super::shell(command, shell_trust, &super::Backend::UI).await
        }

        async fn select(&self, prompt: &str, options: &[String]) -> Result<String> {
            let mut siv = cursive::Cursive::default();
            siv.add_global_callback(cursive::event::Event::CtrlChar('c'), |s| {
                s.quit();
            });
            let v = std::rc::Rc::new(std::cell::Cell::new(None));
            let vx = v.clone();

            let mut select = cursive::views::SelectView::<String>::new()
                .h_align(cursive::align::HAlign::Left)
                .autojump()
                .on_submit(move |s, x: &str| {
                    vx.set(Some(x.to_owned()));
                    s.quit();
                });
            select.add_all_str(options);

            siv.add_layer(
                cursive::views::Dialog::around(select.scrollable().fixed_size((20, 10)))
                    .title(prompt),
            );

            siv.run();
            v.take()
                .ok_or_else(|| Error::new(ErrorKind::Other, "user abort"))
        }

        async fn check(&self, _: &str, options: &[String]) -> Result<String> {
            let mut siv = cursive::Cursive::default();
            let ok_pressed = std::sync::Arc::new(std::cell::Cell::new(false));
            let ok_pressed_siv = ok_pressed.clone();
            let items = std::sync::Arc::new(std::sync::RwLock::new(HashSet::new()));
            let items_view = items.clone();
            let items_view2 = items.clone();

            let view = Multiselect::new(ArrOptions::new(options))
                .on_select(move |_, v| {
                    items_view.try_write().unwrap().insert(v);
                })
                .on_deselect(move |_, v| {
                    items_view2.try_write().unwrap().remove(&v);
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
            let it = items.try_read().unwrap();
            Ok(it
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<String>>()
                .join(", "))
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
