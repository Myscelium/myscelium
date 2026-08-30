# SPDX-License-Identifier: MPL-2.0
# Copyright © 2021-2026 Cristian Camargo Filho

from myscelium import MysceliumClient, ClientPatterns
import os
import time
import signal

client_patterns = ClientPatterns()

from multiprocessing import Process, Event, Manager

class MyClient:

    @staticmethod
    def test_handler(data):

        print("Received data: ", data)

        time.sleep(5)

        return None
    
    @staticmethod
    def test_redirect_handler (data):

        print("Received redirected data: ", data)

        time.sleep(5)

        return None
        
    @staticmethod
    def send_some_data():

        time.sleep(10)
        mys_client = MysceliumClient(client_uid="some_client_id", buffer_path="Client1Data/")
        mys_client.running = True
        mys_client.set_client_uid(client_uid="some_client_id")
        command = client_patterns.command_pattern("python_function", args={"age": 10, "birth": 8, "name": "cristian"})
        result = mys_client.send(command, priority=10)

        print(result)

        return None

    def initializer(self):

        mys_client = MysceliumClient(client_uid="some_client_id", buffer_path="Client1Data/")

        self.mys_client = mys_client

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

    def run(self):

        t1 = Process(target=self.initializer, args=())
        t2 = Process(target=self.send_some_data, args=())

        t1.start()
        time.sleep(5)
        t2.start()

        time.sleep(5)

        # PID is the process ID of the process you want to send the signal to.
        # You would typically get this from the 'pid' attribute of a process.
        # os.kill(t1.pid, signal.SIGINT)

        t2.join()
        t1.join()  # Wait for the process to finish

        return

if __name__ == "__main__":
    MyClient().run()

