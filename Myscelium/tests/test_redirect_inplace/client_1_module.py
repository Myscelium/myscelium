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
CLIENT_NAME = "TestClient1"

def shutdown ():
    print("Receive order to stop client 1")
    System_Status(path="Logs").change_unit_status(Unit="Host", Status=False)
    System_Status(path="Logs").change_unit_status(
        Unit="Client1", Status=False
    )
    return

class Senders:
    def __init__(self):
        pass

    @staticmethod
    def send_some_data():
        time.sleep(25)

        EVManager = Events_Manager(Unit="Client1", path="Logs")

        mys_client = MysceliumClient(
            name="TestClient1",
            client_uid=CLIENT_KEY,
            buffer_path="Temp/Client1Data/",
            is_main_process = False
        )

        mys_client.running = True

        Events_Manager(Unit="Client1", path="Logs").Set_Event(
            step=f"Waiting Client To Be Ready", event_type="Default"
        )

        #! Esplicity Define a ready statues waith mechanism now you don't need it anymore

        TARGET_KEY = "randomsclientids" # -> target is client2

        try: #! Here is required see if client is ready
            mys_client.ensure_client_ready(max_attempts=10, sleep_time=10)
        except Exception as e:
            Events_Manager(Unit="Client1", path="Logs").Set_Event(
                f"{e}",
                event_type="Default",
            )
            shutdown() 
            return

        Events_Manager(Unit="Client1", path="Logs").Set_Event(
            "Client 2 is ready",
            event_type="Default",
        )
        
        try: #! This require the target to be ready
            mys_client.ensure_target_ready(target_key=TARGET_KEY, max_attempts=10, sleep_time=10)
        except Exception as e: 
            Events_Manager(Unit="Client1", path="Logs").Set_Event(
                f"{e}",
                event_type="Default",
            )
            shutdown() 
            return

        try:
            command = client_patterns.command_pattern(
                origin_key=CLIENT_KEY,
                command_function="python_function",
                target_key=TARGET_KEY, # -> target is client2
                kwargs={"age": 10, "birth": 8, "name": "cristian"},
                message="",
                response_type="ExternalFunction",
                response_target="Origin",
                response_actf="test_handler",
                auto_collect_response=False,
            )
        except ValueError as e:
            Events_Manager(Unit="Client1", path="Logs").Set_Event(
                step=f"Error: {e}", event_type="Exception"
            )
            shutdown() 
            return

        parity_id = ""

        try:
            parity_id = mys_client.send(command, priority=10)
            Events_Manager(Unit="Client1", path="Logs").Set_Event(
                step=f"Scheduled command to send with parity id: {parity_id}", event_type="Default"
            )
        except ValueError as e:
            Events_Manager(Unit="Client1", path="Logs").Set_Event(
                step=f"Error: {e}", event_type="Exception"
            )
            shutdown() 
            return
        
        Events_Manager(Unit="Client1", path="Logs").Set_Event(
            step="Data Sended", event_type="Send", event_key="088p72pbv9Ozj7T1"
        )

        response = {}

        try:
            response = mys_client.wait_response(parity_id, timeout_in=80)
        except Exception as e:
            Events_Manager(Unit="Client1", path="Logs").Set_Event(
                step=f"Cant wait response, Error: {e}", event_type="Exception"
            )
            shutdown() 
            return
        
        Events_Manager(Unit="Client1", path="Logs").Set_Event(
            step=f"Receive Response: {response}", event_type="Default"
        )
        
        Events_Manager(Unit="Client1", path="Logs").Set_Event(
            "Receive Inplace Response",
            event_type="Receive",
            event_key="74L648VZDI7J1GV5",
        )        

        print(parity_id)

class MyClient:
    def __init__(self, debug_level):
        self.debug_level = debug_level

    def initializer(self):
        mys_client = MysceliumClient(
            name=CLIENT_NAME,
            client_uid=CLIENT_KEY,
            buffer_path="Temp/Client1Data/",
            log_level=self.debug_level,
            is_main_process = True
        )

        self.mys_client = mys_client

        callbacks = []

        mys_client.set_callbacks(callbacks=callbacks)
        mys_client.set_workers_num(n_workers=2)

        System_Status(path="Logs").change_unit_status(Unit="Client1", Status=True)

        mys_client.initialize_client("127.0.0.1", 4444)

        return

    def monitor_stop_event(self):
        time.sleep(35)  # needs to be a little more to wait to client 2 initialize

        System_Status(path="Logs").change_unit_status(Unit="Client1", Status=True)

        counter = 0 
        max_time = 10 # x*5s (50s) Max time to the test occur, prevent infinite cicles

        while True:
            client_status = System_Status(path="Logs").get_unit_status(Unit="Client2")
            host_status = System_Status(path="Logs").get_unit_status(Unit="Host")

            if counter > max_time:
                System_Status(path="Logs").change_unit_status(
                    Unit="Client1", Status=False
                )
                System_Status(path="Logs").change_unit_status(
                    Unit="Client2", Status=False
                )
                pass
            else:
                counter += 1

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
