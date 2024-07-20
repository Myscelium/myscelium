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
            continue

        return

    def run(self):
        # senders = Senders()

        t1 = Process(target=self.initializer, args=())
        # t2 = Process(target=senders.send_some_data, args=())
        t3 = Process(target=self.monitor_stop_event, args=())

        t1.start()
        time.sleep(5)
        # t2.start()
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
    MyClient("INFO").run()

# print(CallbackCollector([Receivers]).get_callbacks())
