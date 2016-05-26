extern crate base64;
extern crate clap;
#[macro_use]
extern crate lazy_static;
extern crate regex;
extern crate yaml_rust as yaml;

use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::process::exit;

use clap::{App, AppSettings, Arg};

use parameter::user_values;
use template::Template;

mod parameter;
mod processor;
mod template;

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
        .setting(AppSettings::AllowExternalSubcommands)
        .arg(
            Arg::with_name("template")
                .help("Path to the template file to be processed")
                .required(true)
                .index(1)
        )
        .arg(
            Arg::with_name("parameter")
                .help("One or more key-value pairs used to fill in the template's parameters, \
                      formatted as: KEY=VALUE [KEY=VALUE ...]")
                .long("parameter")
                .short("p")
                .multiple(true)
                .takes_value(true)
        )
        .arg(
            Arg::with_name("base64-parameter")
                .help("Same as --parameter, but the value will first be encoded with Base64")
                .long("base64-parameter")
                .short("b")
                .value_name("parameter")
                .multiple(true)
                .takes_value(true)
        )
        .get_matches();

    let mut values = match matches.values_of("parameter") {
        Some(parameters) => try!(
            user_values(parameters.map(|s| s.to_string()).collect(), false)
        ),
        None => HashMap::new(),
    };

    if let Some(parameters_to_encode) = matches.values_of("base64-parameter") {
        let encoded_user_values = try!(
            user_values(parameters_to_encode.map(|s| s.to_string()).collect(), true)
        );

        values.extend(encoded_user_values);
    }

    let filename = matches.value_of("template").expect("template wasn't provided");
    let mut file = try!(File::open(filename).map_err(|err| err.description().to_owned()));
    let mut template_data = String::new();
    try!(file.read_to_string(&mut template_data).map_err(|err| err.description().to_owned()));

    let template = try!(Template::new(template_data, values));

    match template.process() {
        Ok(manifests) => {
            println!("{}", manifests);

            Ok(())
        }
        Err(error) => Err(error),
    }
}
