extern crate clap;
// use clap::{App, Arg};

extern crate dialoguer;
extern crate handlebars;
use std::io::Write;
use handlebars::{Handlebars};

extern crate serde;
extern crate serde_json;

fn get_templates() -> std::io::Result<Vec<(String, String)>> {
    let config = std::fs::read_to_string("./res/config.json")?;
    let json: serde_json::Value = serde_json::from_str(config.as_ref())?;
    let mut templates = Vec::<(String, String)>::with_capacity(json.as_object().unwrap().len());
    for t in json["templates"].as_object().unwrap() {
        templates.push((t.0.to_owned(), t.1["file"].as_str().unwrap().to_owned()));
    }
    Ok(templates)
}

fn select_templates(templates: &Vec<(String, String)>) -> std::io::Result<((String, String), String)> {
    let templates_selection: Vec::<String> = templates
        .as_slice()
        .iter()
        .map(|de| format!("{} ({})", de.0, de.1))
        .collect();
    let selection = dialoguer::Select::new()
        .items(templates_selection.as_slice())
        .default(0)
        .paged(false)
        .interact()?;
    let selection_content = std::fs::read_to_string(&templates[selection].1)?;
    Ok((templates[selection].clone(), selection_content))
}

fn main() -> std::io::Result<()> {
    env_logger::init();

    let templates = get_templates()?;
    let selected_template = select_templates(&templates)?;

    let config = std::fs::read_to_string("./res/config.json")?;
    let json: serde_json::Value = serde_json::from_str(config.as_ref())?;
    let json_template: &serde_json::Value = &json["templates"][&(selected_template.0).0];
    let json_prompt: &Vec<serde_json::Value> = json_template["prompt"].as_array().unwrap();
    let mut json_values: serde_json::Value = json_template["values"]["static"].clone();
    
    let prompts_to_values: Vec::<(String, String)> = json_prompt
        .as_slice()
        .iter()
        .map(|k| {
            let result = dialoguer::Input::new()
                .allow_empty(true)
                .with_prompt(k.as_str().unwrap())
                .interact()
                .unwrap();
            (k.as_str().unwrap().to_owned(), result)
        })
        .collect();

    for kv in prompts_to_values {
        json_values[kv.0] = serde_json::to_value(kv.1)?;
    }

    let mut hb = Handlebars::new();
    hb.set_strict_mode(true);
    let commit_msg = hb.render_template(selected_template.1.as_ref(), &json_values).unwrap();

    std::io::stdout().write_all(commit_msg.as_bytes())?;

    Ok(())
}
