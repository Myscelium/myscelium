# SPDX-License-Identifier: MPL-2.0
# Copyright © 2021-2026 Cristian Camargo Filho

from myscelium import MysceliumClient, ClientPatterns, callback_pattern, CallbackCollector
import os
import time
import signal

client_patterns = ClientPatterns()

from multiprocessing import Process, Event, Manager
from ..Logs.test_logs_manager import Events_Manager, System_Status

CLIENT_ID = "randomsclientids"
CLIENT_NAME = "TestClient2"

def shutdown ():
    print("Receive order to stop client 2")
    System_Status(path="Logs").change_unit_status(Unit="Host", Status=False)
    System_Status(path="Logs").change_unit_status(
        Unit="Client2", Status=False
    )
    return

class Receivers:

    @staticmethod
    def python_function(info:dict, age:int, birth:int, name:str):
        print("Access python function")

        print(f"Info: {info}")
        print(birth)
        print(name)
        print(age)

        if "auto_collect" in info:
            pass
        else:
            Events_Manager(Unit="Client1", path="Logs").Set_Event(
                step=f"info don't have the auto_collect, sending none", event_type="Exception"
            )
            shutdown() 
            return
        
        auto_collect = info["auto_collect"]

        if auto_collect or "response_actf" in info: # only require response_actf if auto_collect is true
            pass
        else:
            Events_Manager(Unit="Client1", path="Logs").Set_Event(
                step=f"info don't have the response_actf, sending none", event_type="Exception"
            )
            shutdown() 
            return
        
        response_actf = info["response_actf"]
        client_patterns = ClientPatterns()

        response = client_patterns.response_pattern(
            activation_function=response_actf,
            kwargs={"data": "hello!"},
            target_key=info["origin"]['ClientKey'],
            auto_collect=auto_collect,
        )

        Events_Manager(
            Unit="Client2", 
            path="Logs"
        ).Set_Event(
            step="Active Basic Callback", 
            event_type="Receive", 
            event_key="088p72pbv9Ozj7T1"
        )
        
        Events_Manager(
            Unit="Client2", 
            path="Logs"
        ).Set_Event(
            step="Send Response", 
            event_type="Send", 
            event_key="74L648VZDI7J1GV5"
        )
        
        Events_Manager(
            Unit="Client2", 
            path="Logs"
        ).Set_Event(
            step=f"Base callback - Receive Data: [{age}, {birth}, {name}]"
        )

        return response

class MyClient:
    def __init__(self, debug_level):
        self.debug_level = debug_level

    def initializer(self):
        mys_client = MysceliumClient(
            name=CLIENT_NAME,
            client_uid=CLIENT_ID,
            buffer_path="Temp/Client2Data/",
            log_level=self.debug_level,
            is_main_process = True
        )

        self.mys_client = mys_client

        callbacks = CallbackCollector(
            [
                Receivers,
            ]
        ).get_callbacks()

        mys_client.set_callbacks(callbacks=callbacks)
        mys_client.set_workers_num(n_workers=2)

        System_Status(path="Logs").change_unit_status(Unit="Client2", Status=True)

        mys_client.initialize_client("127.0.0.1", 4444)

        return

    def monitor_stop_event(self):
        System_Status(path="Logs").change_unit_status(Unit="Client2", Status=True)

        time.sleep(30)

        while True:
            client_status = System_Status(path="Logs").get_unit_status(Unit="Client1")
            host_status = System_Status(path="Logs").get_unit_status(Unit="Host")

            if (not client_status) or (not host_status):
                print("Receive order to stop client 2")
                System_Status(path="Logs").change_unit_status(Unit="Host", Status=False)
                System_Status(path="Logs").change_unit_status(
                    Unit="Client2", Status=False
                )
                break
            else:
                time.sleep(5)
                continue

        return

    def run(self):

        t1 = Process(target=self.initializer, args=())
        t3 = Process(target=self.monitor_stop_event, args=())

        t1.start()

        t3.start()
        t3.join()

        time.sleep(5)

        # PID is the process ID of the process you want to send the signal to.
        # You would typically get this from the 'pid' attribute of a process.
        os.kill(t1.pid, signal.SIGINT)

        t1.join()  # Wait for the process to finish

        return
