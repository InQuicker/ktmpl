use std::error::Error;

use yaml::{EmitError, Yaml, YamlEmitter, YamlLoader};

use parameter::Parameter;

pub struct Template {
    pub objects: Vec<Yaml>,
    pub parameters: Vec<Parameter>,
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
            template_parameters.push(try!(Parameter::from_yaml(parameter)));
        }

        Ok(Template {
            objects: template_objects,
            parameters: template_parameters,
        })
    }

    pub fn process(self) -> Result<String, String> {
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

        Ok(manifests)
    }
}
