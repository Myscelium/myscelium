from myscelium import MysceliumClient, ClientPatterns, CallbackCollector
import os
import time
import signal

client_patterns = ClientPatterns()

from multiprocessing import Process, Event, Manager
from ..Logs.test_logs_mananger import Events_Mananger, System_Status

class Senders:

    def __init__ (self):
        pass

    @staticmethod
    def send_some_data():

        # time.sleep(10)
        mys_client = MysceliumClient(client_uid="some_client_id", buffer_path="Temp/Client1Data/")
        mys_client.runing = True
        mys_client.set_client_uid(client_uid="some_client_id")
        command = client_patterns.command_pattern("python_function", args={"age": 20, "birth": 8, "name": "cristian"})
        result = mys_client.send(command, priority=10)

        Events_Mananger(Unit="Client1", path="Logs").Set_Event(
            "Correct Data Sended", 
            event_type="Send", 
            event_key="95mO7n9g7H4N2eE9"
        )

        print(result)

    @staticmethod
    def send_some_incorrect_data():

        # time.sleep(10)
        mys_client = MysceliumClient(client_uid="some_client_id", buffer_path="Temp/Client1Data/")
        mys_client.runing = True
        mys_client.set_client_uid(client_uid="some_client_id")
        command = client_patterns.command_pattern("python_function", args={"age": 5, "birth": 5, "name": "potato"})
        result = mys_client.send(command, priority=10)

        Events_Mananger(Unit="Client1", path="Logs").Set_Event(
            "Incorrect Data Sended", 
            event_type="Send", 
            event_key="3ATy5d761kn1Y8A9"
        )

        print(result)

class Receivers:

    def __init__ (self):
        pass

    @staticmethod
    def message_test_handler(data:str):

        Events_Mananger(Unit="Client1", path="Logs").Set_Event(
            "Activate Basic Response Test callback handler", 
            event_type="Receive", 
            event_key="A07u4a4sad1UX172"
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

        # time.sleep(5)
        
        # System_Status(path="Logs").change_unit_status(Unit="Client1", Status=False)

    @staticmethod
    def error_test_handler(data:dict):

        Events_Mananger(Unit="Client1", path="Logs").Set_Event(
            "Activate Basic Redirect Test callback handler", 
            event_type="Receive", 
            event_key="J0Wr7s116bM3sT15"
        )

        if "status" in data:
            pass
        else:
            return None
        
        if data["status"] == "success":
            pass
        else:
            return None

        print("Received redirected data: ", data)
        
        System_Status(path="Logs").change_unit_status(Unit="Client1", Status=False)

class MyClient:

    def __init__ (self, debug_level):
        self.debug_level = debug_level

    def initializer(self):

        mys_client = MysceliumClient(client_uid="some_client_id", buffer_path="Temp/Client1Data/", log_level=self.debug_level)

        self.mys_client = mys_client

        mys_client.set_client_uid(client_uid="some_client_id")
        	
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



