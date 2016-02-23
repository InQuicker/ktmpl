# ktmpl

**ktmpl** is a tool for processing [Kubernetes](http://kubernetes.io/) manifest templates.
It is a very simple client-side implementation of the [Templates + Parameterization proposal](https://github.com/kubernetes/kubernetes/blob/master/docs/proposals/templates.md).

## Synopsis

```
ktmpl 0.1.0
Produces a Kubernetes manifest from a parameterized template

USAGE:
	ktmpl [FLAGS] <template>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

ARGS:
    <template>    Path to the template file to be processed
```

## Usage

Simply run `ktmpl TEMPLATE` where TEMPLATE is a path to a Kubernetes manifest template in YAML format.
The included [example.yml](example.yml) is a working example template.

To provide values for template parameters, simply execute the program with matching environment variables set.
Using the provided example.yml, this would mean:

``` bash
MONGODB_PASSWORD=password ktmpl example.yml
```

Template parameters that have default values can be overridden with the same mechanism.

The processed template will be output to stdout, suitable for piping into a `kubectl` command:

``` bash
MONGODB_PASSWORD=password ktmpl example.yml | kubectl create -f -
```

## Installing ktmpl

### Building from source

1. Install the appropriate version of [Rust](https://www.rust-lang.org/) for your system.
2. Run `git clone git@github.com:InQuicker/ktmpl.git`.
3. Inside the freshly cloned repository, run `cargo build --release`.
4. Copy the binary from `target/release/ktmpl` to a directory in your PATH, such as `/usr/local/bin`.

## Legal

ktmpl is released under the MIT license. See `LICENSE` for details.
