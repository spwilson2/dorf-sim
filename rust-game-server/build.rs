extern crate prost_build;

fn main() {
    let srcs = ["game.proto"];
    let proto_paths = "../godot-frontend/protos";
    prost_build::compile_protos(&srcs, &[proto_paths]).unwrap();

    for src in srcs {
        println!("cargo:rerun-if-changed={}/{}", proto_paths, src);
    }
}
