include!("check_features.rs");

use futures::executor::block_on;

use std::io::{Result, Write};

pub mod args;
pub mod config;
pub mod render;

async fn default_config() -> String {
    r###"version: 0.9
    templates:
        one:
            content:
                file: ./.complate/templates/arbitraty-template-file.tpl
            values:
                a.summary:
                    static: "random summary"
        two:
            content:
                inline: |-
                    {{ a.alpha }}
                    {{ b.bravo }}
                    {{ c.charlie }}
                    {{ d.delta }}
                    {{ e.echo }}
            values:
                a.alpha:
                  prompt: "alpha"
                b.bravo:
                  shell: "printf bravo"
                c.charlie:
                  static: "charlie"
                d.delta:
                    select:
                        text: Select the version level that shall be incremented
                        options:
                          alpha:
                            display: alpha
                            value:
                              static: alpha
                          bravo:
                            display: bravo
                            value:
                              shell: printf bravo
                e.echo:
                    check:
                        text: Select the components that are affected
                        separator: ", "
                        options:
                          alpha:
                            display: alpha
                            value:
                              static: alpha
                          bravo:
                            display: bravo
                            value:
                              shell: printf bravo

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
