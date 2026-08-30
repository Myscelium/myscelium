# SPDX-License-Identifier: MPL-2.0
# Copyright © 2021-2026 Cristian Camargo Filho

from myscelium import MysceliumHost, HostPatterns, MysceliumHostInterface, CallbackCollector, CommandInstruction, ClientPattern
from multiprocessing import Process
import os
import signal
import time

# actual_to_compare['ClientName'], actual_to_compare['ClientKey'], actual_to_compare['LastContact']

def client_contact_event_handler (client_name:str, client_key:str, client_last_contact:float):
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
        
        if auto_collect:
        
            response = CommandInstruction (
                command_mode='Response',
                command_type="ExternalFunction",
                command_target=f"ClientKey({info['origin']['ClientKey']})", # -> target is client2
                command_status="Success",
                command_actf=f"{response_actf}",
                command_kwargs={"data": "hello!"},
                command_message="",
                response_type="ExternalFunction",
                response_target="Origin",
                response_actf=f"{response_actf}", 
                auto_collect_response=True,
            ).format()
            
        else:
            
            response = CommandInstruction (
                command_mode='Response',
                command_type="ExternalFunction",
                command_target=f"Origin", # -> target is client2
                command_status="Success",
                command_actf=f"", # -> Don't need actf when auto collect is False
                command_kwargs={"data": "hello!"},
                command_message="",
                response_type="ExternalFunction",
                response_target="Origin",
                response_actf=f"", # -> Don't need response actf when auto collect is False
                auto_collect_response=False,
            ).format()

        return response

    @staticmethod
    def sum (info:dict, num1:int, num2:int):
        print("Access python function")

        print(f"Info: {info}")
        print(f"Sum num1: {num1} with num2: {num2}")

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
        
        if auto_collect:
        
            response = CommandInstruction (
                command_mode='Response',
                command_type="ExternalFunction",
                command_target=f"ClientKey({info['origin']['ClientKey']})", # -> target is client2
                command_status="Success",
                command_actf=f"{response_actf}",
                command_kwargs={"data": num1 + num2},
                command_message="",
                response_type="ExternalFunction",
                response_target="Origin",
                response_actf=f"{response_actf}",
                auto_collect_response=True,
            ).format()
        
        else:
            
            response = CommandInstruction (
                command_mode='Response',
                command_type="ExternalFunction",
                command_target=f"Origin", # -> target is client2
                command_status="Success",
                command_actf=f"", # -> Don't need actf when auto collect is False
                command_kwargs={"data": num1 + num2},
                command_message="",
                response_type="ExternalFunction",
                response_target="Origin",
                response_actf=f"", # -> Don't need response actf when auto collect is False
                auto_collect_response=False,
            ).format()
            
            
    
        return response

class MyHost:

    def __init__(self, debug_level):
        self.debug_level = debug_level
        
    def _monitor_events(self):

        time.sleep(5)

        mys_host_interface = MysceliumHostInterface("Temp/Data/")
        mys_host_interface.set_client_contact_retriever_callback(client_contact_event_handler)
        mys_host_interface.start_client_events_retriever()

        while True:
            continue

        return

    def run_host(self, ip, port):

        callbacks = CallbackCollector(
            [
                Handlers,
            ]
        ).get_callbacks()

        allowed_clients = [

            ClientPattern(
                client_name="TestClient1", 
                client_type="Worker", 
                client_key="some_client_id", 
                client_permission_group="", 
                client_is_super_user=True, 
                max_sub_channels=5
            ).format(),
   

            ClientPattern(
                client_name="TestClient2", 
                client_type="Worker", 
                client_key="randomsclientids", 
                client_permission_group="", 
                client_is_super_user=True, 
                max_sub_channels=5
            ).format(),
            
            ClientPattern(
                client_name="Interface1", 
                client_type="Interface", 
                client_key="InitialHostKey", 
                client_permission_group="", 
                client_is_super_user=True, 
                max_sub_channels=5
            ).format(),

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

        mys_host.initialize_host(ip=ip, port=port)

        return

    def run(self, ip="127.0.0.1", port=8000, event=None):

        host_process = Process(target=self.run_host, args=(ip, port))
        monitor_process = Process(target=self._monitor_events)

        host_process.start()
        monitor_process.start()

        monitor_process.join()

        # Send SIGINT to the process
        os.kill(host_process.pid, signal.SIGINT)

        host_process.join()

        return 


if __name__ == "__main__":
    MyHost("DEBUG").run()           
                
        

    

