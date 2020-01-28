extern crate clap;
use clap::{App, Arg};

extern crate dialoguer;
extern crate handlebars;
use std::io::Write;
use handlebars::{Handlebars};

extern crate serde;
extern crate serde_json;

fn get_templates(config: &str) -> std::io::Result<Vec<(String, String)>> {
    let json: serde_json::Value = serde_json::from_str(config)?;
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

fn config() -> std::io::Result<String> {
    let matches = App::new("complate")
        .version("0.1.0")
        .about("A git commit buddy.")
        .author("Weber, Heiko Alexander <heiko.a.weber@gmail.com>")
        .arg(Arg::with_name("config")
             .short("c")
             .long("config")
             .value_name("FILE")
             .help("The configuration file to use.")
             .default_value("./.complate/config.json")
             .multiple(false)
             .required(false)
             .takes_value(true))
        .get_matches();
    Ok(matches.value_of("config").unwrap().to_owned())
}

fn prompt_and_insert(prompts: &Vec<serde_json::Value>, values: &mut serde_json::Value) -> std::io::Result<()> {
    let prompts_to_values: Vec::<(String, String)> = prompts
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
        values[kv.0] = serde_json::to_value(kv.1)?;
    }

    Ok(())
}

fn replace(template_str: &str, values: &serde_json::Value) -> std::io::Result<String> {
    let mut hb = Handlebars::new();
    hb.set_strict_mode(true);
    Ok(hb.render_template(template_str, values).unwrap())
}

fn main() -> std::io::Result<()> {
    env_logger::init();

    let config = std::fs::read_to_string(config()?)?;

    let templates = get_templates(config.as_ref())?;
    let selected_template = select_templates(&templates)?;

    let json: serde_json::Value = serde_json::from_str(config.as_ref())?;
    let json_template: &serde_json::Value = &json["templates"][&(selected_template.0).0];
    let json_prompt: &Vec<serde_json::Value> = json_template["prompt"].as_array().unwrap();
    let mut json_values: serde_json::Value = json_template["values"]["static"].clone();

    prompt_and_insert(json_prompt, &mut json_values)?;

    let commit_msg = replace(selected_template.1.as_ref(), &json_values)?;
    std::io::stdout().write_all(commit_msg.as_bytes())?;

    Ok(())
}
