from myscelium import MysceliumClient, ClientPatterns, CallbackCollector, callback_pattern

import os
import time
import signal

client_patterns = ClientPatterns()

from multiprocessing import Process, Event, Manager

CLIENT_KEY = "some_client_id"
CLIENT_NAME = "TestClient1"
TEMP_PATH = "Temp/Client1Data/"
LOG_LEVEL = "INFO"


class Senders:
    @staticmethod
    def send_some_data():
        time.sleep(15)

        mys_client = MysceliumClient(
            name="TestClient1",
            client_uid="some_client_id",
            buffer_path="Temp/Client1Data/",
            is_main_process = False
        )

        mys_client.running = True

        max_attempts = 10
        attemtps = 0
        while not mys_client.is_client_ready():
            time.sleep(1)
            print("Client not ready yet")
            attemtps += 1
            if attemtps >= max_attempts:
                assert False, "Take too long to client be ready"
            continue

        # TODO >>> Add panic handler inside myscelium when something returns a error from python side
            
        # origin_key:str, command_function:str, target_key:str="", kwargs:dict={}, message:str=""
        command = client_patterns.command_pattern(
            origin_key=CLIENT_KEY,
            command_function="python_function",
            target_key="",  # Empty is default
            kwargs={"age": 10, "birth": 8, "name": "cristian"},
            message="",
            response_type="ExternalFunction",
            response_target="Origin",
            response_actf="test_handler",
        )
    

        result = mys_client.send(command, priority=10)

        print(result)


class Receivers:
    @staticmethod
    def test_handler(info: dict):
        print("Received data: ", info)

        if "status" in info:
            pass
        else:
            return None

        if info["status"] == "success":
            pass
        else:
            return None

        print("Received data: ", info)

        time.sleep(5)

        return None


class MyClient:
    def __init__(self, debug_level):
        self.debug_level = debug_level

    def initializer(self):
        receivers = Receivers()

        mys_client = MysceliumClient(
            name="TestClien1",
            client_uid=CLIENT_KEY,
            buffer_path="Temp/Client1Data/",
            log_level=self.debug_level,
            is_main_process = True
        )

        self.mys_client = mys_client

        callbacks = [
            callback_pattern(callback=receivers.test_handler),
        ]

        mys_client.set_callbacks(callbacks=callbacks)
        mys_client.set_workers_num(n_workers=2)

        mys_client.initialize_client("127.0.0.1", 4444)

        return

    def monitor_stop_event(self):
        time.sleep(5)

        while True:
            continue

        return

    def run(self):
        senders = Senders()

        t1 = Process(target=self.initializer, args=())
        t2 = Process(target=senders.send_some_data, args=())
        t3 = Process(target=self.monitor_stop_event, args=())

        t1.start()
        time.sleep(5)
        t2.start()
        t3.start()

        t3.join()

        time.sleep(5)

        # PID is the process ID of the process you want to send the signal to.
        # You would typically get this from the 'pid' attribute of a process.
        os.kill(t1.pid, signal.SIGINT)

        t1.join()  # Wait for the process to finish
        t2.join()

        return


if __name__ == "__main__":
    MyClient("INFO").run()

# print(CallbackCollector([Receivers]).get_callbacks())
