# OwOtility

Configuration utility for [OwOpad firmware](https://github.com/chrontax/owopad).

For now it's only CLI, but there are plans for a gui.

## Installation
```
cargo install owotility
```

## Usage
```
Usage: owotility [COMMAND]

Commands:
  devices  List available devices
  configs  Show key configs
  binds    Manage currently set keybinds
  save     Save current configuration to flash
  send     Send something to the keypad
  status   Show live status of the keypad
  help     Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```