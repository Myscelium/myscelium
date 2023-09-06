from myscelium import MysceliumClient, ClientPatterns
import os
import time
import signal

client_patterns = ClientPatterns()

from multiprocessing import Process, Event, Manager
from ..Logs.test_logs_mananger import Events_Mananger, System_Status

class MyClient:

    @staticmethod
    def test_handler(data):

        EVMananger = Events_Mananger(Unit="Client2", path="Logs")
        EVMananger.Set_Event("Activate Basic Response Test callback handler")

        print("Received data: ", data)

        # time.sleep(5)
        
        # System_Status(path="Logs").change_unit_status(Unit="Client2", Status=False)

    @staticmethod
    def send_some_data_to_redirect():

        time.sleep(20)
        mys_client = MysceliumClient(client_uid="randomsclientids", buffer_path="Client2Data/")
        mys_client.runing = True
        mys_client.set_client_uid(client_uid="randomsclientids")
        command = client_patterns.command_pattern("test_redirect", args={"client_id": "some_client_id", "data": 8})
        result = mys_client.send(command, priority=10)

        EVMananger = Events_Mananger(Unit="Client2", path="Logs")
        EVMananger.Set_Event("Data To Redirect Sended")

        print(result)

    def initializer(self):

        mys_client = MysceliumClient(client_uid="randomsclientids", buffer_path="Client2Data/")

        self.mys_client = mys_client

        mys_client.set_client_uid(client_uid="randomsclientids")

        callbacks = [
            client_patterns.callback_pattern(callback=MyClient.test_handler, args={
                "data": "dict"
            }),
        ]
        
        mys_client.set_callbacks(callbacks=callbacks)
        mys_client.set_workers_num(n_workers=2)

        System_Status(path="Logs").change_unit_status(Unit="Client2", Status=True)
        
        mys_client.initialize_client("127.0.0.1", 4444)

        return 
    
    def monitor_stop_event(self):
        
        time.sleep(20)

        System_Status(path="Logs").change_unit_status(Unit="Client2", Status=True)

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

        t1 = Process(target=self.initializer, args=())
        t2 = Process(target=self.send_some_data_to_redirect, args=())
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



