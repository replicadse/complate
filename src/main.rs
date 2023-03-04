use std::io::Write;
use std::path::PathBuf;
use std::result::Result;

use args::ManualFormat;

pub mod args;
pub mod config;
pub mod error;
pub mod reference;
pub mod render;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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
            std::io::stdout().write_all(res.as_bytes())?;
            Ok(())
        },
    }
}

#[cfg(test)]
mod tests {
    use std::{error::Error, process::Command};

    fn exec(command: &str) -> Result<String, Box<dyn Error>> {
        let output = Command::new("sh").arg("-c").arg(command).output()?;
        if output.status.code().unwrap() != 0 {
            return Err(Box::new(crate::error::Error::ShellCommand(
                String::from_utf8(output.stderr).unwrap(),
            )));
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
}
