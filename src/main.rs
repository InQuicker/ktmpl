extern crate clap;
extern crate ktmpl;

use std::collections::{HashMap};
use std::error::Error;
use std::fs::File;
use std::io::{Read, stdin};
use std::process::exit;

use clap::{App, AppSettings, Arg, Values};

use ktmpl::{Template, ParameterValue, ParameterValues, ParameterFile, Secret, Secrets};

fn main() {
    if let Err(error) = real_main() {
        println!("Error: {}", error);

        exit(1);
    }
}

fn real_main() -> Result<(), String> {
    let matches = App::new("ktmpl")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Produces a Kubernetes manifest from a parameterized template")
        .setting(AppSettings::ArgRequiredElseHelp)
        .setting(AppSettings::AllowLeadingHyphen)
        .arg(
            Arg::with_name("template")
                .help("Path to the template file to be processed (use \"-\" to read from stdin)")
                .required(true)
                .index(1)
        )
        .arg(
            Arg::with_name("parameter")
                .help("Supplies a value for the named parameter")
                .next_line_help(true)
                .long("parameter")
                .short("p")
                .multiple(true)
                .takes_value(true)
                .number_of_values(2)
                .value_names(&["NAME", "VALUE"])
        )
        .arg(
            Arg::with_name("base64-parameter")
                .help("Same as --parameter, but for values already encoded in Base64")
                .next_line_help(true)
                .long("base64-parameter")
                .short("b")
                .multiple(true)
                .takes_value(true)
                .number_of_values(2)
                .value_names(&["NAME", "VALUE"])
        )
        .arg(
            Arg::with_name("secret")
                .help("A secret to Base64 encode after parameter interpolation")
                .next_line_help(true)
                .long("secret")
                .short("s")
                .multiple(true)
                .takes_value(true)
                .number_of_values(2)
                .value_names(&["NAME", "NAMESPACE"])
        )
        .arg(
            Arg::with_name("parameter-file")
                .help("Supplies a Yaml file defining any named parameters")
                .next_line_help(true)
                .long("parameter-file")
                .short("f")
                .multiple(true)
                .takes_value(true)
                .number_of_values(1)
                .value_names(&["FILENAME"])
        )
        .get_matches();

    let mut values = HashMap::new();

    // Parse Parameter files first, passing command line parameters
    // should override any values supplied via a file
    if let Some(files) = matches.values_of("parameter-file") {
        let params_from_file = parameter_files(files);
        values.extend(params_from_file);
    }

    if let Some(parameters) = matches.values_of("parameter") {
        values.extend(parameter_values(parameters, false));
    }

    if let Some(parameters) = matches.values_of("base64-parameter") {
        let encoded_values = parameter_values(parameters, true);

        values.extend(encoded_values);
    }

    let secrets = matches
        .values_of("secret")
        .and_then(|secrets| Some(secret_values(secrets)))
        .or(None);

    let filename = matches.value_of("template").expect("template wasn't provided");
    let mut template_data = String::new();

    if filename == "-" {
        stdin().read_to_string(&mut template_data).map_err(|err| err.description().to_owned())?;
    } else {
        let mut file = File::open(filename).map_err(|err| err.description().to_owned())?;
        file.read_to_string(&mut template_data).map_err(|err| err.description().to_owned())?;
    }

    let template = Template::new(template_data, values, secrets)?;

    match template.process() {
        Ok(manifests) => {
            println!("{}", manifests);

            Ok(())
        }
        Err(error) => Err(error),
    }
}

fn parameter_files(mut param_files: Values) -> ParameterValues {
    let mut parameter_values = ParameterValues::new();

    loop {
        if let Some(f) = param_files.next() {
            let param_file = ParameterFile::from_file(&f).unwrap();
            parameter_values.extend(param_file.parameters);
        } else {
            break;
        }
    }
    parameter_values
}

fn parameter_values(mut parameters: Values, base64_encoded: bool) -> ParameterValues {
    let mut parameter_values = ParameterValues::new();

    loop {
        if let Some(name) = parameters.next() {
            let value = parameters.next().expect("Parameter was missing its value.");

            let parameter_value = if base64_encoded {
                ParameterValue::Encoded(value.to_string())
            } else {
                ParameterValue::Plain(value.to_string())
            };

            parameter_values.insert(name.to_string(), parameter_value);
        } else {
            break;
        }
    }

    parameter_values
}

fn secret_values(mut secret_parameters: Values) -> Secrets {
    let mut secrets = Secrets::new();

    loop {
        if let Some(name) = secret_parameters.next() {
            let namespace = secret_parameters.next().expect("Secret was missing its namespace.");

            secrets.insert(Secret {
                name: name.to_string(),
                namespace: namespace.to_string(),
            });
        } else {
            break;
        }
    }

    secrets
}
