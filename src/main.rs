extern crate clap;
extern crate yaml_rust as yaml;

use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::process::exit;

use clap::{App, AppSettings, Arg};

use parameter::param_map;
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
                .help("Key-value pair used to fill in the template's parameters in the format key=value")
                .long("parameter")
                .short("p")
                .multiple(true)
                .takes_value(true)
        )
        .get_matches();

    let filename = matches.value_of("template").expect("template wasn't provided");
    let mut file = try!(File::open(filename).map_err(|err| err.description().to_owned()));
    let mut template_data = String::new();
    try!(file.read_to_string(&mut template_data).map_err(|err| err.description().to_owned()));

    let template = try!(Template::from_string(template_data));
    let parameters = match matches.values_of("parameter") {
        Some(parameters) => try!(param_map(parameters.map(|s| s.to_string()).collect())),
        None => HashMap::new(),
    };

    match template.process(parameters) {
        Ok(manifests) => {
            println!("{}", manifests);

            Ok(())
        }
        Err(error) => Err(error),
    }
}
