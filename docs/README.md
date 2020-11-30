# complate

[![crates.io](https://img.shields.io/crates/v/complate.svg)](https://crates.io/crates/complate)
[![crates.io](https://img.shields.io/crates/d/complate?label=crates.io%20downloads)](https://crates.io/crates/complate)
[![pipeline](https://github.com/replicadse/complate/workflows/pipeline/badge.svg)](https://github.com/replicadse/complate/actions?query=workflow%3Apipeline)
[![docs.rs](https://img.shields.io/badge/docs.rs-latest-blue)](https://docs.rs/crate/complate/latest)
[![website](https://img.shields.io/badge/home-GitHub-blue)](https://github.com/replicadse/complate)
[![website](https://img.shields.io/badge/website-GitHub-blue)](https://replicadse.github.io/complate)

## Introduction and use case

`complate` (a portmanteau of "commit" and "template") is a project that allows the user to generate strings in a guided way. The original use-case of this was the standardization of GIT commit messages.\
Many projects and teams are standardizing their commit messages in a certain way. This is somewhat error prone and people just tend to mess things up. Spaces, spelling errors or linebreaks are common issues that lead to inconsistency. It can also have more effects than just consistency in the format. If you use [github-tag-action by anothrnick](https://github.com/anothrNick/github-tag-action) in GitHub Workflows, the commit message can have direct influence on your version number that is generated on build.

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
├── docs
    └── *
└── ...
```

This way, the `complate` program is redistributed via the GIT repository. If that's not what you want, simply keep the binary in your machine. As long as the configuration file version number fits your installed program major version you're good to go.
Expecting the recommended folder structure, you should be able to simply run `./.complate/complate print | git commit -F -` in order to create a new standardized commit.

## General overview

The template itself can be declared as string inside the configuration file or as a reference to a file that contains the template. The template string can contain variables in handlebars syntax ( `{{ variable}}` ). All distinct variables must have a representation in the according section that then also defines on how to find the value for this variable.\
Pro tip: Variables are prompted in alphabetical order. Prefix you variable with `a`, `b`, `c` and such to generate a custom order.

## Technical documentation

### Disclaimer

All features that are marked as `experimental` are _not_ considered a public API and therefore eplicitly not covered by the backwards-compatibility policy inside a major version (see [semver v2](https://semver.org)). Use these features on your own risk!

### Features

|Name|Description|Default|
|-- |-- |-- |
|backend+cli|The CLI backend which maps to the original dialoguer implementation.|Yes|
|backend+ui|The UI backend which maps to the new cursive/fui implementation.|No|

#### `backend+`

Either one of the `backend+` features (or both) MUST be enabled for `complate` to work (it won't compile otherwise).

### Application level arguments

|Name|Short|Long|Description|
|-- |-- |-- |-- |
|Experimental|-e|--experimental|Activates experimental features that are not stable yet. All features that are marked as experimental are ignored when keeping backwards compatibility inside one major version.|

### Commands

|Command|Description|Status|
|-- |-- |-- |
|help|Prints the help to `STDOUT`.|stable|
|init|Initializes the default configuration in `./.complate/config.yml`|stable|
|print|Prompts for the template, prompts for variable values and prints the data to `STDOUT`|stable|

### `print` command flags

|Name|Short|Long|Description|Remark|Status|
|-- |-- |-- |-- |-- |--|
|Config|-c|--config|The path to the configuration file that shall be used. This path can be relative or absolute. The default path is `./complate/config.yml`.|When setting this argument to pipe (`-`), the config is parsed from the STDIN descriptor to the print command.|stable for file path, experimental for STDIN descriptor. Pipe is only supported for `UI` backend|
|Shell trust||--shell-trust|Enables the shell value provider for replacing template placeholders. Due to the potential security risk with this option, it is disabled by default. Possible values for this option are `none` (default), `prompt` and `ultimate`||stable|
|Template|-t|--template|Skip the template selection by defining the used template from the configuration via this argument||stable|
|Backend|-b|--backend|Defines the backend for the user interaction.||`CLI` is stable. `UI` is experimental (feature = "backend+ui").

**Examples:**
* complate print
* complate print -f .complate/alternative.yml
* cat .complate/alternative.yml | complate -e print -c- -bui -t0.default

### Configuration file

Please find an example amd the documentation for the configuration format in the wiki.\
The templates for this project that can be found in `./complate/config.yml`.

<!-- cargo-sync-readme start -->


<!-- cargo-sync-readme end -->
