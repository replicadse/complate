# 5-usage-of-feature-flags

## Summary

This project makes use of cargo feature flags. Feature flags count as part of the public API and are therefore to be considered when applying the version rules IF NOT marked as experimental (see [ADR 4](./4.md)).\
All feature flags MUST be documented in an appropriate manner in the README.md file.
