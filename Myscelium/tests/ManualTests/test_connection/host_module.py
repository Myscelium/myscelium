# SPDX-License-Identifier: MPL-2.0
# Copyright © 2021-2026 Cristian Camargo Filho

from myscelium import MysceliumHost, HostPatterns, MysceliumHostInterface
from multiprocessing import Process, Event, Manager
import os
import signal
import time

import time

# actual_to_compare['ClientName'], actual_to_compare['ClientKey'], actual_to_compare['LastContact']

def client_contact_event_handler (client_name:str, client_key:str, client_last_contact:float):
    print(client_name, client_key, client_last_contact)
    pass

class Handlers:

    @staticmethod
    def python_function(age:int, birth:int, name:str):
        print("Access python function")
        print(birth)
        print(name)
        print(age)

        host_patterns = HostPatterns()

        response = host_patterns.response_pattern(
            activation_function="test_handler", 
            kwargs={"data": 'hello!'}
        )

        # (callback name) - Receive Data: [Data received list for comparison]

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
            continue

        return

    def run_host(self, ip, port):

        handlers = Handlers()

        callbacks = [
            
            self.host_patterns.callback_pattern(callback=handlers.python_function),
            # self.host_patterns.callback_pattern(callback=self.test_redirect),

        ]

        allowed_clients = [

            self.host_patterns.client_pattern(
                client_name="TestClient1", 
                client_type="Interface", 
                client_key="some_client_id", 
                client_permission_group="", 
                client_is_super_user=True, 
                max_sub_channels=5
            ),

            self.host_patterns.client_pattern(
                client_name="TestClient2", 
                client_type="Interface", 
                client_key="randomsclientids", 
                client_permission_group="", 
                client_is_super_user=True, 
                max_sub_channels=5
            ),

            self.host_patterns.client_pattern(
                client_name="TestClient3", 
                client_type="Interface", 
                client_key="InitialHostKey", 
                client_permission_group="", 
                client_is_super_user=True, 
                max_sub_channels=5
            ),

        ]

        print(allowed_clients)

        # client_name:str, client_key:str, client_permission_group:str, client_is_super_user:bool, client_max_sub_channes:int, client_owned_sub_channels_keys:list

        mys_host = MysceliumHost(
            callbacks=callbacks, 
            host_id="xnsmdkeflerpfsa",
            allowed_clients=allowed_clients, 
            buffer_path="Temp/Data/", 
            n_workers=2, 
            log_level="INFO"
        )

        self.mys_host = mys_host

        # client_heart_beat_handler = [self.host_patterns.callback_pattern(callback=self.handle_client_contact,
        #                                                                  args={"client_id": "str", "event_key": "str"}), ]

        # mys_host.set_client_heartbeat_handler(callback=client_heart_beat_handler)

        # TODO >>> Add callback handler to handle client contact (need to be like the logs transposer {Based on BufferDbTechnologies})
        
        mys_host.initialize_host(ip=ip, port=port)

        return

    def run(self, ip="127.0.0.1", port=8000, event=None):

        host_process = Process(target=self.run_host, args=(ip, port))
        monitor_process = Process(target=self.monitor_stop_event)

        host_process.start()
        monitor_process.start()

        monitor_process.join()

        # Send SIGINT to the process
        os.kill(host_process.pid, signal.SIGINT)

        host_process.join()

        return 

if __name__ == "__main__":
    MyHost("INFO").run()            


        

    

