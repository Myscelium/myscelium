from myscelium import MysceliumClient, ClientPatterns
import os
import time
from threading import Event

client_patterns = ClientPatterns()

def test_handler(data, event):
    print("Received data: ", data)
    event.set()
    return None

def send_some_data(mys_client, event):
    mys_client.running = True
    time.sleep(10)
    command = client_patterns.command_pattern("python_function", args={"age":10, "birth":8, "name":"cristian"})
    result = mys_client.send(command, priority=10)
    print(result)

def initialize_client(event):
    mys_client = MysceliumClient(client_uid="some_client_id", buffer_path="ClientData/")
    mys_client.set_client_uid(client_uid="some_client_id")

    callbacks = [
        client_patterns.callback_pattern(callback=lambda data: test_handler(data, event), args={
            "data": "dict"
        }),
    ]
    mys_client.set_callbacks(callbacks=callbacks)
    mys_client.set_workers_num(n_workers=2)
    mys_client.initialize_client("127.0.0.1",4444)
    return mys_client

def run_client(event):
    client = initialize_client(event)
    send_some_data(client, event)
