# complate

## Introduction

Complate is a currently work in progress application that allows the user to create and use templates for commit messages. The idea is to consolidate commit messages in a project to include all the necessary information.

## Idea

The idea for implementing this is to have an application (`complate`) that substitutes the `git commit` command. It then shows a selection of the available templates for the user to select. After it loaded the templates, it will auto-fill certain information in them. All the information that is not automatically filled will be input by the user.\
The templates themselves are plain text files that contain placeholders in the `handlebars` format (two curly brackets open & closed with key inside, like: `{{ git.branch }}`).\
An example template for a defect could look like so:
```
{{ user.name }}
Fixed defect {{ ticket.nr }}.

{{ description }}

Changes (+ added / o changed / - removed):
{{ git.staged.files }}
```
This example could translate into:
```
Weber, Heiko Alexander & Reddingtion, Raymond
Fixed defect #4711.

Users were not able to login due to a timeout in the Redis connection. Fixed Redis endpoint and made it configurable.

Changes (+ added / o changed / - removed):
+ redis_configure.h
+ redis_configre.cpp
o redis_client.cpp
- old_redis_config.hpp 
```

This project is still in idea phase so feel free to suggest things.

## Current implementation

The structure of the `./.complate/config.yml` file is currently as follows:
```
templates                           # dictionary
└── {{ template name }}              # dictionary
    ├── file                         # string, relative file path
    ├── prompt                       # array of values to prompt
    └── values                       # dictionary
        ├── A
        └── B
```
An example for this could look like follows:
```
templates:
    feature:
        content:
            inline: |-
                {{ summary }} | {{ version }}
                {{ components }}
                {{ author }}
                {{ git.staged.files }}

        values:
            author:
                static: "Weber, Heiko Alexander <heiko.a.weber@gmail.com>"
            summary:
                prompt: "Enter the summary"
            git.staged.files:
                shell: "git diff --name-status --cached"
            version:
                select:
                    - "#patch"
                    - "#minor"
                    - "#major"
            components:
                check:
                    - A
                    - B
                    - C
```

### Values

Values can be provided in three different ways.

|Type|Description|
|-- |-- |
|static|Replaces the value by a static string.|
|prompt|Asks the user for input when executing the templating process.|
|shell|Invokes a certain shell command and renders STDOUT as replacing string into the variable. Due to the fact that this option can run arbitrary shell commands, it is disabled by default. Pass the `--shell-trust` flag to the CLI in order to activate this feature.|
|select|Gives the user the option to select one item from the provided array of items.|
|check|Gives the user the option to select _n_ items from the provided array of items.|

## Usage

In order to use complate, the recommended way is to place the program including the configuration files and templates into the repository itself. Consider the following structure:
```
Repository root
├── .git
├── .complate
│   ├── complate
│   ├── config.yml
│   └── templates
│       └── template_a
├── src
│   └── *
└── docs
    └── *
```

complate writes the generated commit message to the stdout pipe. Expecting the recommended folder structure, you should be able to simply run `./.complate/complate | git commit -F -` in order to create a new standardized commit.

### Arguments

complate can run without specifying any arguments. It will always look for the configuration file under `./.complate/config.yml`. If you want to use a different configuration file, specify the path using `-c` argument. Example: `./.complate/complate -c "./unusual_folder/some-config.yml" | git commit -F -`.

All arguments can be found here:

|Name|Short|Long|Description|
|-- |-- |-- |-- |
|Help|-h|--help|Calls the help that displays all the available arguments and commands.|
|Config file path|-c|--config|The path to the configuration file that shall be used. This path can be relative or absolute.|
|Shell trust||--shell-trust|Enables the shell value provider for replacing template placeholders. Due to the potential security risk with this option, it is disabled by default. Possible values for this option are `none` (default), `prompt` and `ultimate`|
