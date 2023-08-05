# complate

`complate` is a general purpose text templating CLI program that supports interactive mode, prompting the user for values via TUI behaviour and headless mode for use in automation such as CI pipelines.

## Installation

* The rusty way:\
`cargo install complate`
* The manual way:\
Download and install from the GitHub releases.

## Config example

```
version: 0.13
templates:
  zero:
    content:
      inline: |-
        {{ a.alpha }}
        {{ b.bravo }}
    variables:
      a.alpha:
        static: alpha
      b.bravo: arg

  one:
    content:
      file: ./.complate/templates/arbitraty-template-file.tpl
    variables:
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
    variables:
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
        {{ test }}
        {{ _decode "dGVzdA==" }}
    helpers:
      "_decode": printf "$(printf $VALUE | base64 -D)"
    variables:
      test:
        static: "test"

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
