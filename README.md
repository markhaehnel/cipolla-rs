# cipolla-rs

[![Latest version](https://img.shields.io/github/v/release/markhaehnel/cipolla-rs)](https://github.com/markhaehnel/cipolla-rs/releases/latest)
[![GitHub workflow status](https://github.com/markhaehnel/cipolla-rs/actions/workflows/ci.yaml/badge.svg)](https://github.com/markhaehnel/cipolla-rs/actions/workflows/ci.yaml)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](#license)

> 🚧 **WORK IN PROGRESS** 🚧
>
> This tool is still in development and not ready for production use.
> Breaking changes may occur at any time.

Runs multiple http proxies that forward requests to tor.

## Usage

Download the latest release from the [releases page](https://github.com/markhaehnel/cipolla-rs/releases).

```bash
$ cipolla --help
Usage: cipolla [OPTIONS]

Options:
  -p, --port <PORT>                  Starting port to listen on [default: 8080]
  -c, --count <COUNT>                Number of ports to listen on (port + count) [default: 10]
  -e, --exit-country <EXIT_COUNTRY>  Country code to use for exit node
  -h, --help                         Print help
  -V, --version                      Print version
```

## Contributing

See the [contributing guidelines](./CONTRIBUTING.md) for more information.

## License

This code is licensed under either of

- [MIT License](./LICENSE-MIT)
- [Apache-2.0 License](./LICENSE-APACHE)

at your option.

## Disclaimer

This project is not officially associated with the Tor Project.