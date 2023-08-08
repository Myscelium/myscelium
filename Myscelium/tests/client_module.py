from myscelium import MysceliumClient, ClientPatterns
import os
import time

client_patterns = ClientPatterns()

from threading import Event


class MyClient:
    events = {}  # shared events dictionary

    @staticmethod
    def store_event(event_key, event_obj):
        MyClient.events[event_key] = event_obj

    @classmethod
    def set_event(cls, key, event):
        cls.events[key] = event

    @staticmethod
    def get_event(event_key):
        event = MyClient.events.get(event_key, None)
        if not event:
            print(f"No event found for key: {event_key}")
        return event

    @staticmethod
    def test_handler(data):
        print("Received data: ", data)
        event = MyClient.get_event('main_event')
        if event:
            event.set()
        
        # This will stop the client
        MyClient.instance.stop() 

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
        
        client = self.initialize_client(event_key)
        
        # Store the client instance
        self.client_instance = client
        
        self.send_some_data(client, event_key)

    def stop(self):
        if hasattr(self, 'client_instance') and self.client_instance:
            self.client_instance.stop_client()  # assuming MysceliumClient has a stop() method
