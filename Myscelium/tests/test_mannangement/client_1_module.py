from myscelium import MysceliumClient, ClientPatterns
import os
import time
import signal

client_patterns = ClientPatterns()

from multiprocessing import Process, Event, Manager
from ..Logs.test_logs_mananger import Events_Mananger, System_Status

class Receivers:

    @staticmethod
    def add_client_handler (data:any): # -> Need to be implemeneted
        
        # "data" {
        #     "command_type":"response",
        #     "status": "success"
        #     "response_activation_function":"",
        #     "message":"", 
        #     "kwargs":{"arg1": [], "arg2": "", "arg3": {}}
        #     "response_mode":"",
        # }
        
        if "status" in data:
            pass
        else:
            return None
        
        if data["status"] == "success":
            pass
        else:
            return None


        Events_Mananger(Unit="Client1", path="Logs").Set_Event(
            "Activate Basic Response Test Add Client"
        )

        print("Received data: ", data)

    @staticmethod
    def update_client_handler (data:any): # -> Need to be implemeneted

        Events_Mananger(Unit="Client1", path="Logs").Set_Event(
            "Activate Basic Response Test Update Client"
        )

        if "status" in data:
            pass
        else:
            return None
        
        if data["status"] == "success":
            pass
        else:
            return None

        print("Received data: ", data)
        
        # System_Status(path="Logs").change_unit_status(Unit="Client1", Status=False)

    @staticmethod
    def remove_client_handler (data:any): # TODO >>> test_remove_client

        Events_Mananger(Unit="Client1", path="Logs").Set_Event(
            "Activate Basic Response Test Remove Client"
        )
        
        if "status" in data:
            pass
        else:
            return None
        
        if data["status"] == "success":
            pass
        else:
            return None
 
        print("Received data: ", data)

        time.sleep(10)
        
        System_Status(path="Logs").change_unit_status(Unit="Client1", Status=False)


class Senders:

    def start_send_sequence (self):

        time.sleep(10)
        self.test_add_client()

        time.sleep(10)
        self.test_update_client()

        time.sleep(10)
        self.test_remove_client()

    @staticmethod
    def test_add_client (): 

        mys_client = MysceliumClient(client_uid="some_client_id", buffer_path="Temp/Client1Data/")
        mys_client.runing = True
        mys_client.set_client_uid(client_uid="some_client_id")
        
        command = client_patterns.command_pattern(
            "test_add_client", 
            args = {
                "client_name":"test_client", 
                "client_key":"xMndjslwpedcnfe", 
                "client_type":"Test", 
                "permission_group":"", 
                "is_super_user":True, 
                "max_sub_channels":5, 
                "owned_sub_channels_keys":[],
            }
        )

        result = mys_client.send(command, priority=9)

        Events_Mananger(Unit="Client1", path="Logs").Set_Event(
            "Send test add a client", 
            event_type = "Send", 
            event_key = "94G2zy6cV54GN64O"
        ) 
        

    @staticmethod
    def test_update_client (): 

        mys_client = MysceliumClient(client_uid="some_client_id", buffer_path="Temp/Client1Data/")
        mys_client.runing = True
        mys_client.set_client_uid(client_uid="some_client_id")
        
        command = client_patterns.command_pattern(
            "test_update_client", 
            args = {
                "actual_client_key":"xMndjslwpedcnfe",
                "client_key":"xMndjslwpedcnfe", 
                "client_name":"test_client", 
                "client_type":"Test", 
                "permission_group":"", 
                "is_super_user":True, 
                "max_sub_channels":10, 
                "owned_sub_channels_keys":[]
            }
        )
        
        result = mys_client.send(command, priority=8)

        Events_Mananger(Unit="Client1", path="Logs").Set_Event(
            "Send test update a client", 
            event_type="Send", 
            event_key="3p7194Y33W6BnYlA"
        ) 
 
    @staticmethod
    def test_remove_client (): 

        mys_client = MysceliumClient(client_uid="some_client_id", buffer_path="Temp/Client1Data/")
        mys_client.runing = True
        mys_client.set_client_uid(client_uid="some_client_id")
        
        command = client_patterns.command_pattern(
                        "test_remove_client", 
                        args = {
                            "client_key": "xMndjslwpedcnfe"
                        }
                    )
        
        result = mys_client.send(command, priority=7)

        Events_Mananger(Unit="Client1", path="Logs").Set_Event(
            "Send test remove a client", 
            event_type="Send", 
            event_key="30bt28u819A1QDpH"
        )     

class MyClient: 

    def initializer(self):

        my_handlers = Receivers ()

        mys_client = MysceliumClient(client_uid="some_client_id", buffer_path="Temp/Client1Data/", log_level="DEBUG")

        self.mys_client = mys_client

        mys_client.set_client_uid(client_uid="some_client_id")

        callbacks = [

            client_patterns.callback_pattern(
                callback=my_handlers.add_client_handler, 
            ),

            client_patterns.callback_pattern(
                callback=my_handlers.update_client_handler, 
            ),

            client_patterns.callback_pattern(
                callback=my_handlers.remove_client_handler, 
            ),

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
        t2 = Process(target=senders.start_send_sequence, args=())

        # TODO >>> Implement new senders

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



