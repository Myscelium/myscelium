from myscelium import MysceliumClient, ClientPatterns
import os
import time

client_patterns = ClientPatterns()


from multiprocessing import Process, Event, Manager



class MyClient:

    @staticmethod
    def test_handler(data):
        print("Received data: ", data)
        
        # TODO >>> Save event in the test databse log
        
        # This will stop the client
        MyClient.instance.stop() 

    @staticmethod
    def send_some_data():
        time.sleep(10)
        mys_client = MysceliumClient(client_uid="some_client_id", buffer_path="ClientData/")
        mys_client.runing = True
        mys_client.set_client_uid(client_uid="some_client_id")
        command = client_patterns.command_pattern("python_function", args={"age": 10, "birth": 8, "name": "cristian"})
        result = mys_client.send(command, priority=10)
        print(result)

    def initializer(self, event_key):

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

        return

    def run(self, event_key):
        
        t1 = Process(target=self.initializer, args=(event_key, ))

        t2 = Process(target=self.send_some_data, args=())

        t1.start()
        time.sleep(5)
        t2.start()

        t2.join()
        t1.join()  # Wait for the process to finish

        return


    def stop(self):
        if hasattr(self, 'client_instance') and self.client_instance:
            self.client_instance.stop_client()  # assuming MysceliumClient has a stop() method


