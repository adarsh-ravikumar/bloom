import socket
import struct
import json
import datetime

sock = socket.create_connection(("127.0.0.1", 31415))
print("Established Connection", flush=True)


version = bytes([0, 1, 0])
client_id = b"ASLDASDA"
app_name = "my-app".encode("UTF-8")
app_name = app_name.ljust(32, b"\0")[:32]
padding = b"\0"

connection_packet = version + client_id + app_name + padding

print("Sending client information", flush=True)
sock.send(connection_packet)

count = 0
while True:
    msg_len_bytes = sock.recv(4)
    # msg now holds packet length
    msg_len = int.from_bytes(msg_len_bytes, byteorder="big")
    print(f"Recieveing size: {msg_len}", flush=True)

    packet = sock.recv(msg_len)
    print(f"Packet isss: {packet}", flush=True)

    packet_data = json.loads(packet)

    if packet_data["event"] == "button_pressed":
        inc_command = {
            "command": "increment",
            "data": {}
        }

        command_ser = bytes(json.dumps(inc_command), 'utf-8')

        payload = command_ser.encode(
            'utf-8') if isinstance(command_ser, str) else command_ser
        length_header = struct.pack('!I', len(payload))

        print(f"Sending command of length {len(payload)}", flush=True)
        sock.send(length_header)

        count += 1
        print(f"Sending payload [predicted count: {
              count} [{datetime.datetime.now()}]", flush=True)
        sock.send(payload)
