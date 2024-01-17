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

class Handlers:

    @staticmethod   
    def python_function(age:int, birth:str, name:str):
        
        print("Access python function")
        print(birth)
        print(name)
        print(age)

        birth = int(birth)

        print(f"age: {birth + 25}")
        print(f"age is a type: {type(birth)}")

        if birth == int(8): # -> Correct response

            print("Case 8 activated")

            response = HostPatterns().response_pattern(
                "message_test_handler",
                kwargs={"data": 'hello!'},
                message="Success"
            )
            
            Events_Manager(Unit="Host", path="Logs").Set_Event(
                step="Active Basic Callback For Correct Data", 
                event_type="Receive", 
                event_key="95mO7n9g7H4N2eE9"
            )

            Events_Manager(Unit="Host", path="Logs").Set_Event(
                step=f"Base callback - Receive Data: [{age}, {birth}, {name}]"
            )

            Events_Manager(Unit="Host", path="Logs").Set_Event(
                step="Return Basic Callback Success Response", 
                event_type="Send", 
                event_key="A07u4a4sad1UX172"
            )

            return response
        
        if birth == 5: # -> Incorrect response

            print("Case 5 activated")

            response = HostPatterns().error_response_pattern( # For now this is only to origin
                error_message="incorrect_birth",
                expected_remote_error_handler='error_test_handler',
            )

            Events_Manager(Unit="Host", path="Logs").Set_Event(
                step="Active Basic Callback For Incorrect Data", 
                event_type="Receive", 
                event_key="3ATy5d761kn1Y8A9"
            )

            Events_Manager(Unit="Host", path="Logs").Set_Event(
                step=f"Base callback - Receive Data: [{age}, {birth}, {name}]"
            )

            Events_Manager(Unit="Host", path="Logs").Set_Event(
                step="Return Basic Callback Error Response",
                event_type="Send", 
                event_key="J0Wr7s116bM3sT15"
            )

            return response


        # (callback name) - Receive Data: [Data received list for comparison]

        return None
    
    # @staticmethod
    # def test_redirect_messages (client_id:str, data:int):
    #     if isinstance(client_id, str):
    #         print(f"Redirecting data: {data} to client: {client_id}")
    #         host_patterns = HostPatterns()

    #         response_data = {'data':data}

    #         response = host_patterns.response_pattern(
    #             response=response_data,
    #             response_mode='redirect',
    #             redirect_to_client_id=client_id,
    #             response_activation_function="test_redirect_handler"
    #         )

    #         print(f"Response Before send to engine: {response}")

    #         Events_Manager(Unit="Host", path="Logs").Set_Event(step="Active Host Redirect Callback") # This doesn't have a event_key because it is on the destine
            
    #         return response
    #     else:
    #         print("Client id isn't a string, failed to redirect data!")
    #         return None

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

            if (not client_1_status):
                print("Receive stop host from client 1")
                mys_host_interface.stop_client_events_retriever()
                System_Status(path="Logs").change_unit_status(Unit="Host", Status=False)
                break

            else:
                time.sleep(5)
                n += 1
                continue

        return

    def run_host(self, ip, port):

        callbacks = CallbackCollector([Handlers]).get_callbacks()

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

        # client_name:str, client_key:str, client_permission_group:str, client_is_super_user:bool, client_max_sub_channels:int, client_owned_sub_channels_keys:list

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

            


        

    

