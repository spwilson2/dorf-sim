use bevy::{
    ecs::{component::TableStorage, storage::SparseSet},
    prelude::Component,
};
use prost::Message;
use std::net::{Ipv4Addr, SocketAddr, UdpSocket};
//use bevy::{prelude::*, ecs::component::ComponentStorage, ecs::component::Component::sealed::Sealed};

// Include the `items` module, which is generated from items.proto.
pub mod game {
    include!(concat!(env!("OUT_DIR"), "/game.rs"));
}

//#[derive(Component)]
struct J {}

//type Storage = #storage;
impl Component for J {
    type Storage = TableStorage;
}

fn main() -> std::io::Result<()> {
    {
        let socket = UdpSocket::bind("127.0.0.1:5555")?;
        let pos = game::Position::default();
        let pos2 = game::Position { x: 100, y: 100 };

        // Receives a single datagram message on the socket. If `buf` is too small to hold
        // the message, it will be cut off.
        let mut buf = [0; 10];
        //let (amt, src) = socket.recv_from(&mut buf)?;

        // Redeclare `buf` as slice of the received data and send reverse data back to origin.
        //let buf = &mut buf[..amt];
        //buf.reverse();
        let mut buf = Vec::new();
        pos2.encode(&mut buf);
        socket.send_to(&buf, "127.0.0.1:6666")?;
    } // the socket is closed here
    Ok(())
}
