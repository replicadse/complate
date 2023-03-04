mod error;

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
