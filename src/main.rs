use {
    anyhow::Result,
    args::ManualFormat,
    std::path::PathBuf,
};

mod args;
mod config;
mod reference;
mod render;

#[tokio::main]
async fn main() -> Result<()> {
    let cmd = args::ClapArgumentLoader::load().await?;
    cmd.validate().await?;

    match cmd.command {
        | args::Command::Manual { path, format } => {
            let out_path = PathBuf::from(path);
            std::fs::create_dir_all(&out_path)?;
            match format {
                | ManualFormat::Manpages => {
                    reference::build_manpages(&out_path)?;
                },
                | ManualFormat::Markdown => {
                    reference::build_markdown(&out_path)?;
                },
            }
            Ok(())
        },
        | args::Command::Autocomplete { path, shell } => {
            let out_path = PathBuf::from(path);
            std::fs::create_dir_all(&out_path)?;
            reference::build_shell_completion(&out_path, &shell)?;
            Ok(())
        },
        | args::Command::Init => {
            std::fs::create_dir_all("./.complate")?;
            std::fs::write("./.complate/config.yaml", config::default_config().await)?;
            Ok(())
        },
        | args::Command::Render(x) => {
            let res = render::select_and_render(x).await?;
            print!("{}", res);
            Ok(())
        },
        | args::Command::Schema => {
            println!(
                "{}",
                serde_json::to_string_pretty(&schemars::schema_for!(config::Config))?
            );
            Ok(())
        },
    }
}

#[cfg(test)]
mod tests {
    use {
        anyhow::Result,
        std::process::Command,
    };

    fn exec(command: &str) -> Result<String> {
        let output = Command::new("sh").arg("-c").arg(command).output()?;
        if output.status.code().unwrap() != 0 {
            return Err(anyhow::anyhow!(String::from_utf8(output.stderr)?));
        }
        Ok(String::from_utf8(output.stdout)?)
    }

    const CONFIG_PATH: &'static str = "./test/.complate/config.yaml";

    #[test]
    fn template_var_static() {
        assert!("alpha" == exec(&format!("cargo run -- render -c {} -t var:static", CONFIG_PATH)).unwrap())
    }

    #[test]
    fn template_var_env() {
        assert!(
            "alpha"
                == exec(&format!(
                    "alpha=\"alpha\" cargo run -- render -c {} -t var:env",
                    CONFIG_PATH
                ))
                .unwrap()
        )
    }

    #[test]
    fn template_var_shell() {
        assert!(
            "alpha"
                == exec(&format!(
                    "alpha=\"alpha\" cargo run -- render -c {} -t var:shell --trust",
                    CONFIG_PATH
                ))
                .unwrap()
        )
    }

    #[test]
    fn template_overrides() {
        assert!(
            "alpha"
                == exec(&format!(
                    "cargo run -- render -c {} -t override -v a.alpha=\"alpha\"",
                    CONFIG_PATH
                ))
                .unwrap()
        )
    }

    #[test]
    fn template_helper() {
        assert!("bananarama" == exec(&format!("cargo run -- render -c {} -t helper --trust", CONFIG_PATH)).unwrap())
    }

    #[test]
    fn template_vals_multiple() {
        assert!(
            "alpha\nbravo"
                == exec(&format!(
                    "cargo run -- render -c {} --trust -t vals:multiple -v a.alpha=alpha -v b.bravo=bravo",
                    CONFIG_PATH
                ))
                .unwrap()
        )
    }

    #[test]
    fn template_var_argument() {
        assert!(
            "alpha"
                == exec(&format!(
                    "cargo run -- render -c {} -t var:argument -v a.alpha=alpha",
                    CONFIG_PATH
                ))
                .unwrap()
        )
    }
}
