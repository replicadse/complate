include!("check_features.rs");

use futures::executor::block_on;

use std::io::{Result, Write};

pub mod args;
pub mod config;
pub mod render;

async fn default_config() -> String {
    r###"version: 0.6
templates:
    default:
        content:
            inline: |-
                {{ a.summary }} | {{ e.version }}
                Components: [{{ f.components }}]
                Author: {{ b.author.name }} | {{ c.author.account }}
                
                Files:
                {{ d.git.staged.files }}
        values:
            a.summary:
                prompt: "Enter the summary"
            b.author.name:
                shell: "git config user.name | tr -d '\n'"
            c.author.account:
                shell: "whoami | tr -d '\n'"
            d.git.staged.files:
                shell: "git diff --name-status --cached"
            e.version:
                select:
                    text: Select the version level that shall be incremented
                    options:
                        - "#patch"
                        - "#minor"
                        - "#major"
            f.components:
                check:
                    text: Select the components that are affected
                    options:
                        - security
                        - command::print
                        - backend+cli
                        - backend+ui
                        - misc
    
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
        crate::args::Command::Print(x) => {
            let res = crate::render::select_and_render(x).await?;
            std::io::stdout().write_all(res.as_bytes())?;
            Ok(())
        }
    }
}

fn main() -> Result<()> {
    block_on(async_main())
}
