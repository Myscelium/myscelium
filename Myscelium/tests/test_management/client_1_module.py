from myscelium import MysceliumClient, ClientPatterns, CallbackCollector, callback_pattern
import os
import time
import signal

client_patterns = ClientPatterns()

from multiprocessing import Process, Event, Manager
from ..Logs.test_logs_manager import Events_Manager, System_Status

CLIENT_ID = "some_client_id"


class Receivers:
    @staticmethod
    def add_client_handler(info: dict):  # -> Need to be implemented
        # "data" {
        #     "command_type":"response",
        #     "status": "Success"
        #     "response_activation_function":"",
        #     "message":"",
        #     "kwargs":{"arg1": [], "arg2": "", "arg3": {}}
        #     "response_mode":"",
        # }

        Events_Manager(Unit="Client1", path="Logs").Set_Event(
            "Activate Basic Response Test Add Client"
        )

        print("Received data in python callback: ", info)

        if "status" in info:
            pass
        else:
            return None  # This return that the callback called don't have a response for this case

        if info["status"] == "Success":
            pass
        else:
            return None

    @staticmethod
    def update_client_handler(info: dict):  # -> Need to be implemented
        Events_Manager(Unit="Client1", path="Logs").Set_Event(
            "Activate Basic Response Test Update Client"
        )

        if "status" in info:
            pass
        else:
            return None

        if info["status"] == "Success":
            pass
        else:
            return None

        print("Received data: ", info)

        # System_Status(path="Logs").change_unit_status(Unit="Client1", Status=False)

    @staticmethod
    def remove_client_handler(info: dict):  # TODO >>> test_remove_client
        Events_Manager(Unit="Client1", path="Logs").Set_Event(
            "Activate Basic Response Test Remove Client"
        )

        if "status" in info:
            pass
        else:
            return None

        if info["status"] == "Success":
            pass
        else:
            return None

        print("Received data: ", info)

        time.sleep(10)

        System_Status(path="Logs").change_unit_status(Unit="Client1", Status=False)


