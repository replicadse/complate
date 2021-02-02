[![crates.io](https://img.shields.io/crates/v/complate.svg)](https://crates.io/crates/complate)
[![crates.io](https://img.shields.io/crates/d/complate?label=crates.io%20downloads)](https://crates.io/crates/complate)
[![pipeline](https://github.com/replicadse/complate/workflows/pipeline/badge.svg)](https://github.com/replicadse/complate/actions?query=workflow%3Apipeline)
[![dependency status](https://deps.rs/repo/github/replicadse/complate/status.svg)](https://deps.rs/repo/github/replicadse/complate)\
[![docs.rs](https://img.shields.io/badge/docs.rs-latest-blue)](https://docs.rs/crate/complate/latest)
[![website](https://img.shields.io/badge/home-GitHub-blue)](https://github.com/replicadse/complate)
[![website](https://img.shields.io/badge/website-GitHub-blue)](https://replicadse.github.io/complate)

# Introduction and use case

`complate` (a portmanteau of "commit" and "template") is a project that allows the user to generate strings in a guided way. The original use-case of this was the standardization of GIT commit messages.\
Many projects and teams are standardizing their commit messages in a certain way. This is somewhat error prone and people just tend to mess things up. Spaces, spelling errors or linebreaks are common issues that lead to inconsistency. It can also have more effects than just consistency in the format. If you use [github-tag-action by anothrnick](https://github.com/anothrNick/github-tag-action) in GitHub Workflows, the commit message can have direct influence on your version number that is generated on build.

## Idea

The idea for the concrete use case of standardizing GIT commit messages is to have a configuration file inside the repository which is read by the program. You are then able to select a template that you would like to use for your message. The configuration file declares the template (in handlebars syntax) as well as variables and how to replace them.

## Usage

In order to use `complate`, the recommended way is install the tool via `cargo` and place the configuration files and templates into the repository itself. Consider the following structure:
```
Repository root
├── .git
├── .complate
│   ├── config.yml
│   └── templates
│       └── template-a.tpl
├── src
│   └── *
├── docs
    └── *
└── ...
```

As long as the configuration file version number fits your installed program major version you're good to go.
Expecting the recommended folder structure (and assuming that the program is in your `PATH` environment variable), you should be able to simply run `complate render | git commit -F -` in order to create a new standardized commit.

## General overview

The template itself can be declared as string inside the configuration file or as a reference to a file that contains the template. The template string can contain variables in handlebars syntax ( `{{ variable}}` ). All distinct variables must have a representation in the according section that then also defines on how to find the value for this variable.\
Pro tip: Variables are prompted in alphabetical order. Prefix you variable with `a`, `b`, `c` and such to generate a custom order.
