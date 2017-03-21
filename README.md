# ktmpl

**ktmpl** is a tool for processing [Kubernetes](http://kubernetes.io/) manifest templates.
It is a very simple client-side implementation of the [Templates + Parameterization proposal](https://github.com/kubernetes/kubernetes/blob/master/docs/proposals/templates.md).

## Synopsis

```
ktmpl 0.6.0
Produces a Kubernetes manifest from a parameterized template

USAGE:
    ktmpl [OPTIONS] <template>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -b, --base64-parameter <NAME> <VALUE>
            Same as --parameter, but for values already encoded in Base64
    -p, --parameter <NAME> <VALUE>
            Supplies a value for the named parameter
    -f, --parameter-file <PARAMETER_FILE>...
            Path to a YAML file with parameter values
    -s, --secret <NAME> <NAMESPACE>
            A secret to Base64 encode after parameter interpolation

ARGS:
    <template>    Path to the template file to be processed (use "-" to read from stdin)
```

## Usage

Run `ktmpl TEMPLATE` where TEMPLATE is a path to a Kubernetes manifest template in YAML format.
The included [example.yml](example.yml) is a working example template.

To provide values for template parameters, use the `--parameter`, `--base64-parameter` or `--parameter-file` options.

The included `example.yml` and `params.yml` files are used in the following examples.

Using the `--parameter` option:

``` bash
ktmpl example.yml --parameter MONGODB_PASSWORD secret
```

Using the `--parameter-file` option:

```
ktmpl example.yml --parameter-file params.yml
```

A parameter file must contain one or more YAML documents.
Each document must be a YAML hash with string keys and string values, corresponding to template
parameters and their values, respectively.

In parameter files, if the same parameter is defined more than once, the last defined value will be
used.
Parameter values supplied via the `--parameter` option will override any that are defined via
parameter files.

Template parameters that have default values can be overridden with the same mechanisms:

``` bash
ktmpl example.yml --parameter MONGODB_USER carl --parameter MONGODB_PASSWORD secret
```

The processed template will be output to stdout, suitable for piping into a `kubectl` command:

``` bash
ktmpl example.yml --parameter MONGODB_PASSWORD password | kubectl create -f -
```

If a parameter's `parameterType` is `base64`, the value passed for that parameter via `--parameter` will be Base64-encoded before being inserted into the template.
If the value passed via `--parameter` is already Base64-encoded, and hence encoding it again would be an error, use the `--base64-parameter` option instead:

``` bash
ktmpl example.yml --base64-parameter MONGODB_PASSWORD c2VjcmV0 | kubectl create -f -
```

This can be handy when working with Kubernetes secrets, or anywhere else binary or opaque data is needed.
Values provided via parameter files are always treated as plain strings, so the `--base64-parameter` option is required for values that are already Base64-encoded.

When working with Kubernetes secrets with values that _contain_ a parameter as well as literal text, such as a config file with passwords in it, the `--secret` option is useful.
For example, using the provided [secret_example.yml](secret_example.yml) template:

``` bash
ktmpl secret_example.yml --parameter PASSWORD narble --secret webapp default
```

"webapp" is the name of the secret and "default" is the namespace of the secret.
This will cause ktmpl to Base64 encode every value in the named secret's data hash after the normal parameter interpolation step.
When using the `--secret` option, you probably don't want to use `base64` as the `parameterType` for any parameters within the secret, since the entire value will be Base64 encoded after interpolation.
If a Kubernetes secret named with the `--secret` option is not found in the template, ktmpl will exit with an error.

It's also possible to supply the template via stdin instead of a named file by using `-` as the filename:

``` bash
cat example.yml | ktmpl - --parameter MONGODB_PASSWORD password | kubectl create -f -
```

## Installing ktmpl

### Homebrew

```
brew install ktmpl
```

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
