# ktmpl

**ktmpl** is a tool for processing [Kubernetes](http://kubernetes.io/) manifest templates.
It is a very simple client-side implementation of the [Templates + Parameterization proposal](https://github.com/kubernetes/kubernetes/blob/master/docs/proposals/templates.md).

## Synopsis

```
ktmpl 0.1.0
Produces a Kubernetes manifest from a parameterized template

USAGE:
	ktmpl [FLAGS] [OPTIONS] <template> [--]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -p, --parameter <parameter>...    Key-value pair used to fill in the template's parameters in the format key=value

ARGS:
    <template>    Path to the template file to be processed
```

## Usage

Run `ktmpl TEMPLATE` where TEMPLATE is a path to a Kubernetes manifest template in YAML format.
The included [example.yml](example.yml) is a working example template.

To provide values for template parameters, use the `--parameter` option to supply key-value pairs.
Using the provided example.yml, this would mean:

``` bash
ktmpl example.yml --parameter MONGODB_PASSWORD=password
```

Template parameters that have default values can be overridden with the same mechanism.

The processed template will be output to stdout, suitable for piping into a `kubectl` command:

``` bash
ktmpl example.yml --parameter MONGODB_PASSWORD=password | kubectl create -f -
```

## Installing ktmpl

### Cargo

```
cargo install ktmpl
```

### Precompiled binary

[Releases](https://github.com/InQuicker/ktmpl/releases)

### Docker

```
docker pull inquicker/ktmpl
```

### Building from source

1. Install the appropriate version of [Rust](https://www.rust-lang.org/) for your system.
2. Run `git clone git@github.com:InQuicker/ktmpl.git`.
3. Inside the freshly cloned repository, run `cargo install --path .`.

Make sure Cargo's bin directory is added to your PATH environment variable.

## Development

To package the current release for distribution, update `TAG` in the Makefile and then run `make`.
Release artifacts will be written to the `dist` directory.
Your GPG secret key will be required to sign `sha256sums.txt`.

Docker images for `inquicker/ktmpl` and `inquicker/ktmpl:$TAG` will be created, but you must push them manually.
`cargo publish` must be run manually to release to crates.io.

## Legal

ktmpl is released under the MIT license. See `LICENSE` for details.
