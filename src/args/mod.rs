use std::{result::Result, collections::HashMap};

use clap::{Arg, ArgAction};

#[derive(Debug)]
pub struct CallArgs {
    pub privileges: Privilege,
    pub command: Command,
}

impl CallArgs {
    #[allow(clippy::single_match)]
    pub async fn validate(&self) -> Result<(), Box<dyn std::error::Error>> {
        match &self.command {
            Command::Render(args) => {
                if args.helpers && args.shell_trust != ShellTrust::Ultimate {
                    return Err(Box::new(crate::error::NoShellTrust::new(
                        "need ultimate shell trust for helper functions",
                    )));
                }
            }
            _ => {}
        }

        match self.privileges {
            Privilege::Normal => match &self.command {
                Command::Render(args) => {
                    match args.backend {
                        #[cfg(feature = "backend+ui")]
                        Backend::UI => {
                            return Err(Box::new(crate::error::NeedExperimentalFlag::new(
                                "to enable UI backend",
                            )))
                        }
                        #[cfg(feature = "backend+cli")]
                        Backend::CLI => {}
                    };
                    if args.value_overrides.len() > 0 {
                        return Err(Box::new(crate::error::NeedExperimentalFlag::new(
                            "to enable value overrides",
                        )));
                    }
                    if args.helpers {
                        return Err(Box::new(crate::error::NeedExperimentalFlag::new(
                            "to enable helpers",
                        )));
                    }
                    #[allow(unreachable_code)]
                    Ok(())
                }
                _ => Ok(()),
            },
            Privilege::Experimental => Ok(()),
        }
    }
}

#[derive(Debug)]
pub enum Privilege {
    Normal,
    Experimental,
}

#[derive(Debug)]
pub enum Command {
    Init,
    Render(RenderArguments),
}

#[derive(Debug)]
pub struct RenderArguments {
    pub configuration: String,
    pub template: Option<String>,
    pub value_overrides: std::collections::HashMap<String, String>,
    pub helpers: bool,
    pub shell_trust: ShellTrust,
    pub strict: bool,
    pub backend: Backend,
}

#[derive(Debug, Eq, PartialEq)]
pub enum ShellTrust {
    None,
    Ultimate,
}

#[derive(Debug)]
pub enum Backend {
    #[cfg(feature = "backend+cli")]
    CLI,
    #[cfg(feature = "backend+ui")]
    UI,
}

pub struct ClapArgumentLoader {}

impl ClapArgumentLoader {
    pub async fn load_from_cli() -> std::result::Result<CallArgs, Box<dyn std::error::Error>> {
        let mut backend_values = Vec::new();
        if cfg!(feature = "backend+cli") {
            backend_values.push("cli");
        }
        if cfg!(feature = "backend+ui") {
            backend_values.push("ui");
        }

        let root_command = clap::Command::new("complate")
            .version(env!("CARGO_PKG_VERSION"))
            .about("A rusty text templating application for CLIs.")
            .author("replicadse <aw@voidpointergroup.com>")
            .propagate_version(true)
            .subcommand_required(true)
            .args([
                Arg::new("experimental")
                    .short('e')
                    .long("experimental")
                    .help("enables experimental features")
                    .num_args(0)
            ])
            .subcommand(clap::Command::new("init")
                .about("Initializes a dummy default configuration in \"./.complate/config.yaml\"."))
            .subcommand(clap::Command::new("render")
                .about("Renders a template by replacing values as specified by the configuration.")
                .arg(clap::Arg::new("config")
                    .short('c')
                    .long("config")
                    .help("The configuration file to use.")
                    .default_value("./.complate/config.yaml"))
                .arg(clap::Arg::new("template")
                    .short('t')
                    .long("template")
                    .help("Specify the template to use from the config and skip it's selection."))
                .arg(clap::Arg::new("trust")
                    .long("trust")
                    .help("Enables the shell command execution. This is potentially insecure and should only be done for trustworthy sources.")
                    .action(ArgAction::SetTrue))
                .arg(clap::Arg::new("strict")
                    .short('s')
                    .long("strict")
                    .action(ArgAction::SetTrue)
                    .help("Defines whether the templating is done in strict mode (fail on missing value for variable)."))
                .arg(clap::Arg::new("helpers")
                    .long("helpers")
                    .action(ArgAction::SetTrue)
                    .help("Enables handlebar helpers."))
                .arg(clap::Arg::new("backend")
                    .short('b')
                    .long("backend")
                    .help("The execution backend (cli=native-terminal, ui=ui emulator in terminal).")
                    .value_parser(backend_values.clone())
                    .default_value(backend_values.first().unwrap()))
                .arg(clap::Arg::new("value")
                    .short('v')
                    .long("value")
                    .help("Overrides a certain value definition with a string.")));
        let command_matches = root_command.get_matches();

        let privileges = if command_matches.get_flag("experimental") {
            Privilege::Experimental
        } else {
            Privilege::Normal
        };

        if let Some(..) = command_matches.subcommand_matches("init") {
            Ok(CallArgs {
                command: Command::Init,
                privileges
            })
        } else if let Some(subc) = command_matches.subcommand_matches("render") {
            let config = std::fs::read_to_string(subc.get_one::<String>("config").unwrap())?;
            let template = subc.get_one::<String>("template").map(|v| v.to_owned());
            let shell_trust = if subc.get_flag("trust") {
                ShellTrust::Ultimate
            } else {
                ShellTrust::None
            };
            let strict = subc.get_flag("strict");

            let mut value_overrides = HashMap::<String, String>::new();
            if let Some(vo_arg) = subc.get_many::<String>("value") {
                for vo in vo_arg {
                    let spl = vo.splitn(2, "=").collect::<Vec<_>>();
                    value_overrides.insert(spl[0].to_owned(), spl[1].to_owned());
                }
            }
            let backend = match subc.get_one::<String>("backend").unwrap().as_str() {
                #[cfg(feature = "backend+cli")]
                "cli" => Backend::CLI,
                #[cfg(feature = "backend+ui")]
                "ui" => Backend::UI,
                _ => return Err(Box::new(crate::error::Failed::new("no backend specified"))),
            };
            let helpers = subc.get_flag("helpers");

            Ok(CallArgs {
                privileges,
                command: Command::Render(RenderArguments {
                    configuration: config,
                    template,
                    value_overrides,
                    shell_trust,
                    strict,
                    backend,
                    helpers,
                }),
            })
        } else {
            return Err(Box::new(crate::error::Failed::new("unknown subcommand")))
        }
    }
}
