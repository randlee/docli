# demo CLI Reference

Version 1.0.0

Generated from the CLI command tree. Do not hand-edit.

## `demo`

A slightly longer description of the demo tool that spans a sentence.

```text
demo [OPTIONS] <COMMAND>
```

**Arguments:**

- `config` — Path to the config file

**Options:**

- `--verbose, -v` — Increase verbosity
- `--output, -o <PATH>` — choices: `json, yaml` — Write output to PATH

See the full docs for more detail.

### `demo run`

Run the thing

```text
demo run [OPTIONS]
```

**Options:**

- `--dry-run` — default: `false` — Do not execute

#### `demo run once`

Run exactly once

```text
demo run once
```

### `demo check`

Check the config

```text
demo check <CONFIG>
```

**Arguments:**

- `config` *(required)* — Config to check

