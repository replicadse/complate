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
    fn setup_test() -> clitest::CliTestSetup {
        let mut x = clitest::CliTestSetup::new();
        x.with_env("CFG", CONFIG_PATH);
        x
    }

    const CONFIG_PATH: &'static str = "./test/.complate/config.yaml";

    #[test]
    fn template_var_static() {
        assert!("alpha" == setup_test()
            .run("render -c $CFG -t var:static").unwrap().success().unwrap().stdout_str());
    }

    #[test]
    fn template_var_env() {
        assert!("alpha" == setup_test()
            .with_env("alpha", "alpha")
            .run("render -c $CFG -t var:env").unwrap().success().unwrap().stdout_str());
    }

    #[test]
    fn template_var_shell() {
        assert!("alpha" == setup_test()
            .with_env("alpha", "alpha")
            .run("render -c $CFG -t var:shell --trust").unwrap().success().unwrap().stdout_str());
    }

    #[test]
    fn template_overrides() {
        assert!("alpha" == setup_test()
            .run("render -c $CFG -t override -v a.alpha=\"alpha\"").unwrap().success().unwrap().stdout_str());
    }

    #[test]
    fn template_helper() {
        assert!("bananarama" == setup_test()
            .run("render -c $CFG -t helper --trust").unwrap().success().unwrap().stdout_str());
    }

    #[test]
    fn template_vals_multiple() {
        assert!("alpha\nbravo" == setup_test()
            .run("render -c $CFG --trust -t vals:multiple -v a.alpha=alpha -v b.bravo=bravo").unwrap().success().unwrap().stdout_str());
    }

    #[test]
    fn template_var_argument() {
        assert!("alpha" == setup_test()
            .run("render -c $CFG -t var:argument -v a.alpha=alpha").unwrap().success().unwrap().stdout_str());
    }
}
