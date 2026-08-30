# SPDX-License-Identifier: MPL-2.0
# Copyright © 2021-2026 Cristian Camargo Filho

from myscelium import MysceliumHost, HostPatterns, MysceliumHostInterface, callback_pattern, ClientPattern
from multiprocessing import Process, Event, Manager
from ..Logs.test_logs_manager import Events_Manager, System_Status
import os
import signal
import time

import time

# actual_to_compare['ClientName'], actual_to_compare['ClientKey'], actual_to_compare['LastContact']

def client_contact_event_handler (client_name:str, client_key:str, client_last_contact:float):
    Events_Manager(Unit="Host", path="Logs").Set_Event(step=f"Contact received from Client: {client_key}")
    print(client_name, client_key, client_last_contact)
    pass

class Handlers:

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
            print("info don't have the auto_collect, sending none")
            return None
        
        auto_collect = info["auto_collect"]

        if auto_collect or "response_actf" in info: # only require response_actf if auto_collect is true
            pass
        else:
            print("info don't have the response_actf, sending none")
            return None
        
        response_actf = info["response_actf"]
        
        host_patterns = HostPatterns()

        response = host_patterns.response_pattern(
            activation_function=response_actf, 
            kwargs={"data": 'hello!'},
            auto_collect=auto_collect,
        )

        Events_Manager(
            Unit="Host", 
            path="Logs"
        ).Set_Event(
            step="Active Basic Callback", 
            event_type="Receive", 
            event_key="088p72pbv9Ozj7T1"
        )
        
        Events_Manager(
            Unit="Host", 
            path="Logs"
        ).Set_Event(
            step="Send Response", 
            event_type="Send", 
            event_key="74L648VZDI7J1GV5"
        )
        
        Events_Manager(
            Unit="Host", 
            path="Logs"
        ).Set_Event(
            step=f"Base callback - Receive Data: [{age}, {birth}, {name}]"
        )

        return response

class MyHost:

    def __init__(self, debug_level):
        self.host_patterns = HostPatterns()
        self.debug_level = debug_level

    def monitor_stop_event(self):

        time.sleep(5)

        # -> Define how much time host will be alive!
        # TODO >>> In the future change to use 100% timeout
        
        n = 0 
        COUNTER = 12 # Each counter is 5 secs of waiting

        mys_host_interface = MysceliumHostInterface("Temp/Data/")
        mys_host_interface.set_client_contact_retriever_callback(client_contact_event_handler)
        mys_host_interface.start_client_events_retriever()

        while True:

            client_status = System_Status(path="Logs").get_unit_status(Unit="Client1")

            if (not client_status) or (n >= COUNTER):
                print("Receive stop host")
                mys_host_interface.stop_client_events_retriever()
                System_Status(path="Logs").change_unit_status(Unit="Host", Status=False)
                break

            else:
                time.sleep(5)
                n += 1
                continue

        return

    def run_host(self, ip, port):

        handlers = Handlers()

        callbacks = [
            
            callback_pattern(callback=handlers.python_function),

        ]

        allowed_clients = [

            ClientPattern(
                client_name="TestClient1", 
                client_type="Interface", 
                client_key="some_client_id", 
                client_permission_group="", 
                client_is_super_user=True, 
                max_sub_channels=5
            ).format(),

            ClientPattern(
                client_name="TestClient2", 
                client_type="Interface", 
                client_key="randomsclientids", 
                client_permission_group="", 
                client_is_super_user=True, 
                max_sub_channels=5
            ).format(),

        ]

        print(allowed_clients)

        mys_host = MysceliumHost(
            callbacks=callbacks, 
            host_id="xnsmdkeflerpfsa",
            allowed_clients=allowed_clients, 
            buffer_path="Temp/Data/", 
            n_workers=2, 
            log_level=self.debug_level
        )

        self.mys_host = mys_host

        System_Status(path="Logs").change_unit_status(Unit="Host", Status=True)

        mys_host.initialize_host(ip=ip, port=port)

        return

    def run(self, ip="127.0.0.1", port=4444, event=None):

        host_process = Process(target=self.run_host, args=(ip, port))
        monitor_process = Process(target=self.monitor_stop_event)

        host_process.start()
        monitor_process.start()

        monitor_process.join()

        # Send SIGINT to the process
        os.kill(host_process.pid, signal.SIGINT)

        host_process.join()

        return 

            


        

    

