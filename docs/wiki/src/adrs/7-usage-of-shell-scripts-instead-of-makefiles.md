# 7-usage-of-shell-scripts-instead-of-makefiles

## Summary

Originally, this project had a Makefile in order to do the build steps and such. Since Makefile still is kind of an alien syntax and has some disadvantages against standard plain shell scripts (bash scripts), I decided to replace the Makefile with a shell script (`./do.sh`).\
This script now contains the important commands like generating coverage reports and initializing the repository.
