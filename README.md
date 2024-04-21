# CLI SNAP

cli-snap is a command-line interface (CLI) tool designed for testing other CLI applications using a snapshot-based testing strategy.

## Installation

Using **cargo**

```bash
cargo install cli-snap
```

## Usage

1. Create a test suite file named **cli-snap.toml** to define your testing commands:

```toml
[[tests]]
commands = ["echo 'Hello'"]
id = "hello-world"

[[tests]]
commands = ["echo 'test 2'", "echo 'second hello'"]
id = "test-2"

[config]
snapshot_directory = "./snaps"
```

| Ensure each test has a unique ID to distinguish and identify each snapshot.

2. Run the test suite using the following command:

```bash
cli-snap --config <directory where you have saved toml>
```

[output image](./docs/output.png)

3. To update snapshots, run the command.

```bash
cli-snap --config <directory where you have saved toml> --update-snapshot
```
