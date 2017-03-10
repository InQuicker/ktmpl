extern crate clap;
extern crate ktmpl;
extern crate yaml_rust as yaml;

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
        .arg(
            Arg::with_name("parameter-file")
                .help("Supplies a Yaml file defining any named parameters")
                .next_line_help(true)
                .long("param-file")
                .short("f")
                .multiple(true)
                .takes_value(true)
                .number_of_values(1)
                .value_names(&["FILENAME"])
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

    if let Some(files) = matches.values_of("parameter-file") {
      for file in files {
        let mut file_handle = try!(File::open(file).map_err(|err| err.description().to_owned()));
        let mut contents = String::new();
        file_handle.read_to_string(&mut contents).unwrap();
        let docs = yaml::YamlLoader::load_from_str(&contents).unwrap();
        for doc in &docs {
          let primary_key = "";
          let param_values = parse_yaml(doc, primary_key);
          values.extend(param_values);
        }
      }
    }

    let filename = matches.value_of("template").expect("template wasn't provided");
    let mut template_data = String::new();

    if filename == "-" {
        try!(
            stdin().read_to_string(&mut template_data).map_err(|err| err.description().to_owned())
        );
    } else {
        let mut file = try!(File::open(filename).map_err(|err| err.description().to_owned()));
        try!(file.read_to_string(&mut template_data).map_err(|err| err.description().to_owned()));
    }

    let template = try!(Template::new(template_data, values));

    match template.process() {
        Ok(manifests) => {
            println!("{}", manifests);

            Ok(())
        }
        Err(error) => Err(error),
    }
}

fn parse_yaml(doc: &yaml::Yaml, primary_key: &str) -> ParameterValues {
  let mut param_values = ParameterValues::new();

  match doc {
    &yaml::Yaml::Hash(ref h) => {
      for (key, value) in h {
        let combined_key = primary_key.to_string() + key.as_str().unwrap();
        match value {
          &yaml::Yaml::Hash(ref h) => {
            // Found a nested hash of values, prepend the parent key to each sub-key with '_'
            // For example:
            // ---
            // MONGODB:
            //  USER: mongodb
            //  PASSWORD: secret
            //
            // Produces:
            // MONGODB_USER: mongodb
            // MONGODB_PASSWROD: secret
            //
            for (key, value) in h {
              let sub_key = combined_key.to_string() + "_" + key.as_str().unwrap();
              let sub_values = parse_yaml(value, &sub_key);
              param_values.extend(sub_values);
            }
          },
          &yaml::Yaml::String(ref s) => {
            let pv = ParameterValue::Plain(s.to_string());
            param_values.insert(combined_key,pv);
          },
          &yaml::Yaml::Integer(ref i) => {
            let pv = ParameterValue::Plain(i.to_string());
            param_values.insert(combined_key,pv);
          },
          &yaml::Yaml::Real(ref r) => {
            let pv = ParameterValue::Plain(r.to_string());
            param_values.insert(combined_key,pv);
          },
          &yaml::Yaml::Boolean(ref b) => {
            let pv = ParameterValue::Plain(b.to_string());
            param_values.insert(combined_key,pv);
          },
          _ => {
            // Value type not currently supported
            // Currently unsuported types include:
            // Array, Alias and None
          }
        }
      }
    },
    &yaml::Yaml::String(ref s) => {
      let pv = ParameterValue::Plain(s.to_string());
      param_values.insert(primary_key.to_string(),pv);
    },
    _ => {
      // Key type not convered - document
    }
  }
  param_values
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
