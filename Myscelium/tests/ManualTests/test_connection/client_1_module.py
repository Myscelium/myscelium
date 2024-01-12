from myscelium import MysceliumClient, ClientPatterns
import os
import time
import signal

client_patterns = ClientPatterns()

from multiprocessing import Process, Event, Manager

CLIENT_KEY = "some_client_id"

class Senders:

    @staticmethod
    def send_some_data():

        time.sleep(20)

        mys_client = MysceliumClient(client_uid="some_client_id", buffer_path="Temp/Client1Data/")
        mys_client.running = True

        # origin_key:str, command_function:str, target_key:str="", kwargs:dict={}, message:str=""
        command = client_patterns.command_pattern(
            CLIENT_KEY,
            "python_function", 
            "", # Empty is default
            {"age": 10, "birth": 8, "name": "cristian"}
        )

        result = mys_client.send(command, priority=10)


        print(result)

class Receivers:

    @staticmethod
    def test_handler(data:dict):

        if "status" in data:
            pass
        else:
            return None
        
        if data["status"] == "success":
            pass
        else:
            return None

        print("Received data: ", data)

        time.sleep(5)
        

class MyClient:

    def __init__ (self, debug_level):
        self.debug_level = debug_level

    def initializer(self):

        receivers = Receivers()

        mys_client = MysceliumClient(client_uid=CLIENT_KEY, buffer_path="Temp/Client1Data/", log_level=self.debug_level)

        self.mys_client = mys_client

        callbacks = [
            client_patterns.callback_pattern(callback=receivers.test_handler),
        ]
        
        mys_client.set_callbacks(callbacks=callbacks)
        mys_client.set_workers_num(n_workers=2)

        
        mys_client.initialize_client("127.0.0.1", 4444)

        return 
    
    def monitor_stop_event(self):
        
        time.sleep(5)

        while True:
            continue

        return

    def run(self):

        senders = Senders()

        t1 = Process(target=self.initializer, args=())
        t2 = Process(target=senders.send_some_data, args=())
        t3 = Process(target=self.monitor_stop_event, args=())

        t1.start()
        time.sleep(5)
        t2.start()
        t3.start()

        
        t3.join()  

        time.sleep(5)

        # PID is the process ID of the process you want to send the signal to.
        # You would typically get this from the 'pid' attribute of a process.
        os.kill(t1.pid, signal.SIGINT)

        t1.join()  # Wait for the process to finish
        t2.join()

        return

if __name__ == "__main__":
    MyClient("DEBUG").run()  
