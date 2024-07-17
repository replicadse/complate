use {
    anyhow::Result,
    clap::{
        Arg,
        ArgAction,
    },
    std::{
        collections::HashMap,
        str::FromStr,
    },
};

#[derive(Debug)]
pub struct CallArgs {
    pub privileges: Privilege,
    pub command: Command,
}

impl CallArgs {
    pub async fn validate(&self) -> Result<()> {
        match self.privileges {
            | Privilege::Normal => {
                match &self.command {
                    | _ => Ok(()),
                }
            },
            | Privilege::Experimental => Ok(()),
        }
    }
}

#[derive(Debug)]
pub enum Privilege {
    Normal,
    Experimental,
}

#[derive(Debug)]
pub enum ManualFormat {
    Manpages,
    Markdown,
}

#[derive(Debug)]
pub enum Command {
    Manual { path: String, format: ManualFormat },
    Autocomplete { path: String, shell: clap_complete::Shell },
    Init,
    Schema,
    Render(crate::render::RenderArguments),
}

pub struct ClapArgumentLoader {}

impl ClapArgumentLoader {
    pub fn root_command() -> clap::Command {
        let mut backend_values = Vec::from(["headless"]);
        if cfg!(feature = "backend+cli") {
            backend_values.push("cli");
        }

        clap::Command::new("complate")
            .version(env!("CARGO_PKG_VERSION"))
            .about("A rusty text templating application for CLIs.")
            .author("replicadse <aw@voidpointergroup.com>")
            .propagate_version(true)
            .subcommand_required(true)
            .args([Arg::new("experimental")
                .short('e')
                .long("experimental")
                .help("enables experimental features")
                .num_args(0)])
            .subcommand(
                clap::Command::new("man")
                    .about("Renders the manual.")
                    .arg(clap::Arg::new("out").short('o').long("out").required(true))
                    .arg(
                        clap::Arg::new("format")
                            .short('f')
                            .long("format")
                            .value_parser(["manpages", "markdown"])
                            .required(true),
                    ),
            )
            .subcommand(
                clap::Command::new("autocomplete")
                    .about("Renders shell completion scripts.")
                    .arg(clap::Arg::new("out").short('o').long("out").required(true))
                    .arg(
                        clap::Arg::new("shell")
                            .short('s')
                            .long("shell")
                            .value_parser(["bash", "zsh", "fish", "elvish", "powershell"])
                            .required(true),
                    ),
            )
            .subcommand(
                clap::Command::new("init")
                    .about("Initializes a dummy default configuration in \"./.complate/config.yaml\"."),
            )
            .subcommand(clap::Command::new("schema").about("Renders the configuration schema."))
            .subcommand(
                clap::Command::new("render")
                    .about("Renders a template by replacing values as specified by the configuration.")
                    .arg(
                        clap::Arg::new("config")
                            .short('c')
                            .long("config")
                            .help("The configuration file to use.")
                            .default_value("./.complate/config.yaml"),
                    )
                    .arg(
                        clap::Arg::new("template")
                            .short('t')
                            .long("template")
                            .help("Specify the template to use from the config and skip it's selection."),
                    )
                    .arg(
                        clap::Arg::new("trust")
                            .long("trust")
                            .help(
                                "Enables the shell command execution. This is potentially insecure and should only be \
                                 done for trustworthy sources.",
                            )
                            .action(ArgAction::SetTrue),
                    )
                    .arg(
                        clap::Arg::new("loose")
                            .short('l')
                            .long("loose")
                            .action(ArgAction::SetTrue)
                            .help(
                                "Defines that the templating is done in non-strict mode (allow missing value for \
                                 variable).",
                            ),
                    )
                    .arg(
                        clap::Arg::new("backend")
                            .short('b')
                            .long("backend")
                            .help("The execution backend (cli=native-terminal, ui=ui emulator in terminal).")
                            .value_parser(backend_values.clone())
                            .default_value("headless"),
                    )
                    .arg(
                        clap::Arg::new("value")
                            .short('v')
                            .long("value")
                            .action(ArgAction::Append)
                            .help("Overrides a certain value definition with a string."),
                    ),
            )
    }

    pub async fn load() -> Result<CallArgs> {
        let root_command = Self::root_command();
        let command_matches = root_command.get_matches();

        let privileges = if command_matches.get_flag("experimental") {
            Privilege::Experimental
        } else {
            Privilege::Normal
        };

        if let Some(subc) = command_matches.subcommand_matches("man") {
            Ok(CallArgs {
                command: Command::Manual {
                    path: subc.get_one::<String>("out").unwrap().into(),
                    format: match subc.get_one::<String>("format").unwrap().as_str() {
                        | "manpages" => ManualFormat::Manpages,
                        | "markdown" => ManualFormat::Markdown,
                        | _ => return Err(anyhow::anyhow!("unknown format")),
                    },
                },
                privileges,
            })
        } else if let Some(subc) = command_matches.subcommand_matches("autocomplete") {
            Ok(CallArgs {
                command: Command::Autocomplete {
                    path: subc.get_one::<String>("out").unwrap().into(),
                    shell: clap_complete::Shell::from_str(subc.get_one::<String>("shell").unwrap().as_str()).unwrap(),
                },
                privileges,
            })
        } else if let Some(..) = command_matches.subcommand_matches("schema") {
            Ok(CallArgs {
                command: Command::Schema,
                privileges,
            })
        } else if let Some(..) = command_matches.subcommand_matches("init") {
            Ok(CallArgs {
                command: Command::Init,
                privileges,
            })
        } else if let Some(subc) = command_matches.subcommand_matches("render") {
            let config = std::fs::read_to_string(subc.get_one::<String>("config").unwrap())?;
            let template = subc.get_one::<String>("template").map(|v| v.into());
            let shell_trust = if subc.get_flag("trust") {
                crate::render::ShellTrust::Ultimate
            } else {
                crate::render::ShellTrust::None
            };
            let loose = subc.get_flag("loose");

            let mut value_overrides = HashMap::<String, String>::new();
            if let Some(vo_arg) = subc.get_many::<String>("value") {
                for vo in vo_arg {
                    let spl = vo.splitn(2, "=").collect::<Vec<_>>();
                    value_overrides.insert(spl[0].into(), spl[1].into());
                }
            }
            let backend = match subc.get_one::<String>("backend").unwrap().as_str() {
                | "headless" => crate::render::Backend::Headless,
                #[cfg(feature = "backend+cli")]
                | "cli" => crate::render::Backend::CLI,
                | _ => return Err(anyhow::anyhow!("no backend specified")),
            };

            Ok(CallArgs {
                privileges,
                command: Command::Render(crate::render::RenderArguments {
                    configuration: config,
                    template,
                    value_overrides,
                    shell_trust,
                    loose,
                    backend,
                }),
            })
        } else {
            return Err(anyhow::anyhow!("unknown command"));
        }
    }
}
