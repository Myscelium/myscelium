from myscelium import MysceliumClient, ClientPatterns, MysceliumClientInterface
from threading import Thread
from _thread import *

from multiprocessing import Process
import time
import os

client_patterns = ClientPatterns ()

def logs_handler (log:dict):

    try:
        log_time  = log["log_time"]
        log_level = log["log_level"]
        log_msg   = log["log_msg"]

        print(f"{log_time} - {log_level} - {log_msg}")
    except:
        pass

    pass

# -> Response callbacks

def test_handler (data):

    print ("Receive data: ", data)

    return None


callbacks = [

    client_patterns.callback_pattern(callback=test_handler, args={
        "data" : "dict"
    }),

]

# -> Send Mecanism

def send_some_data ():

    mys_client = MysceliumClient(client_uid="some_client_id", buffer_path="ClientData/", log_level="DEBUG")

    # > ----------------------------------------------------------------------------------
    # > Logs
    #! Feature removed for now.

    # logs_callback_handler = [client_patterns.callback_pattern(callback=logs_handler, args={
    #     "node_name":"str",
    #     "log_time":"float",
    #     "log_name":"str",
    #     "log_msg":"str",
    # }),]

    # mys_client.set_logs_callback_handler (logs_callback_handler)

    # > ----------------------------------------------------------------------------------
    # > Initialization

    mys_client.set_client_uid(client_uid="some_client_id")

    mys_client.runing = True

    time.sleep(10)

    command = client_patterns.command_pattern("python_function", args={"age":10, "birth":8, "name":"cristian"})

    result = mys_client.send(command, priority=10)

    print (result)

    pass

# -> Initializers

# def logs_handler (node_name:str, log_time:float, log_name:str, log_msg:str):
#     print(f"{log_time} - {log_name} - {log_msg}")
#     pass

def initialize_client ():

    BUFFER_PATH = "ClientData/"

    mys_client = MysceliumClient(client_uid="some_client_id", buffer_path=BUFFER_PATH, log_level="DEBUG")

    # > ----------------------------------------------------------------------------------
    # > Logs
    #! Feature removed for now.

    # logs_callback_handler = [client_patterns.callback_pattern(callback=logs_handler, args={
    #     "node_name":"str",
    #     "log_time":"float",
    #     "log_name":"str",
    #     "log_msg":"str",
    # }),]

    # mys_client.set_logs_callback_handler (logs_callback_handler)

    # > ----------------------------------------------------------------------------------
    # > Initialization

    mys_client.set_callbacks(callbacks=callbacks)
    mys_client.set_workers_num(n_workers=2)

    client_logs_interface = MysceliumClientInterface(buffer_path=BUFFER_PATH)

    client_logs_interface.set_logs_callback(callback=logs_handler)

    client_logs_interface.allow_multi_handlers(workers_num=3)

    client_logs_interface.start_logs_retriver()

    mys_client.initialize_client("127.0.0.1",4444)


if __name__ == '__main__':
    
    # print(mys_client.get_registred_commands())

    p1 = Process(target=initialize_client, args=())
    p2 = Process(target=send_some_data, args=())

    p1.start()
    p2.start()

    p1.join()
    p2.join()