# 6-render-command-supporting-stdin-pipe

## Summary

The render command MUST support piping the configuration into it via STDIN as alternative to the file based approach.\
The indicator for the pipe being used MUST be the `-` character as value for the `configuration` argument to the command.\
