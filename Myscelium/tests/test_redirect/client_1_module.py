from myscelium import MysceliumClient, ClientPatterns
import os
import time
import signal

client_patterns = ClientPatterns()

from multiprocessing import Process, Event, Manager
from ..Logs.test_logs_manager import Events_Manager, System_Status

CLIENT_ID = "some_client_id"

class Senders:

    def __init__ (self):
        pass

    @staticmethod
    def send_some_data():

        # time.sleep(10)
        mys_client = MysceliumClient(client_uid=CLIENT_ID, buffer_path="Temp/Client1Data/")
        mys_client.running = True

        command = client_patterns.command_pattern(
            CLIENT_ID,
            "python_function", 
            kwargs={"age": 10, "birth": 8, "name": "cristian"}
        )

        result = mys_client.send(command, priority=10)

        Events_Manager(Unit="Client1", path="Logs").Set_Event(
            "Data Sended", 
            event_type="Send", 
            event_key="1dX2A63Rp7O79x6t"
        )

        print(result)


class Receivers:

    def __init__ (self):
        pass

    @staticmethod
    def test_handler(data:str):

        Events_Manager(Unit="Client1", path="Logs").Set_Event(
            "Activate Basic Response Test callback handler", 
            event_type="Receive", 
            event_key="r99F3i89D20Oj1lq"
        )

        if "status" in data:
            pass
        else:
            return None
        
        if data["status"] == "Success":
            pass
        else:
            return None

        print("Received data: ", data)

        # time.sleep(5)
        
        # System_Status(path="Logs").change_unit_status(Unit="Client1", Status=False)

    @staticmethod
    def test_redirect_handler(data:dict):

        Events_Manager(Unit="Client1", path="Logs").Set_Event(
            "Activate Basic Redirect Test callback handler", 
            event_type="Receive", 
            event_key="02V0P37Dz09zR3fL"
        )

        if "status" in data:
            pass
        else:
            return None
        
        if data["status"] == "Success":
            pass
        else:
            return None
        
        # TODO >>> Maybe implement a response redirect test from here

        print("Received redirected data: ", data)
        
        System_Status(path="Logs").change_unit_status(Unit="Client1", Status=False)

        return None

class MyClient:

    def __init__ (self, debug_level):
        self.debug_level = debug_level

    def initializer(self):

        mys_client = MysceliumClient(client_uid="some_client_id", buffer_path="Temp/Client1Data/", log_level=self.debug_level)

        self.mys_client = mys_client
        	
        receivers = Receivers()

        callbacks = [
            client_patterns.callback_pattern(callback=receivers.test_handler),
            client_patterns.callback_pattern(callback=receivers.test_redirect_handler),
        ]

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

    def run(self):

        t1 = Process(target=self.initializer, args=())

        # t2 = Process(target=senders.send_some_data, args=())
        t3 = Process(target=self.monitor_stop_event, args=())

        t1.start()
        time.sleep(15)
        # t2.start()
        t3.start()

        # t2.join()
        t3.join()  

        time.sleep(5)

        # PID is the process ID of the process you want to send the signal to.
        # You would typically get this from the 'pid' attribute of a process.
        os.kill(t1.pid, signal.SIGINT)

        t1.join()  # Wait for the process to finish

        return



