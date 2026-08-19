import socket

sock = socket.create_connection(("127.0.0.1", 31415))
print("Connected!")

# Connection Request Packet -> 44 bytes
# : Client Version -> [major, minor, patch] 3 bytes
# : ID ->             8 bytes (8 random characters)
# : App name       -> 32 bytes [UTF-8 encoded string, NUL-padded]
# : NUL-Padding    -> 1 byte


version = bytes([0, 1, 0])
client_id = b"ABCDEFGH"
app_name = "my-app".encode("UTF-8")
app_name = app_name.ljust(32, b"\0")[:32]
padding = b"\0"

connection_packet = version + client_id + app_name + padding

print(len(connection_packet))

sock.send(b"Hello, World!")
