use std::error::Error;

use yaml::{EmitError, Yaml, YamlEmitter, YamlLoader};

use parameter::{ParamMap, Parameter, UserValues};
use processor::process_yaml;

pub struct Template {
    pub objects: Vec<Yaml>,
    pub param_map: ParamMap,
}

impl Template {
    pub fn new(string: String, user_values: UserValues) -> Result<Self, String> {
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

        let mut param_map = ParamMap::new();
        let parameter_specs = match doc["parameters"].as_vec() {
            Some(parameter_specs) => parameter_specs,
            None => return Err("Key \"parameters\" must be present and must be an array.".to_owned())
        };

        for parameter_spec in parameter_specs {
            let parameter = try!(Parameter::new(parameter_spec, &user_values));

            param_map.insert(parameter.name.clone(), parameter);
        }

        Ok(Template {
            objects: template_objects,
            param_map: param_map,
        })
    }

    pub fn process(mut self) -> Result<String, String> {
        for object in self.objects.iter_mut() {
            try!(process_yaml(object, &self.param_map));
        }

        dump(self.objects)
    }
}

fn dump(objects: Vec<Yaml>) -> Result<String, String> {
    let mut manifests = String::new();
    let last = objects.len() - 1;

    for (i, object) in objects.iter().enumerate() {
        {
            let mut emitter = YamlEmitter::new(&mut manifests);
            try!(emitter.dump(&object).map_err(|error| {
                match error {
                    EmitError::FmtError(error) => format!("{}", error),
                    EmitError::BadHashmapKey => "Bad hashmap key in YAML structure.".to_owned(),
                }
            }));
        }

        if i != last {
            manifests.push_str("\n");
        }
    }

    Ok(manifests)
}
