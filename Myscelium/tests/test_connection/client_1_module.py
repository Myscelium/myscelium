# SPDX-License-Identifier: MPL-2.0
# Copyright © 2021-2026 Cristian Camargo Filho

from myscelium import MysceliumClient, ClientPatterns, callback_pattern
import os
import time
import signal

client_patterns = ClientPatterns()

from multiprocessing import Process, Event, Manager
from ..Logs.test_logs_manager import Events_Manager, System_Status

CLIENT_KEY = "some_client_id"

def shutdown ():
    print("Receive order to stop client 1")
    System_Status(path="Logs").change_unit_status(Unit="Host", Status=False)
    System_Status(path="Logs").change_unit_status(
        Unit="Client1", Status=False
    )
    return

class Senders:
    @staticmethod
    def send_some_data():
        time.sleep(25)

        mys_client = MysceliumClient(
            name="TestClient1",
            client_uid="some_client_id",
            buffer_path="Temp/Client1Data/",
            is_main_process = False
        )

        mys_client.running = True

        Events_Manager(Unit="Client1", path="Logs").Set_Event(
            step=f"Waiting Client To Be Ready", event_type="Default"
        )
        
        try: #! Here is required see if client is ready
            mys_client.ensure_client_ready(max_attempts=25, sleep_time=1)
        except Exception as e:
            Events_Manager(Unit="Client1", path="Logs").Set_Event(
                f"{e}",
                event_type="Default",
            )
            shutdown() 
            return

        #! Esplicity Define a ready statues waith mechanism now you don't need it anymore
        
        # max_attempts = 10
        # attemtps = 0
        # while not mys_client.is_client_ready():
        #     time.sleep(1)
        #     attemtps += 1
        #     if attemtps >= max_attempts:
        #         Events_Manager(Unit="Client1", path="Logs").Set_Event(
        #             step=f"Take too long to client be ready!", event_type="Exception"
        #         )
        #         assert False, "Take too long to client be ready"
        #     continue

        # origin_key:str, command_function:str, target_key:str="", kwargs:dict={}, message:str=""
        try:
            command = client_patterns.command_pattern(
                command_function="python_function",
                target_key="",  # Empty is default
                kwargs={"age": 10, "birth": 8, "name": "cristian"},
                message="",
                response_type="ExternalFunction",
                response_target="Origin",
                response_actf="test_handler",
                auto_collect_response=True,
            )
        except ValueError as e:
            Events_Manager(Unit="Client1", path="Logs").Set_Event(
                step=f"Error: {e}", event_type="Exception"
            )
            return

        try:
            parity_id = mys_client.send(command, priority=10)
            Events_Manager(Unit="Client1", path="Logs").Set_Event(
                step=f"Scheduled command to send with parity id: {parity_id}", event_type="Default"
            )
        except ValueError as e:
            Events_Manager(Unit="Client1", path="Logs").Set_Event(
                step=f"Error: {e}", event_type="Exception"
            )
            return
    
        Events_Manager(Unit="Client1", path="Logs").Set_Event(
            step="Data Sended", event_type="Send", event_key="088p72pbv9Ozj7T1"
        )

        print(parity_id)


class Receivers:
    @staticmethod
    def test_handler(info: dict):
        EVManager = Events_Manager(Unit="Client1", path="Logs")
        EVManager.Set_Event(
            "Activate Basic Response Test callback handler",
            event_type="Receive",
            event_key="74L648VZDI7J1GV5",
        )

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

        System_Status(path="Logs").change_unit_status(Unit="Client1", Status=False)


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

        System_Status(path="Logs").change_unit_status(Unit="Client1", Status=True)

        mys_client.initialize_client("127.0.0.1", 4444)

        return

    def monitor_stop_event(self):
        time.sleep(5)

        while True:
            client_status = System_Status(path="Logs").get_unit_status(Unit="Client1")
            host_status = System_Status(path="Logs").get_unit_status(Unit="Host")

            if (not client_status) or (not host_status):
                print("Receive stop client")
                System_Status(path="Logs").change_unit_status(Unit="HOST", Status=False)
                break
            else:
                time.sleep(5)
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
