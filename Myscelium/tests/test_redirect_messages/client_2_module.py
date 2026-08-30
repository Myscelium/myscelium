# SPDX-License-Identifier: MPL-2.0
# Copyright © 2021-2026 Cristian Camargo Filho

from myscelium import MysceliumClient, ClientPatterns, CallbackCollector
import os
import time
import signal

client_patterns = ClientPatterns()

from multiprocessing import Process, Event, Manager
from ..Logs.test_logs_manager import Events_Manager, System_Status

class Senders:

    def __init__ (self):
        pass

class Receivers:

    def __init__ (self):
        pass
    
    @staticmethod
    def data_handler (self, data:dict):

        if "status" in data:
            pass
        else:
            print("Status isn't in data")
            return None
        
        if data["status"] == "success":
            pass
        else:
            print("Status isn't success")
            return None

        if "kwargs" in data:
            pass
        else:
            print("Kwargs isn't in data")
            return None
    
        data_dict = data["kwargs"]

        if "birth" in data_dict:
            pass
        else:
            print("birth key isn't in kwargs")

        if "name" in data_dict:
            pass
        else:
            print("name key isn't in kwargs")

        if "age" in data_dict:
            pass
        else:
            print("age key isn't in kwargs")
        
        birth = data_dict["birth"]
        name  = data_dict["name"]
        age   = data_dict["age"] 

        print("Access python function")
        
        print(birth)
        print(name)
        print(age)

        birth = int(birth)

        print(f"age is a type: {type(birth)}")

        if birth == int(8): # -> Correct response

            print("Case 8 activated")

            # response = HostPatterns().response_pattern(
            #     response_mode='to_origin',
            #     response_activation_function="message_test_handler",
            #     response={"data": 'hello!'},
            #     message="Success"
            # )

            # TODO >>> Implement the new mechanism to allow extract the origin of the command 

            response = client_patterns.response_pattern(
                kwargs={}, 
                response_mode="retransmit",
                retransmit_to_client_id="",
            )

            # TODO >>> Add incorrect data message to send back through redirect
            
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

            # response = HostPatterns().error_response_pattern( # For now this is only to origin
            #     error_message="incorrect_birth",
            #     expected_remote_error_handler='error_test_handler',
            # )

            response =  client_patterns.redirect_error_pattern(
                error_message="incorrect_birth",
                expected_remote_error_handler="",
                redirect_to="",
            )


            # TODO >>> Add incorrect data message to send back through redirect

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

class MyClient:

    def __init__ (self, debug_level):
        self.debug_level = debug_level

    def initializer(self):

        mys_client = MysceliumClient(client_uid="randomsclientids", buffer_path="Temp/Client1Data/", log_level=self.debug_level)

        self.mys_client = mys_client

        mys_client.set_client_uid(client_uid="randomsclientids")
        	
        callbacks = CallbackCollector([Receivers]).get_callbacks()

        mys_client.set_callbacks(callbacks=callbacks)
        mys_client.set_workers_num(n_workers=2)

        System_Status(path="Logs").change_unit_status(Unit="Client1", Status=True)
        
        mys_client.initialize_client("127.0.0.1", 4444)

        return 
    
    def monitor_stop_event(self):
        
        time.sleep(25) # needs to be a little more to wait to client 2 initialize

        System_Status(path="Logs").change_unit_status(Unit="Client1", Status=True)

        while True:

            client_status = System_Status(path="Logs").get_unit_status(Unit="Client1")
            host_status = System_Status(path="Logs").get_unit_status(Unit="Host")

            if (not client_status) or (not host_status):
                print("Receive order to stop client 1")
                System_Status(path="Logs").change_unit_status(Unit="Host", Status=False)
                System_Status(path="Logs").change_unit_status(Unit="Client1", Status=False)
                break
            else:
                time.sleep(5)
                continue

        return
    
    def start_data_sending_routine (self):

        senders = Senders()

        time.sleep(5)
        senders.send_some_data()
        time.sleep(10)
        senders.send_some_incorrect_data()

    def run(self):

        t1 = Process(target=self.initializer, args=())
        t2 = Process(target=self.start_data_sending_routine, args=())
        t3 = Process(target=self.monitor_stop_event, args=())

        t1.start()
        time.sleep(5)
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



