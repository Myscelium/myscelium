from myscelium import MysceliumClient, ClientPatterns, callback_pattern
import os
import time
import signal

client_patterns = ClientPatterns()

from multiprocessing import Process, Event, Manager
from ..Logs.test_logs_manager import Events_Manager, System_Status

CLIENT_ID = "some_client_id"
CLIENT_NAME = "TestClient1"


class Senders:
    def __init__(self):
        pass

    @staticmethod
    def send_some_data():
        # time.sleep(10)
        mys_client = MysceliumClient(
            name=CLIENT_NAME, client_uid=CLIENT_ID, buffer_path="Temp/Client1Data/", is_main_process = False
        )
        mys_client.running = True

        max_attempts = 20
        attemtps = 0
        while not mys_client.is_client_ready():
            time.sleep(1)
            attemtps += 1
            if attemtps >= max_attempts:
                assert False, "Take too long to client be ready"
            continue
            
        command = client_patterns.command_pattern(
            origin_key=CLIENT_ID,
            command_function="python_function",
            kwargs={"age": 10, "birth": 8, "name": "cristian"},
            response_type="ExternalFunction",
            response_target= "Origin",
            response_actf="test_handler",
        )

        try:
            result = mys_client.send(command, priority=10)
        except ValueError as e:
            Events_Manager(Unit="Client1", path="Logs").Set_Event(
                f"Error: {e}", event_type="Exception"
            )

        Events_Manager(Unit="Client1", path="Logs").Set_Event(
            "Data Sended", event_type="Send", event_key="1dX2A63Rp7O79x6t"
        )

        print(result)


class Receivers:
    def __init__(self):
        pass

    @staticmethod
    def test_handler(info: dict):
        Events_Manager(Unit="Client1", path="Logs").Set_Event(
            "Activate Basic Response Test callback handler",
            event_type="Receive",
            event_key="r99F3i89D20Oj1lq",
        )

        if "status" in info:
            pass
        else:
            return None

        if info["status"] == "Success":
            pass
        else:
            return None

        print("Received data: ", info)

    @staticmethod
    def test_redirect_handler(info: dict):
        Events_Manager(Unit="Client1", path="Logs").Set_Event(
            "Activate Basic Redirect Test callback handler",
            event_type="Receive",
            event_key="02V0P37Dz09zR3fL",
        )

        if "status" in info:
            pass
        else:
            return None

        if info["status"] == "Success":
            pass
        else:
            return None

        print("Received redirected data: ", info)

        System_Status(path="Logs").change_unit_status(Unit="Client1", Status=False)

        return None


class MyClient:
    def __init__(self, debug_level):
        self.debug_level = debug_level

    def initializer(self):
        mys_client = MysceliumClient(
            name=CLIENT_NAME,
            client_uid="some_client_id",
            buffer_path="Temp/Client1Data/",
            log_level=self.debug_level,
            is_main_process = True
        )

        self.mys_client = mys_client

        receivers = Receivers()

        callbacks = [
            callback_pattern(callback=receivers.test_handler),
            callback_pattern(callback=receivers.test_redirect_handler),
        ]

        mys_client.set_callbacks(callbacks=callbacks)
        mys_client.set_workers_num(n_workers=2)

        System_Status(path="Logs").change_unit_status(Unit="Client1", Status=True)

        mys_client.initialize_client("127.0.0.1", 4444)

        return

    def monitor_stop_event(self):
        time.sleep(35)  # needs to be a little more to wait to client 2 initialize

        System_Status(path="Logs").change_unit_status(Unit="Client1", Status=True)

        while True:
            client_status = System_Status(path="Logs").get_unit_status(Unit="Client2")
            host_status = System_Status(path="Logs").get_unit_status(Unit="Host")

            if (not client_status) or (not host_status):
                print("Receive order to stop client 1")
                System_Status(path="Logs").change_unit_status(Unit="Host", Status=False)
                System_Status(path="Logs").change_unit_status(
                    Unit="Client1", Status=False
                )
                break
            else:
                time.sleep(5)
                continue

        return

    def run(self):
        t1 = Process(target=self.initializer, args=())

        t2 = Process(target=Senders().send_some_data, args=())
        t3 = Process(target=self.monitor_stop_event, args=())

        t1.start()

        t2.start()
        t3.start()

        t2.join()
        t3.join()

        time.sleep(5)

        # PID is the process ID of the process you want to send the signal to.
        # You would typically get this from the 'pid' attribute of a process.
        os.kill(t1.pid, signal.SIGINT)

        t1.join()  # Wait for the process to finish

        return
