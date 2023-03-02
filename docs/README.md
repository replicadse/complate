# complate

`complate` is a general purpose text templating CLI program that supports interactive mode, prompting the user for values via TUI behaviour and headless mode for use in automation such as CI pipelines.

## Installation

* The rusty way:\
`cargo install complate --force`
* The manual way:\
Download and install from the GitHub releases


## Usage

* `complate render`
* `complate render -c .complate/alternative.yml`
* `complate -e render -c <(cat .complate/alternative.yaml) -bui -t0.default`
* `complate -e render -t three --helpers`

## Configuration

An example:

```
version: 0.11
templates:
  zero:
    content:
      inline: |-
        {{ a.alpha }}
    values:
      a.alpha:
        static: ALPHA
  one:
    content:
      file: ./.complate/templates/arbitraty-template-file.tpl
    values:
      a.pwd:
        env: "PWD"
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
      f.foxtrot:
        env: "FOXTROT"
  three:
    content:
      inline: |-
        {{ _decode "dGVzdA==" }}
    helpers:
      "_decode":
        shell: |-
          printf "$(printf $VALUE | base64 -D)"
    values: {}

```

| Key    | Behaviour                                                            | Input                                                                                                                  |
| ------ | -------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------- |
| env    | Retrieves value from the specified env var                           | None                                                                                                                   |
| static | Simply replaces the variable with a static value                     | None                                                                                                                   |
| prompt | Asks the user for text input (can be empty)                          | The prompt                                                                                                             |
| shell  | Invokes a shell command to resolve the variable (read from `STDOUT`) | None                                                                                                                   |
| select | Asks the user to select one item from a list                         | `text`: string (context), `options`: list (available options to select from)                                           |
| check  | Asks the user to select `0..n` item(s) from a list (multiselect)     | `text`: string (context), `options`: list of options {display: str, value: str} (the available options to select from) |

Since the `shell` value provider is able to run arbitrary shell commands, it is only allowed if and only if the `SHELL_TRUST` argument is explicitly set. See the `render` command reference for possible values for this setting. If *not* set, the provider will throw an unrecoverable error and the program will abort.

## Command reference

### Disclaimer

All features that are marked as `experimental` are _not_ considered a public API and therefore eplicitly not covered by the backwards-compatibility policy inside a major version (see https://semver.org[semver v2]). Use these features on your own risk!

### Application level arguments

| Name         | Short | Long           | Description                                                                                                                                                                      |
| ------------ | ----- | -------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Experimental | -e    | --experimental | Activates experimental features that are not stable yet. All features that are marked as experimental are ignored when keeping backwards compatibility inside one major version. |

### Commands

| Command | Description                                                                            | Status |
| ------- | -------------------------------------------------------------------------------------- | ------ |
| help    | Prints the help to `STDOUT`.                                                           | stable |
| init    | Initializes the default configuration in `./.complate/config.yaml`                     | stable |
| render  | Prompts for the template, prompts for variable values and renders the data to `STDOUT` | stable |

#### `render` command flags

| Name        | Short | Long          | Description                                                                                                                                                                                                               | Remark                                                                                                          | Status                                                                                           |
| ----------- | ----- | ------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------ |
| Config      | -c    | --config      | The path to the configuration file that shall be used. This path can be relative or absolute. The default path is `./complate/config.yaml`.                                                                               | When setting this argument to pipe (`-`), the config is parsed from the STDIN descriptor to the render command. | stable for file path, experimental for STDIN descriptor. Pipe is only supported for `UI` backend |
| Shell trust |       | --shell-trust | Enables the shell value provider for replacing template placeholders. Due to the potential security risk with this option, it is disabled by default. Possible values for this option are `none` (default) and `ultimate` |                                                                                                                 | stable                                                                                           |
| Template    | -t    | --template    | Skip the template selection by defining the used template from the configuration via this argument                                                                                                                        |                                                                                                                 | stable                                                                                           |
| Backend     | -b    | --backend     | Defines the backend for the user interaction.                                                                                                                                                                             |                                                                                                                 | `CLI` is stable. `UI` is experimental (feature = "backend+ui").                                  |
| Value       | -v    | --value       | Overrides a certain variable with a given string value. Specify the variable and value with an equals sign as separator                                                                                                   | Multiple allowed. Example: <br/> `-v"variable=some arbitrary value"`                                            | experimental                                                                                     |
| Helpers     |       | --helpers     | Enables helper functions.                                                                                                                                                                                                 |                                                                                                                 | experimental                                                                                     |

## Features

| Name        | Description                                                          | Default |
| ----------- | -------------------------------------------------------------------- | ------- |
| backend+cli | The CLI backend which maps to the original dialoguer implementation. | Yes     |
| backend+ui  | The UI backend which maps to the new cursive/fui implementation.     | No      |

### `backend+`

Either one of the `backend+` features (or both) MUST be enabled for `complate` to work (it won't compile otherwise).
