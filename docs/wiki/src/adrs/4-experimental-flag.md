# 4: Experimental flag

There is a application level argument (flag) `experimental` (`-e` | `--experimental`) that indicates that experimental features can now be accessed. This flag explicitly marks features that are NOT part of the official public API and therefore NOT considered when applying the versioning scheme (see link:/complate/docs/adrs/3-versioning[ADR 3]). +
This flag is designed to be used with and therefore CAN be used with link:/complate/docs/adrs/5-feature-flags[feature Flags as specified in ADR 5].
