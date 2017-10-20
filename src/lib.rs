//! the [Templates + Parameterization proposal](https://github.com/kubernetes/kubernetes/blob/master/docs/proposals/templates.md).
//!
//! The crate ships with a command line utility and a library for programmatically processing
//! manifest templates. See the [README](https://github.com/InQuicker/ktmpl) for documentation for
//! the command line utility, and a general overview of the Kubernetes template system.
//!
//! Here is a simple example of using the library:
//!
//! ```
//! extern crate ktmpl;
//!
//! use ktmpl::{ParameterValue, ParameterValues, Template};
//!
//! fn main() {
//!     let template_contents = r#"
//! ---
//! kind: "Template"
//! apiVersion: "v1"
//! metadata:
//!   name: "example"
//! objects:
//!   - kind: "Service"
//!     apiVersion: "v1"
//!     metadata:
//!       name: "$(DATABASE_SERVICE_NAME)"
//!     spec:
//!       ports:
//!         - name: "db"
//!           protocol: "TCP"
//!           targetPort: 3000
//!       selector:
//!         name: "$(DATABASE_SERVICE_NAME)"
//! parameters:
//!   - name: "DATABASE_SERVICE_NAME"
//!     description: "Database service name"
//!     required: true
//!     parameterType: "string"
//! "#;
//!
//!     let mut parameter_values = ParameterValues::new();
//!
//!     parameter_values.insert(
//!         "DATABASE_SERVICE_NAME".to_string(),
//!         ParameterValue::Plain("mongo".to_string()),
//!     );
//!
//!     let template = Template::new(
//!         template_contents.to_string(),
//!         parameter_values,
//!         None,
//!     ).unwrap();
//!     let processed_template = template.process().unwrap();
//!
//!     assert_eq!(
//!         processed_template.lines().map(|l| l.trim_right()).collect::<Vec<&str>>().join("\n"),
//!         r#"---
//! apiVersion: v1
//! kind: Service
//! metadata:
//!   name: mongo
//! spec:
//!   ports:
//!     -
//!       name: db
//!       protocol: TCP
//!       targetPort: 3000
//!   selector:
//!     name: mongo"#
//!     );
//! }
//! ```

#![deny(missing_debug_implementations)]
#![deny(missing_docs)]

extern crate base64;
#[macro_use] extern crate lazy_static;
extern crate regex;
extern crate yaml_rust as yaml;

pub use template::Template;
pub use parameter::{
    ParameterValue,
    ParameterValues,
    parameter_values_from_file,
    parameter_values_from_str,
    parameter_values_from_yaml,
    parameter_values_from_env,
};
pub use secret::{Secret, Secrets};

mod parameter;
mod processor;
mod secret;
mod template;

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::Read;

    use super::{
        ParameterValue,
        ParameterValues,
        Secret,
        Secrets,
        Template,
        parameter_values_from_file,
        parameter_values_from_env,
    };

    #[test]
    fn encode_secrets() {
        let template_contents = r#"
---
kind: "Template"
apiVersion: "v1"
metadata:
  name: "example"
objects:
  - kind: "Secret"
    apiVersion: "v1"
    metadata:
      name: "webapp"
    data:
      config.yml: |
        username: "carl"
        password: "$(PASSWORD)"
    type: "Opaque"
parameters:
  - name: "PASSWORD"
    description: "The password for the web app"
    required: true
    parameterType: "string"
"#;

        let mut parameter_values = ParameterValues::new();

        parameter_values.insert(
            "PASSWORD".to_string(),
            ParameterValue::Plain("narble".to_string()),
        );

        let mut secrets = Secrets::new();

        secrets.insert(Secret {
            name: "webapp".to_string(),
            namespace: "default".to_string(),
        });

        let template = Template::new(
            template_contents.to_string(),
            parameter_values,
            Some(secrets),
        ).unwrap();

        let processed_template = template.process().unwrap();

        assert_eq!(
            processed_template.lines().map(|l| l.trim_right()).collect::<Vec<&str>>().join("\n"),
            r#"---
apiVersion: v1
data:
  config.yml: dXNlcm5hbWU6ICJjYXJsIgpwYXNzd29yZDogIm5hcmJsZSIK
kind: Secret
metadata:
  name: webapp
type: Opaque"#
        );
    }

    #[test]
    fn missing_secret() {
        let template_contents = r#"
---
kind: "Template"
apiVersion: "v1"
metadata:
  name: "example"
objects:
  - kind: "Namespace"
    apiVersion: "v1"
    metadata:
      name: "$(NAMESPACE)"
parameters:
  - name: "NAMESPACE"
    description: "The namespace to create"
    required: true
    parameterType: "string"
"#;

        let mut parameter_values = ParameterValues::new();

        parameter_values.insert(
            "NAMESPACE".to_string(),
            ParameterValue::Plain("foo".to_string()),
        );

        let mut secrets = Secrets::new();

        secrets.insert(Secret {
            name: "ghost".to_string(),
            namespace: "default".to_string(),
        });

        let template = Template::new(
            template_contents.to_string(),
            parameter_values,
            Some(secrets),
        ).unwrap();

        assert!(template.process().is_err());
    }

    #[test]
    fn parameter_file() {
        let mut template_file = File::open("example.yml").unwrap();
        let mut template_contents = String::new();

        template_file.read_to_string(&mut template_contents).unwrap();

        let parameter_values = parameter_values_from_file("params.yml").unwrap();

        let template = Template::new(
            template_contents.to_string(),
            parameter_values,
            None,
        ).unwrap();

        let processed_template = template.process().unwrap();

        assert_eq!(
            processed_template.lines().map(|l| l.trim_right()).collect::<Vec<&str>>().join("\n"),
            r#"---
apiVersion: v1
kind: Service
metadata:
  name: mongodb
spec:
  ports:
    -
      name: mongo
      protocol: TCP
      targetPort: 27017
  selector:
    name: mongodb
---
apiVersion: v1
kind: ReplicationController
metadata:
  name: mongodb
spec:
  replicas: 2
  selector:
    name: mongodb
  template:
    metadata:
      creationTimestamp: ~
      labels:
        name: mongodb
    spec:
      containers:
        -
          env:
            -
              name: MONGODB_USER
              value: carl
            -
              name: MONGODB_PASSWORD
              value: c2VjcmV0
            -
              name: MONGODB_DATABASE
              value: sampledb
          image: "docker.io/centos/mongodb-26-centos7"
          name: mongodb
          ports:
            -
              containerPort: 27017
              protocol: TCP"#
        );
    }
}
