# Configuration

## Introduction

The `complate` configuration schema written in YAML. It includes a version number that needs to be equal to the major version of the installed `complate` instance.\
*Bootstrap* the configuration file in `./.complate/config.yml` by using the `complate init` command.

## Structure by example

```
version: 0.9
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

```

The first tag has the current version of the configuration schema. It needs to be equal to the major version number of the cli program. Since `complate` is still sub-one, the versioning is not yet stable and therefore, the major *and* minor version need to be equal to the ones of the cli program.\
But now the important part, the templates. `complate` supports two types of loading templates. One is a file reference (relative or absolute), the other one is specifying the template inline in the configuration. This should be rather self explanatory by the example above.

### Template variables and values

The variables inside the template content are defined in the handlebars syntax ("`{{ xyz }}`"). These variables need a corresponding declaration on how to resolve the value for it under the `values` section of the template.\
You can resolve the value for each variable individual. Following options are available:\

| Key | Behaviour | Input | Remark |
|--- |--- |--- |--- |
|static|Simply replaces the variable with a static value |None||
|prompt|Asks the user for text input (can be empty)|The prompt||
|shell|Invokes a shell command to resolve the variable (read from STDOUT)|None|See `shell security`|
|select|Asks the user to select one item from a list|`text`: string (context), `options`: list (available options to select from)||
|check|Asks the user to select 0..n item(s) from a list (multiselect)|`text`: string (context), `options`: list of options {display: str, value: str} (the available options to select from)||

#### Shell security

Since the `shell` value provider is able to run arbitrary shell commands, it is only allowed if and only if the `SHELL_TRUST` argument is explicitly set. See the `render` command reference for possible values for this setting. If *not* set, the provider will throw an unrecoverable error and the program will abort.
