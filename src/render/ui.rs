use crate::error::Error;

use super::UserInput;
use async_trait::async_trait;
use cursive::traits::*;
use cursive::views::Dialog;
use std::collections::{HashMap, HashSet};
use std::ops::Deref;
use std::result::Result;

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
    async fn prompt(&self, text: &str) -> Result<String, Box<dyn std::error::Error>> {
        let v = std::rc::Rc::new(std::cell::Cell::new(None));
        let vx = v.clone();
        let key = std::rc::Rc::new(text.to_owned());

        let mut siv = cursive::default();
        let form = fui::form::FormView::new()
            .field(fui::fields::Text::new(text))
            .on_submit(move |s, x| {
                vx.set(Some(x.get(key.deref()).unwrap().as_str().unwrap().to_owned()));
                s.quit()
            });

        siv.add_layer(Dialog::around(form).full_screen());
        siv.run();

        match v.take() {
            | Some(x) => Ok(x),
            | None => Err(Box::new(Error::InteractAbort)),
        }
    }

    async fn select(
        &self,
        prompt: &str,
        options: &std::collections::BTreeMap<String, super::Option>,
    ) -> Result<String, Box<dyn std::error::Error>> {
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
            siv.add_layer(cursive::views::Dialog::around(select.scrollable().fixed_size((20, 10))).title(prompt));
            siv.run();
            sel.set(v.take().unwrap());
        }

        let sel_value = sel.take();
        let selection = &options[&keys[display_vals.iter().position(|x| *x == sel_value).unwrap()]];
        match &selection.value {
            | super::OptionValue::Static(x) => Ok(x.to_owned()),
            | super::OptionValue::Shell(cmd) => super::shell(cmd, &HashMap::new(), &self.shell_trust).await,
        }
    }

    async fn check(
        &self,
        _: &str,
        separator: &str,
        options: &std::collections::BTreeMap<String, super::Option>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let mut opts = Vec::new();
        {
            let ok_pressed = std::sync::Arc::new(std::cell::Cell::new(false));
            let ok_pressed_siv = ok_pressed.clone();
            let items = std::sync::Arc::new(std::sync::RwLock::new(HashSet::<std::string::String>::new()));
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
                return Err(Box::new(Error::InteractAbort));
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
                | super::OptionValue::Static(x) => x.to_owned(),
                | super::OptionValue::Shell(cmd) => {
                    super::shell(&cmd, &HashMap::new(), &self.shell_trust).await.unwrap()
                },
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
        Self { opts: Vec::from(opts) }
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
