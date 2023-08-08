from myscelium import MysceliumClient, ClientPatterns
import os
import time

client_patterns = ClientPatterns()

from threading import Event


class MyClient:
    events = {}  # shared events dictionary

    @classmethod
    def set_event(cls, key, event):
        cls.events[key] = event

    @classmethod
    def get_event(cls, key):
        return cls.events.get(key)

    @staticmethod
    def test_handler(data):
        print("Received data: ", data)
        event = MyClient.get_event('main_event')
        if event:
            event.set()

    @staticmethod
    def send_some_data(mys_client):
        mys_client.running = True
        time.sleep(10)
        command = client_patterns.command_pattern("python_function", args={"age": 10, "birth": 8, "name": "cristian"})
        result = mys_client.send(command, priority=10)
        print(result)

    def initialize_client(self, event_key):
        mys_client = MysceliumClient(client_uid="some_client_id", buffer_path="ClientData/")
        mys_client.set_client_uid(client_uid="some_client_id")

        callbacks = [
            client_patterns.callback_pattern(callback=MyClient.test_handler, args={
                "data": "dict"
            }),
        ]
        mys_client.set_callbacks(callbacks=callbacks)
        mys_client.set_workers_num(n_workers=2)
        mys_client.initialize_client("127.0.0.1", 4444)
        return mys_client

    def run(self, event_key):
        MyClient.set_event(event_key, Event())
        self.client = self.initialize_client(event_key)
        MyClient.send_some_data(self.client)