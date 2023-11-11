from myscelium import MysceliumHost, HostPatterns, MysceliumHostInterface, CallbackCollector
from multiprocessing import Process, Event, Manager
from ..Logs.test_logs_manager import Events_Manager, System_Status
import os
import signal
import time

import time

# ctual_to_compare['ClientName'], actual_to_compare['ClientKey'], actual_to_compare['LastContact']

def client_contact_event_handler (client_name:str, client_key:str, client_last_contact:float):
    Events_Manager(Unit="Host", path="Logs").Set_Event(step=f"Contact received from Client: {client_key}")
    print(client_name, client_key, client_last_contact)
    pass

class Retransmiters:

    def retransmit_data (self, client_to_redirect:str, data:dict):
        
        print (f"Receive:\n{data}\nto redirect to: {client_to_redirect}")

        response = HostPatterns().response_pattern(
            response=data,
            response_mode="redirect",
            response_activation_function="data_handler",
            redirect_to_client_id=client_to_redirect,
            message=""
        )
        
        return response

class MyHost:

    def __init__(self, debug_level):
        self.host_patterns = HostPatterns()
        self.debug_level = debug_level

    def monitor_stop_event(self):

        time.sleep(10)

        # -> Define how much time host will be alive!
        # TODO >>> In the future change to use 100% timeout
        
        n = 0 
        COUNTER = 62 # Each counter is 5 secs of waiting
        
        mys_host_interface = MysceliumHostInterface("Temp/Data/")

        mys_host_interface.set_client_contact_retriever_callback(client_contact_event_handler)

        mys_host_interface.start_client_events_retriever()

        time.sleep(30)

        while True:

            client_1_status = System_Status(path="Logs").get_unit_status(Unit="Client1")
            client_2_status = System_Status(path="Logs").get_unit_status(Unit="Client2")

            if (not client_1_status): # -> If Client1 status is offline
                print("Receive stop host from client 1")
                System_Status(path="Logs").change_unit_status(Unit="Client2", Status=False) # -> Turn Client2 Offline
                mys_host_interface.stop_client_events_retriever()
                System_Status(path="Logs").change_unit_status(Unit="Host", Status=False)
                break 
            
            elif (not client_2_status): # -> If Client2 status is offline
                print("Receive stop host from client 2")
                System_Status(path="Logs").change_unit_status(Unit="Client1", Status=False) # -> Turn Client1 Offline
                mys_host_interface.stop_client_events_retriever()
                System_Status(path="Logs").change_unit_status(Unit="Host", Status=False)
                break

            else:
                time.sleep(5)
                n += 1
                continue

        return

    def run_host(self, ip, port):

        callbacks = CallbackCollector([Retransmiters]).get_callbacks()

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

        ]

        print(allowed_clients)

        # client_name:str, client_key:str, client_permission_group:str, client_is_super_user:bool, client_max_sub_channes:int, client_owned_sub_channels_keys:list

        mys_host = MysceliumHost(
            callbacks=callbacks, 
            host_id="xnsmdkeflerpfsa",
            allowed_clients=allowed_clients, 
            buffer_path="Temp/Data/", 
            n_workers=2, 
            log_level=self.debug_level
        )

        self.mys_host = mys_host

        # TODO >>> Add callback handler to handle client contact (need to be like the logs transposer {Based on BufferDbTechnologies})

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

            


        

    