class Senders:
    def start_send_sequence(self):
        time.sleep(15)
        self.test_add_client()
        time.sleep(10) 
        
        # This 5 seconds dellay is necessary to the client db changes event watcher have time to detect changes 
        # without consume too much ressources by have to use more fast refresh loops
        
        self.test_update_client()
        time.sleep(10)
        
        # This 5 seconds dellay is necessary to the client db changes event watcher have time to detect changes 
        # without consume too much ressources by have to use more fast refresh loops
        
        self.test_remove_client()

    # > -------------------------------------------------------------------------------------------------------------------------------------
    # > USING DIRECT MANAGEMENT FUNCTIONS:
    

    @staticmethod
    def test_add_client():
        
        #> Client            Host
        #>   |                |
        #>   |--------------> | Host receives add client order 
        #>   |                |
        #>   |               (|) Add client (verification needs to be done here)
        #>   |                |
        #>   |<---------------| Send a confirmation back
        #>   |                |
        
        # TODO >>> Centralize this Myscelium class
        mys_client = MysceliumClient(
            name="TestClient1",
            client_uid="some_client_id",
            buffer_path="Temp/Client1Data/",
            is_main_process = False
        )
        mys_client.running = True

        max_attempts = 10
        attemtps = 0
        while not mys_client.is_client_ready():
            time.sleep(1)
            attemtps += 1
            if attemtps >= max_attempts:
                assert False, "Take too long to client be ready"
            continue
        
        command = client_patterns.inner_management_command_pattern(
            origin_key=CLIENT_ID,  # origin
            command_function="add_client",  # actf
            kwargs={
                "client_name": "test_client",
                "client_key": "xMndjslwpedcnfe",
                "client_type": "Test",
                "permission_group": "",
                "is_super_user": True,
                "max_sub_channels": 5,
                "owned_sub_channels_keys": [],
            },
            response_type="ExternalFunction",
            response_target = "Origin",
            response_actf="add_client_handler",
        )

        _ = mys_client.send(command, priority=9)

        Events_Manager(Unit="Client1", path="Logs").Set_Event(
            "Send test add a client", event_type="Send", event_key="94G2zy6cV54GN64O"
        )

    @staticmethod
    def test_update_client():
        
        #> Client            Host
        #>   |                |
        #>   |--------------> | Host receives update client order 
        #>   |                |
        #>   |               (|) Update client (verification needs to be done here)
        #>   |                |
        #>   |<---------------| Send a confirmation back
        #>   |                |
        
        # TODO >>> Centralize this Myscelium class
        mys_client = MysceliumClient(
            name="TestClient1",
            client_uid="some_client_id",
            buffer_path="Temp/Client1Data/",
            is_main_process = False
        )
        mys_client.running = True

        max_attempts = 10
        attemtps = 0
        while not mys_client.is_client_ready():
            time.sleep(1)
            attemtps += 1
            if attemtps >= max_attempts:
                assert False, "Take too long to client be ready"
            continue
        
        #! HERE SINCE IS A TEST THE TEST REQUIRES KEY TO BE CONSTANT, 
        #! SO NEVER CHANGES IT IN TEST CASES BECAUSE CHANGES MAY NOT BE DETECTED BY EVENT WATCHER
        
        #> Changes done:
        # Change client name
        # Turn super user to false
                
        command = client_patterns.inner_management_command_pattern(
            CLIENT_ID,  # origin
            "update_client",  # actf
            kwargs={
                "actual_client_key": "xMndjslwpedcnfe",
                "updated_client": {
                    "client_key": "xMndjslwpedcnfe", 
                    "client_name": "changed_test_client", 
                    "client_type": "Test",
                    "permission_group": "",
                    "is_super_user": False, 
                    "max_sub_channels": 10,
                    "owned_sub_channels_keys": [],
                },
            },
            response_type="ExternalFunction",
            response_target = "Origin",
            response_actf="update_client_handler",
        )

        _ = mys_client.send(command, priority=8)

        Events_Manager(Unit="Client1", path="Logs").Set_Event(
            "Send test update a client", event_type="Send", event_key="3p7194Y33W6BnYlA"
        )

    @staticmethod
    def test_remove_client():
        
        #> Client            Host
        #>   |                |
        #>   |--------------> | Host receives remove client order 
        #>   |                |
        #>   |               (|) Remove client (verification needs to be done here)
        #>   |                |
        #>   |<---------------| Send a confirmation back
        #>   |                |
        
        # TODO >>> Centralize this Myscelium class
        mys_client = MysceliumClient(
            name="TestClient1",
            client_uid="some_client_id",
            buffer_path="Temp/Client1Data/",
            is_main_process = False
        )
        mys_client.running = True

        max_attempts = 10
        attemtps = 0
        while not mys_client.is_client_ready():
            time.sleep(1)
            attemtps += 1
            if attemtps >= max_attempts:
                assert False, "Take too long to client be ready"
            continue
            
        command = client_patterns.inner_management_command_pattern(
            origin_key=CLIENT_ID, 
            command_function="remove_client", 
            kwargs={"client_key": "xMndjslwpedcnfe"},
            response_type="ExternalFunction",
            response_target = "Origin",
            response_actf="remove_client_handler",
        )
        
        _ = mys_client.send(command, priority=7)

        Events_Manager(Unit="Client1", path="Logs").Set_Event(
            "Send test remove a client", event_type="Send", event_key="30bt28u819A1QDpH"
        )

    # > -------------------------------------------------------------------------------------------------------------------------------------
    # > USING EXTERNAL FUNCTIONS RESPONSE

    @staticmethod
    def test_add_client_from_response():
        # TODO >>> Centralize this Myscelium class
        mys_client = MysceliumClient(
            name="TestClient1",
            client_uid="some_client_id",
            buffer_path="Temp/Client1Data/",
            is_main_process = False
        )
        mys_client.running = True

        max_attempts = 10
        attemtps = 0
        while not mys_client.is_client_ready():
            time.sleep(1)
            attemtps += 1
            if attemtps >= max_attempts:
                assert False, "Take too long to client be ready"
            continue

        command = client_patterns.inner_management_command_pattern(
            origin_key=CLIENT_ID,  # origin
            command_function="test_add_client",  # actf
            kwargs={
                "client_name": "test_client",
                "client_key": "xMndjslwpedcnfe",
                "client_type": "Test",
                "permission_group": "",
                "is_super_user": True,
                "max_sub_channels": 5,
                "owned_sub_channels_keys": [],
            },
        )

        _ = mys_client.send(command, priority=9)

        Events_Manager(Unit="Client1", path="Logs").Set_Event(
            "Send test add a client", event_type="Send", event_key="94G2zy6cV54GN64O"
        )

    @staticmethod
    def test_update_client_from_response():
        # TODO >>> Centralize this Myscelium class
        mys_client = MysceliumClient(
            name="TestClient1",
            client_uid="some_client_id",
            buffer_path="Temp/Client1Data/",
            is_main_process = False
        )
        mys_client.running = True

        max_attempts = 10
        attemtps = 0
        while not mys_client.is_client_ready():
            time.sleep(1)
            attemtps += 1
            if attemtps >= max_attempts:
                assert False, "Take too long to client be ready"
            continue

        command = client_patterns.inner_management_command_pattern(
            origin_key=CLIENT_ID,  # origin
            command_function="test_update_client",  # actf
            kwargs={
                "actual_client_key": "xMndjslwpedcnfe",
                "client_key": "xMndjslwpedcnfe",
                "client_name": "test_client",
                "client_type": "Test",
                "permission_group": "",
                "is_super_user": True,
                "max_sub_channels": 10,
                "owned_sub_channels_keys": [],
            },
        )

        _ = mys_client.send(command, priority=8)

        Events_Manager(Unit="Client1", path="Logs").Set_Event(
            "Send test update a client", event_type="Send", event_key="3p7194Y33W6BnYlA"
        )

    @staticmethod
    def test_remove_client_from_response():
        # TODO >>> Centralize this Myscelium class
        mys_client = MysceliumClient(
            name="TestClient1",
            client_uid="some_client_id",
            buffer_path="Temp/Client1Data/",
            is_main_process = False
        )

        mys_client.running = True

        max_attempts = 10
        attemtps = 0
        while not mys_client.is_client_ready():
            time.sleep(1)
            attemtps += 1
            if attemtps >= max_attempts:
                assert False, "Take too long to client be ready"
            continue

        command = client_patterns.inner_management_command_pattern(
            CLIENT_ID, 
            "test_remove_client", 
            kwargs={"client_key": "xMndjslwpedcnfe"}
        )

        _ = mys_client.send(command, priority=7)

        Events_Manager(Unit="Client1", path="Logs").Set_Event(
            "Send test remove a client", event_type="Send", event_key="30bt28u819A1QDpH"
        )


class MyClient:#
    def __init__(self, debug_level):
        self.debug_level = debug_level

    def initializer(self):
        mys_client = MysceliumClient(
            name="TestCient1",
            client_uid=CLIENT_ID,
            buffer_path="Temp/Client1Data/",
            log_level=self.debug_level,
            is_main_process = True
        )

        self.mys_client = mys_client

        callbacks = CallbackCollector(
            [
                Receivers,
            ]
        ).get_callbacks()

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
