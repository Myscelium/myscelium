from myscelium import MysceliumClient, ClientPatterns
import os
import time
import signal

client_patterns = ClientPatterns()

from multiprocessing import Process, Event, Manager
from ..Logs.test_logs_mananger import Events_Mananger, System_Status


class Senders:

    @staticmethod
    def send_some_data_to_redirect():

        # time.sleep(20)
        mys_client = MysceliumClient(client_uid="randomsclientids", buffer_path="Temp/Client2Data/")
        mys_client.runing = True
        mys_client.set_client_uid(client_uid="randomsclientids")
        command = client_patterns.command_pattern("test_redirect", args={"client_id": "some_client_id", "data": 8})
        result = mys_client.send(command, priority=10)

        Events_Mananger(Unit="Client2", path="Logs").Set_Event(
            "Data To Redirect Sended", 
            event_type="Send", 
            event_key="02V0P37Dz09zR3fL"
        )
        
        print(result)

class Receivers:

    @staticmethod
    def test_handler(data:str):

        Events_Mananger(Unit="Client2", path="Logs").Set_Event(
            "Activate Basic Response Test callback handler"
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
        
        # System_Status(path="Logs").change_unit_status(Unit="Client2", Status=False)


class MyClient:

    def initializer(self):

        mys_client = MysceliumClient(client_uid="randomsclientids", buffer_path="Temp/Client2Data/", log_level="DEBUG")

        self.mys_client = mys_client

        mys_client.set_client_uid(client_uid="randomsclientids")

        receivers = Receivers()

        callbacks = [
            client_patterns.callback_pattern(callback=receivers.test_handler),
        ]
        
        mys_client.set_callbacks(callbacks=callbacks)
        mys_client.set_workers_num(n_workers=2)

        System_Status(path="Logs").change_unit_status(Unit="Client2", Status=True)
        
        mys_client.initialize_client("127.0.0.1", 4444)

        return 
    
    def monitor_stop_event(self):
        
        time.sleep(20)

        System_Status(path="Logs").change_unit_status(
            Unit="Client2", 
            Status=True
        )

        while True:

            client_status = System_Status(path="Logs").get_unit_status(Unit="Client1")
            host_status = System_Status(path="Logs").get_unit_status(Unit="Host")

            if (not client_status) or (not host_status):
                print("Receive order to stop client 2")
                System_Status(path="Logs").change_unit_status(Unit="Host", Status=False)
                System_Status(path="Logs").change_unit_status(Unit="Client2", Status=False)
                break
            else:
                time.sleep(5)
                continue

        return

    def run(self):  

        senders = Senders ()

        t1 = Process(target=self.initializer, args=())
        t2 = Process(target=senders.send_some_data_to_redirect, args=())
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



