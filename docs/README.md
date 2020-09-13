# complate

[![crates.io](https://img.shields.io/crates/v/complate.svg)](https://crates.io/crates/complate)
[![crates.io](https://img.shields.io/crates/d/complate?label=crates.io%20downloads)](https://crates.io/crates/complate)
[![pipeline](https://github.com/replicadse/complate/workflows/pipeline/badge.svg)](https://github.com/replicadse/complate/actions?query=workflow%3Apipeline)
[![docs.rs](https://img.shields.io/badge/docs.rs-latest-blue)](https://docs.rs/crate/complate/latest)

## Introduction and use case

`complate` (a portmanteau of "commit" and "template") is a project that allows the user to generate strings in a guided way. The original use-case of this was the standardization of GIT commit messages.\
Many projects and teams are standardizing their commit messages in a certain way. This is somewhat error prone and people just tend to mess things up. Spaces, spelling errors or linebreaks are common issues that lead to inconsistency. It can also have more effects than just consistency in the format. If you use [github-tag-action by anothrnick](https://github.com/anothrNick/github-tag-action) in GitHub Workflows (like this project does), the commit message can have direct influence on your version number that is generated on build.

## Installation via `cargo`
Find this project on [crates.io](https://crates.io/crates/complate).
Install or update (update needs the `--force` flag) the software by executing:
```
cargo install complate --force
```

## Idea

The idea for the concrete use case of standardizing GIT commit messages is to have a configuration file inside the repository which is read by the program. You are then able to select a template that you would like to use for your message. The configuration file declares the template (in handlebars syntax) as well as variables and how to replace them.

## Usage

In order to use `complate`, the recommended way is to place the program including the configuration files and templates into the repository itself. Consider the following structure:
```
Repository root
├── .git
├── .complate
│   ├── complate
│   ├── config.yml
│   └── templates
│       └── template-a.tpl
├── src
│   └── *
└── docs
    └── *
```

`complate` writes the generated message to the stdout pipe.\
Expecting the recommended folder structure, you should be able to simply run `./.complate/complate print | git commit -F -` in order to create a new standardized commit.

## General overview

The template itself can be declared as string inside the configuration file or as a reference to a file that contains the template. The template string can contain variables in handlebars syntax ( `{{ variable}}` ). All distinct variables must have a representation in the according section that then also defines on how to find the value for this variable.

## Technical documentation

### Disclaimer

All features that are marked as `experimental` are _not_ considered a public API and therefore eplicitly not covered by the backwards-compatibility policy inside a major version (see [semver v2](https://semver.org)). Use these features on your own risk!

### Features

|Name|Description|Default|
|-- |-- |-- |
|backend::cli|The CLI backend which maps to the original dialoguer implementation.|Yes|
|baclend::ui|The UI backend which maps to the new cursive/fui implementation.|No|

### Application level arguments

|Name|Short|Long|Description|
|-- |-- |-- |-- |
|Experimental|-e|--experimental|Activates experimental features that are not stable yet. All features that are marked as experimental are not referenced when keeping backwards compatibility inside one major version.|

### Commands

|Command|Description|Status|
|-- |-- |-- |
|help|Prints the help to `STDOUT`.|stable|
|init|Initializes the default configuration in `./.complate/config.yml`|stable|
|print|Prompts for the template, prompts for variable values and prints the data to `STDOUT`|stable|

### `print` command flags
|Name|Short|Long|Description|Status|
|-- |-- |-- |-- |-- |
|Config file path|-c|--config|The path to the configuration file that shall be used. This path can be relative or absolute. The default path is `./complate/config.yml`.|stable|
|Shell trust||--shell-trust|Enables the shell value provider for replacing template placeholders. Due to the potential security risk with this option, it is disabled by default. Possible values for this option are `none` (default), `prompt` and `ultimate`|stable|
|Backend|-b|--backend|Defines the backend for the user interaction.|`CLI` is stable. `UI` is experimental (feature = "backend::ui").|

### Configuration file

Please find an example for the configuration file here:
```
version: 0.5
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
                        - backend::cli
                        - backend::ui
                        - misc

```
This project also uses complate templates that can be found in `./complate/config.yml`.

#### Value providers

|Type|Description|Fields|
|-- |-- |-- |
|static|Replaces the value by a static string.|-|
|prompt|Asks the user for input when executing the templating process.|text: string|
|shell|Invokes a certain shell command and renders STDOUT as replacing string into the variable. Due to the fact that this option can run arbitrary shell commands, it is disabled by default. Pass the `--shell-trust` flag to the CLI in order to activate this feature.|-|
|select|Gives the user the option to select _one_ item from the provided array of items.|text: string, options: []string|
|check|Gives the user the option to select _n_ items from the provided array of items.|text: string, options: []string|

<!-- cargo-sync-readme start -->


<!-- cargo-sync-readme end -->
