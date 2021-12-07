# Command reference

## Disclaimer

All features that are marked as `experimental` are _not_ considered a public API and therefore eplicitly not covered by the backwards-compatibility policy inside a major version (see [semver v2](https://semver.org)). Use these features on your own risk!

## Features

|Name|Description|Default|
|-- |-- |-- |
|backend+cli|The CLI backend which maps to the original dialoguer implementation.|Yes|
|backend+ui|The UI backend which maps to the new cursive/fui implementation.|No|

### `backend+`

Either one of the `backend+` features (or both) MUST be enabled for `complate` to work (it won't compile otherwise).

## Application level arguments

|Name|Short|Long|Description|
|-- |-- |-- |-- |
|Experimental|-e|--experimental|Activates experimental features that are not stable yet. All features that are marked as experimental are ignored when keeping backwards compatibility inside one major version.|

## Commands

|Command|Description|Status|
|-- |-- |-- |
|help|Prints the help to `STDOUT`.|stable|
|init|Initializes the default configuration in `./.complate/config.yaml`|stable|
|render|Prompts for the template, prompts for variable values and renders the data to `STDOUT`|stable|

## `render` command flags

|Name|Short|Long|Description|Remark|Status|
|-- |-- |-- |-- |-- |--|
|Config|-c|--config|The path to the configuration file that shall be used. This path can be relative or absolute. The default path is `./complate/config.yaml`.|When setting this argument to pipe (`-`), the config is parsed from the STDIN descriptor to the render command.|stable for file path, experimental for STDIN descriptor. Pipe is only supported for `UI` backend|
|Shell trust||--shell-trust|Enables the shell value provider for replacing template placeholders. Due to the potential security risk with this option, it is disabled by default. Possible values for this option are `none` (default), `prompt` and `ultimate`||stable|
|Template|-t|--template|Skip the template selection by defining the used template from the configuration via this argument||stable|
|Backend|-b|--backend|Defines the backend for the user interaction.||`CLI` is stable. `UI` is experimental (feature = "backend+ui").|
|Value|-v|--value|Overrides a certain variable with a given string value. Specify the variable and value with an equals sign as separator|Multiple allowed. Example: <br/> `-v"variable=some arbitrary value"`|experimental|

**Examples:**
* `complate render`
* `complate render -c .complate/alternative.yml`
* `complate -e render -c <(cat .complate/alternative.yaml) -bui -t0.default`

## Configuration file

Please find an example amd the documentation for the configuration format in the wiki.\
The templates for this project that can be found in `./complate/config.yaml`.
