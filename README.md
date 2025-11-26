# RRMockDecoder
Mock Race|Result decoder for TCP/IP connection based development

## Usage
Download the latest release from the [releases](https://github.com/gillian/rrmockdecoder/releases) page and run it with:

```bash
./rrmockdecoder --help
```

### Options

- `-t, --transponders <TRANSPONDERS>`: Number of unique random transponders to simulate [default: 10]
- `-i, --interval <INTERVAL>`: Interval between passings in seconds [default: 1.0]
- `-h, --help`: Print help
- `-V, --version`: Print version

### Example

Simulate 50 transponders with a passing every 0.5 seconds:

```bash
./rrmockdecoder --transponders 50 --interval 0.5
```
