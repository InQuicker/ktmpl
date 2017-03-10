extern crate clap;
extern crate ktmpl;

use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{Read, stdin};
use std::process::exit;

use clap::{App, AppSettings, Arg, Values};

use ktmpl::{Template, ParameterValue, ParameterValues};

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
        .get_matches();

    let mut values = match matches.values_of("parameter") {
        Some(parameters) => parameter_values(parameters, false),
        None => HashMap::new(),
    };

    if let Some(parameters) = matches.values_of("base64-parameter") {
        let encoded_values = parameter_values(parameters, true);

        values.extend(encoded_values);
    }

    let filename = matches.value_of("template").expect("template wasn't provided");
    let mut template_data = String::new();

    if filename == "-" {
        stdin().read_to_string(&mut template_data).map_err(|err| err.description().to_owned())?;
    } else {
        let mut file = File::open(filename).map_err(|err| err.description().to_owned())?;
        file.read_to_string(&mut template_data).map_err(|err| err.description().to_owned())?;
    }

    let template = Template::new(template_data, values)?;

    match template.process() {
        Ok(manifests) => {
            println!("{}", manifests);

            Ok(())
        }
        Err(error) => Err(error),
    }
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
