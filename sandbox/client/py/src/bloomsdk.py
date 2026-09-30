# Bloom SDK
CLIENT_VERSION = (0, 1, 0)
PROTOCOL_VERSION = (1, 0, 0)


class App:
    def __init__(self, app_name):
        pass
        client_version = bytes(CLIENT_VERSION)
        protocol_version = bytes(PROTOCOL_VERSION)
        client_id = b"ASLDASDA"
        app_name = "my-app".encode("UTF-8")
        app_name = app_name.ljust(32, b"\0")[:32]
        padding = b"\0"
