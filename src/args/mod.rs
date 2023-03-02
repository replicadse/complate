use std::result::Result;

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

pub enum Privilege {
    Normal,
    Experimental,
}

pub enum Command {
    Init,
    Render(RenderArguments),
}

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

        let command = clap::App::new("complate")
            .version(env!("CARGO_PKG_VERSION"))
            .about("A rusty text templating application for CLIs.")
            .author("replicadse <aw@voidpointergroup.com>")
            .arg(clap::Arg::with_name("experimental")
                    .short("e")
                    .long("experimental")
                    .value_name("EXPERIMENTAL")
                    .help("Enables experimental features that do not count as stable.")
                    .required(false)
                    .takes_value(false))
            .subcommand(clap::App::new("init")
                .about("Initializes a dummy default configuration in \"./.complate/config.yaml\"."))
            .subcommand(clap::App::new("render")
                .about("Renders a template by replacing values as specified by the configuration.")
                .arg(clap::Arg::with_name("config")
                    .short("c")
                    .long("config")
                    .value_name("FILE")
                    .help("The configuration file to use.")
                    .default_value("./.complate/config.yaml")
                    .multiple(false)
                    .required(false)
                    .takes_value(true))
                .arg(clap::Arg::with_name("template")
                    .short("t")
                    .long("template")
                    .value_name("TEMPLATE")
                    .help("Specify the template to use from the config and skip it's selection.")
                    .multiple(false)
                    .required(false)
                    .takes_value(true))
                .arg(clap::Arg::with_name("shell-trust")
                    .long("shell-trust")
                    .value_name("SHELL_TRUST")
                    .help("Enables the shell mode. This is potentially insecure and should only be done for trustworthy sources.")
                    .possible_values(&["none", "ultimate"])
                    .multiple(false)
                    .required(false)
                    .default_value("none")
                    .takes_value(true))
                .arg(clap::Arg::with_name("strict")
                    .short("s")
                    .long("strict")
                    .value_name("STRICT")
                    .help("Defines whether the templating is done in strict mode (fail on missing value for variable).")
                    .possible_values(&["true", "false"])
                    .default_value("true")
                    .multiple(false)
                    .required(false)
                    .takes_value(true))
                .arg(clap::Arg::with_name("helpers")
                    .long("helpers")
                    .value_name("HELPERS")
                    .help("Enables handlebar helpers.")
                    .multiple(false)
                    .required(false)
                    .takes_value(false))
                .arg(clap::Arg::with_name("backend")
                    .short("b")
                    .long("backend")
                    .value_name("BACKEND")
                    .help("The execution backend (cli=native-terminal, ui=ui emulator in terminal).")
                    .possible_values(backend_values.as_slice())
                    .default_value(backend_values.first().unwrap())
                    .multiple(false)
                    .required(false)
                    .takes_value(true))
                .arg(clap::Arg::with_name("value")
                    .short("v")
                    .long("value")
                    .value_name("VALUE")
                    .help("Overrides a certain value definition with a string.")
                    .multiple(true)
                    .required(false)
                    .takes_value(true)))
            .get_matches();

        let privileges = if command.is_present("experimental") {
            Privilege::Experimental
        } else {
            Privilege::Normal
        };

        if command.subcommand_matches("init").is_some() {
            return Ok(CallArgs {
                privileges,
                command: Command::Init,
            });
        }

        match command.subcommand_matches("render") {
            Some(x) => {
                let config = if x.is_present("config") {
                    let config_param = x.value_of("config").unwrap();
                    std::fs::read_to_string(config_param.to_owned())?
                } else {
                    return Err(Box::new(crate::error::Failed::new(
                        "configuration unspecified",
                    )));
                };

                let template = if x.is_present("template") {
                    Some(x.value_of("template").unwrap().to_owned())
                } else {
                    None
                };

                let shell_trust = match x.value_of("shell-trust") {
                    Some(x) => match x {
                        "none" => ShellTrust::None,
                        "ultimate" => ShellTrust::Ultimate,
                        _ => ShellTrust::None,
                    },
                    None => ShellTrust::None,
                };

                let strict = match x.value_of("strict") {
                    Some(x) => x.parse::<bool>()?,
                    None => true,
                };

                let mut value_overrides: std::collections::HashMap<String, String> =
                    std::collections::HashMap::new();
                if let Some(values_overrides_arg) = x.values_of("value") {
                    for vo in values_overrides_arg {
                        let spl: Vec<&str> = vo.splitn(2, "=").collect();
                        value_overrides.insert(spl[0].to_owned(), spl[1].to_owned());
                    }
                }

                let backend = match x.value_of("backend") {
                    Some(x) => match x {
                        #[cfg(feature = "backend+cli")]
                        "cli" => Backend::CLI,
                        #[cfg(feature = "backend+ui")]
                        "ui" => Backend::UI,
                        _ => {
                            return Err(Box::new(crate::error::Failed::new("no backend specified")))
                        }
                    },
                    #[cfg(feature = "backend+cli")]
                    None => Backend::CLI,
                    #[cfg(feature = "backend+ui")]
                    #[allow(unreachable_patterns)]
                    None => Backend::UI,
                };

                let helpers = x.is_present("helpers");

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
            }
            None => Err(Box::new(crate::error::UnknownCommand::default())),
        }
    }
}
