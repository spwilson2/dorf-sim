extern crate prost_build;

// /// A service descriptor.
// #[derive(Debug)]
// pub struct Service {
//     /// The service name in Rust style.
//     pub name: String,
//     /// The service name as it appears in the .proto file.
//     pub proto_name: String,
//     /// The package name as it appears in the .proto file.
//     pub package: String,
//     /// The service comments.
//     pub comments: Comments,
//     /// The service methods.
//     pub methods: Vec<Method>,
//     /// The service options.
//     pub options: prost_types::ServiceOptions,
// }
//
// /// A service method descriptor.
// #[derive(Debug)]
// pub struct Method {
//     /// The name of the method in Rust style.
//     pub name: String,
//     /// The name of the method as it appears in the .proto file.
//     pub proto_name: String,
//     /// The method comments.
//     pub comments: Comments,
//     /// The input Rust type.
//     pub input_type: String,
//     /// The output Rust type.
//     pub output_type: String,
//     /// The input Protobuf type.
//     pub input_proto_type: String,
//     /// The output Protobuf type.
//     pub output_proto_type: String,
//     /// The method options.
//     pub options: prost_types::MethodOptions,
//     /// Identifies if client streams multiple client messages.
//     pub client_streaming: bool,
//     /// Identifies if server streams multiple server messages.
//     pub server_streaming: bool,
// }

use convert_case::{Case, Casing};
use prost_build::{Service, ServiceGenerator};

struct MyServiceGen;

impl ServiceGenerator for MyServiceGen {
    fn generate(&mut self, service: Service, _buf: &mut String) {
        _buf.push_str(format!("pub trait {} {{\n", service.name).as_str());

        for meth in service.methods {
            _buf.push_str(
                format!(
                    "fn {}({}: {}) -> {}; \n",
                    meth.name,
                    meth.input_type.to_case(Case::Camel),
                    meth.input_type,
                    meth.output_type
                )
                .as_str(),
            );
        }
        // End service trait.
        _buf.push_str("\n}");
    }

    fn finalize(&mut self, _buf: &mut String) {}
}

fn main() {
    let srcs = ["game.proto"];
    let proto_paths = "../godot-frontend/protos";
    prost_build::compile_protos(&srcs, &[proto_paths]).unwrap();

    prost_build::Config::new()
        .service_generator(Box::new(MyServiceGen))
        .compile_protos(&srcs, &[proto_paths])
        .unwrap();

    for src in srcs {
        println!("cargo:rerun-if-changed={}/{}", proto_paths, src);
    }
}
