from myscelium import (
    MysceliumClient,
    ClientPatterns,
    CallbackCollector,
    callback_pattern,
    CommandInstruction
)

import os
import time
import signal

client_patterns = ClientPatterns()

from multiprocessing import Process, Event, Manager

CLIENT_KEY = "some_client_id"
CLIENT_NAME = "TestClient1"
TEMP_PATH = "Temp/Client1Data/"
LOG_LEVEL = "INFO"
CLIENT_ONLINE = True

def shutdown ():
    print("Receive order to stop client 1")
    CLIENT_ONLINE = False
    return

class Senders:
    @staticmethod
    def send_some_data():
        time.sleep(5)
        
        print("Starting sender threads")

        mys_client = MysceliumClient(
            name="TestClient1",
            client_uid="some_client_id",
            buffer_path="Temp/Client1Data/",
            is_main_process = False
        )

        mys_client.running = True

        try: #! Here is required see if client is ready
            mys_client.ensure_client_ready(max_attempts=25, sleep_time=1)
        except Exception as e:
            shutdown() 
            return

        #! Esplicity Define a ready statues waith mechanism now you don't need it anymore
        
        # max_attempts = 10
        # attemtps = 0
        # while not mys_client.is_client_ready():
        #     time.sleep(1)
        #     attemtps += 1
        #     if attemtps >= max_attempts:
        #         Events_Manager(Unit="Client1", path="Logs").Set_Event(
        #             step=f"Take too long to client be ready!", event_type="Exception"
        #         )
        #         assert False, "Take too long to client be ready"
        #     continue

        # origin_key:str, command_function:str, target_key:str="", kwargs:dict={}, message:str=""
        try:
            command = client_patterns.command_pattern(
                command_function="python_function",
                target_key="",  # Empty is default
                kwargs={"age": 10, "birth": 8, "name": "cristian"},
                message="",
                response_type="ExternalFunction",
                response_target="Origin",
                response_actf="test_handler",
                auto_collect_response=True,
            )
        except ValueError as e:
            return
        
        try:
            parity_id = mys_client.send(command, priority=10)
        except ValueError as e:
            return

        print(parity_id)

class Handlers:
    @staticmethod
    def python_function(info: dict, age: int, birth: int, name: str):
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

        if "origin" in info:
            pass
        else:
            return None

        if (
            auto_collect or "response_actf" in info
        ):  # only require response_actf if auto_collect is true
            pass
        else:
            print("info don't have the response_actf, sending none")
            return None

        response_actf = info["response_actf"]

    
        ty = type(info["origin"])
        inf = info["origin"]

        print(f"type: {ty} origin: {inf}")
    
        client_patterns = ClientPatterns()
        
        if auto_collect:
        
            response = CommandInstruction (
                command_mode='Response',
                command_type="ExternalFunction",
                command_target=f"ClientKey({info['origin']['ClientKey']})", # -> target is client2
                command_status="Success",
                command_actf="",
                command_kwargs={"data": "hello!"},
                command_message="",
                response_type="ExternalFunction",
                response_target="Origin",
                response_actf="", # -> Don't need response actf when auto collect is False
                auto_collect_response=True,
            ).format()
            
        else:
            
            response = CommandInstruction (
                command_mode='Response',
                command_type="ExternalFunction",
                command_target=f"Origin", # -> target is client2
                command_status="Success",
                command_actf="",
                command_kwargs={"data": "hello!"},
                command_message="",
                response_type="ExternalFunction",
                response_target="Origin",
                response_actf="", # -> Don't need response actf when auto collect is False
                auto_collect_response=False,
            ).format()
            
    
        return response

    @staticmethod
    def sum(info: dict, num1: int, num2: int):
        print("Access python function")

        print(f"Info: {info}")
        print(f"Sum num1: {num1} with num2: {num2}")

        if "auto_collect" in info:
            pass
        else:
            print("info don't have the auto_collect, sending none")
            return None

        auto_collect = info["auto_collect"]

        if (
            auto_collect or "response_actf" in info
        ):  # only require response_actf if auto_collect is true
            pass
        else:
            print("info don't have the response_actf, sending none")
            return None


        if auto_collect:
            
            response = CommandInstruction (
                command_mode='Response',
                command_type="ExternalFunction",
                command_target=f"ClientKey({info['origin']['ClientKey']})", # -> target is client2
                command_status="Success",
                command_actf="",
                command_kwargs={"data": num1 + num2},
                command_message="",
                response_type="ExternalFunction",
                response_target="Origin",
                response_actf="", # -> Don't need response actf when auto collect is False
                auto_collect_response=True,
            ).format()
        
        else:
            
            response = CommandInstruction (
                command_mode='Response',
                command_type="ExternalFunction",
                command_target=f"Origin", # -> target is client2
                command_status="Success",
                command_actf="",
                command_kwargs={"data": num1 + num2},
                command_message="",
                response_type="ExternalFunction",
                response_target="Origin",
                response_actf="", # -> Don't need response actf when auto collect is False
                auto_collect_response=False,
            ).format()
            
        return response

class MyClient:
    def __init__(self, debug_level):
        self.debug_level = debug_level

    def initializer(self):

        mys_client = MysceliumClient(
            name="TestClien1",
            client_uid=CLIENT_KEY,
            buffer_path="Temp/Client1Data/",
            log_level=self.debug_level,
            is_main_process=True,
        )

        self.mys_client = mys_client
        
        callbacks = CallbackCollector(
            [
                Handlers,
            ]
        ).get_callbacks()

        mys_client.set_callbacks(callbacks=callbacks)
        mys_client.set_workers_num(n_workers=2)

        mys_client.initialize_client("127.0.0.1", 8000)

        return

    def monitor_stop_event(self):
        time.sleep(5)

        while True:
            if not CLIENT_ONLINE:
                break
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
    MyClient("").run()

# print(CallbackCollector([Receivers]).get_callbacks())
