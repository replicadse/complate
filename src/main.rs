include!("check_features.rs");

use futures::executor::block_on;

use std::io::{Result, Write};

pub mod args;
pub mod config;
pub mod render;

async fn default_config() -> String {
    r###"version: 0.7
templates:
    default:
        content:
            inline: |-
                {{ a }}
                {{ b }}
                {{ c }}
                {{ d }}
        values:
            a:
                prompt: "A"
            b:
                shell: "whoami | tr -d '\n'"
            c:
                select:
                    text: Select the version level that shall be incremented
                    options:
                        - display: "#patch"
                          value: "#patch"
                        - display: "#minor"
                          value: "#minor"
                        - display: "#major"
                          value: "#major"
            d:
                check:
                    text: Select the components that are affected
                    options:
                        - display: security
                          value: security
                        - display: command:init
                          value: command:init
                        - display: command:render
                          value: command:render
                        - display: backend+cli
                          value: backend+cli
                        - display: backend+ui
                          value: backend+ui
                        - display: docs
                          value: docs
                        - display: ci
                          value: ci
                        - display: misc
                          value: misc
    
"###
    .to_owned()
}

async fn async_main() -> Result<()> {
    let cmd = crate::args::ClapArgumentLoader::load_from_cli().await?;
    cmd.validate().await?;

    match cmd.command {
        crate::args::Command::Init => {
            std::fs::create_dir_all("./.complate")?;
            std::fs::write("./.complate/config.yml", default_config().await)?;
            Ok(())
        }
        crate::args::Command::Render(x) => {
            let res = crate::render::select_and_render(x).await?;
            std::io::stdout().write_all(res.as_bytes())?;
            Ok(())
        }
    }
}

fn main() -> Result<()> {
    block_on(async_main())
}
