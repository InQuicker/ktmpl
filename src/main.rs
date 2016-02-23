extern crate clap;
extern crate yaml_rust as yaml;

use std::env::{VarError, var};
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::process::exit;

use clap::{App, AppSettings, Arg};
use yaml::{EmitError, Yaml, YamlEmitter, YamlLoader};


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
        .get_matches();

    let filename = matches.value_of("template").expect("template wasn't provided");
    let mut file = try!(File::open(filename).map_err(|err| err.description().to_owned()));
    let mut template_data = String::new();
    try!(file.read_to_string(&mut template_data).map_err(|err| err.description().to_owned()));

    let template = try!(Template::from_string(template_data));

    match template.process() {
        Ok(manifests) => {
            println!("{}", manifests);

            Ok(())
        }
        Err(error) => Err(error),
    }
}

struct Template {
    pub objects: Vec<Yaml>,
    pub parameters: Vec<Parameter>,
}

struct Parameter {
    pub description: String,
    pub name: String,
    pub value: Option<String>,
}

impl Template {
    pub fn from_string(string: String) -> Result<Self, String> {
        let docs = try!(YamlLoader::load_from_str(&string).map_err(|err| err.description().to_owned()));

        if docs.len() != 1 {
            return Err("Only one YAML document can be present in the template.".to_owned());
        }

        let doc = &docs[0];

        let mut template_objects = vec![];
        let objects = match doc["objects"].as_vec() {
            Some(objects) => objects,
            None => return Err("Key \"objects\" must be present and must be an array.".to_owned())
        };

        for object in objects {
            template_objects.push(object.clone());
        }

        let mut template_parameters = vec![];
        let parameters = match doc["parameters"].as_vec() {
            Some(parameters) => parameters,
            None => return Err("Key \"parameters\" must be present and must be an array.".to_owned())
        };

        for parameter in parameters {
            template_parameters.push(Parameter {
                description: match parameter["description"].as_str() {
                    Some(description) => description.to_string(),
                    None => return Err(
                        "Parameters must have a \"description\" field and its value must be a string.".to_owned()
                    ),
                },
                name: match parameter["name"].as_str() {
                    Some(name) => name.to_string(),
                    None => return Err(
                        "Parameters must have a \"name\" field and its value must be a string.".to_owned()
                    ),
                },
                value: parameter["value"].as_str().map(|value| value.to_string()),
            });
        }

        Ok(Template {
            objects: template_objects,
            parameters: template_parameters,
        })
    }

    fn process(&self) -> Result<String, String> {
        let mut manifests = String::new();

        for object in self.objects.iter() {
            let mut emitter = YamlEmitter::new(&mut manifests);
            try!(emitter.dump(&object).map_err(|error| {
                match error {
                    EmitError::FmtError(error) => format!("{}", error),
                    EmitError::BadHashmapKey => "Bad hashmap key in YAML structure.".to_owned(),
                }
            }));
        }

        for parameter in self.parameters.iter() {
            let value = match var(&parameter.name) {
                Ok(value) => format!("{}", value),
                Err(error) => match error {
                    VarError::NotPresent => match parameter.value {
                        Some(ref value) => format!("{}", value),
                        None => return Err(
                            format!("Parameter \"{}\" is required.", parameter.name)
                        ),
                    },
                    VarError::NotUnicode(_) => return Err(error.description().to_owned()),
                }
            };

            manifests = manifests.replace(&format!("\"$(({}))\"", parameter.name)[..], &value);
            manifests = manifests.replace(
                &format!("\"$({})\"", parameter.name)[..],
                &format!("\"{}\"", &value)[..],
            );
        }

        Ok(manifests)
    }
}
