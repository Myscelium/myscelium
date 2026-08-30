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

        return None

        # time.sleep(5)
        
        # System_Status(path="Logs").change_unit_status(Unit="Client2", Status=False)

    @staticmethod
    def send_some_data_to_redirect():

        time.sleep(20)
        mys_client = MysceliumClient(client_uid="randomsclientids", buffer_path="Client2Data/")
        mys_client.running = True
        mys_client.set_client_uid(client_uid="randomsclientids")
        command = client_patterns.command_pattern("test_redirect", args={"client_id": "some_client_id", "data": 8})
        result = mys_client.send(command, priority=10)

        print(result)

        return None

    def initializer(self):

        mys_client = MysceliumClient(client_uid="randomsclientids", buffer_path="Client2Data/")

        self.mys_client = mys_client

        mys_client.set_client_uid(client_uid="randomsclientids")

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
        t2 = Process(target=self.send_some_data_to_redirect, args=())

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