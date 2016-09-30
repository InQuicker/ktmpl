use std::error::Error;

use yaml::{EmitError, Yaml, YamlEmitter, YamlLoader};

use parameter::{ParamMap, Parameter, ParameterValues};
use processor::process_yaml;

/// A Kubernetes manifest template and the values for each of its parameters.
pub struct Template {
    objects: Vec<Yaml>,
    param_map: ParamMap,
}

impl Template {
    /// Creates a new template.
    ///
    /// # Parameters
    ///
    /// * template_contents: The YAML template file's contents.
    /// * parameter_values: A map of the template's parameters and the user-supplied values for
    ///   each.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    ///
    /// * There was more than one YAML document present in the template contents.
    /// * The YAML document did not contain an "objects" key or it was not an array value.
    /// * The YAML document did not contain a "parameters" key or it was not an array value.
    /// * One of the parameters doesn't have a "name" key.
    /// * One of the parameters specifies an invalid "parameterType".
    /// * One of the parameters requires a value which wasn't supplied.
    pub fn new(template_contents: String, parameter_values: ParameterValues)
    -> Result<Self, String> {
        let docs = try!(YamlLoader::load_from_str(&template_contents)
            .map_err(|err| err.description().to_owned()));

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
            let parameter = try!(Parameter::new(parameter_spec, &parameter_values));

            param_map.insert(parameter.name.clone(), parameter);
        }

        Ok(Template {
            objects: template_objects,
            param_map: param_map,
        })
    }

    /// Interpolates the parameters' values into the YAML template, returning the results.
    ///
    /// # Errors
    ///
    /// Returns an error if the processed template was not valid YAML.
    pub fn process(mut self) -> Result<String, String> {
        for object in self.objects.iter_mut() {
            process_yaml(object, &self.param_map);
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
