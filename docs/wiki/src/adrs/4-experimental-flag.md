# 4-experimental-flag

## Summary

There is a application level argument (flag) `experimental` (`-e` | `--experimental`) that indicates that experimental features can now be accessed. This flag explicitly marks features that are NOT part of the official public API and therefore NOT considered when applying the versioning scheme (see [ADR 3](./3.md)).\
This flag is designed to be used with and therefore CAN be used with [feature flags](./5.md).
