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
{{ commit.content }}
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

The structure of the `./res/config.json` file is currently as follows:
```
templates                           # dictionary
└──{{ template name }}              # dictionary
    ├──file                         # string, relative file path
    ├──prompt                       # array of values to prompt
    └──values                       # dictionary
        └──static                   # dictionary
            ├──{{ key 0 }}          # string
            └──{{ key 1 }}          # string
```
An example for this could look like follows:
```
{
    "templates": {
        "feature": {
            "file": ".complate/templates/feature.tpl",
            "prompt": [
                "issue",
                "summary"
            ],
            "values": {
                "static": {
                    "git": {
                        "author": "Weber, Heiko Alexander <heiko.a.weber@gmail.com>"
                    }
                }
            }
        }
    }
}
```
A matching template could look like so:
```
#{{ issue }} | {{ git.author }}
#{{ summary }}
```
Be aware of the fact that you are able to set static values using a flat hierarchy and nesting. Therefore `{ "git": { "author": "Reddingtion" } }` will lead to the same result as `{ "git.author": "Reddingtion" }`.

## Example

complate writes the generated commit message to the stdout pipe. Therefore you should be able to simply run `complate | git commit -F -` in order to create a new standardized commit.
