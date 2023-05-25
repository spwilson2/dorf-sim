using Godot;
using System;
using System.Collections.Generic;

public partial class UDPServerNode : Node
{
    private UdpServer _server = new UdpServer();
    private List<PacketPeerUdp> _peers  = new List<PacketPeerUdp>();

	private Area2D playerObject;

    public override void _Ready()
    {
        _server.Listen(6666, "127.0.0.1");
        playerObject = GetParent() as Area2D;
    }

    public override void _Process(double delta)
    {
        _server.Poll(); // Important!
        if (_server.IsConnectionAvailable())
        {
            PacketPeerUdp peer = _server.TakeConnection();
            byte[] packet = peer.GetPacket();
            GD.Print($"Accepted Peer: {peer.GetPacketIP()}:{peer.GetPacketPort()}");
            //GD.Print($"Received Data: {packet.GetStringFromUtf8()}");
            // Reply so it knows we received the message.
            //peer.PutPacket(packet);
            // Keep a reference so we can keep contacting the remote peer.
            _peers.Add(peer);
            var position = Game.Position.Parser.ParseFrom(packet);

            playerObject.Position = new Vector2(position.X, position.Y);
            GD.Print($"Moved to {position.X} {position.Y}");
        }
        foreach (var peer in _peers)
        {
            // Do something with the peers.
			peer.PutPacket("Moved".ToUtf32Buffer());
        }
    }
}
