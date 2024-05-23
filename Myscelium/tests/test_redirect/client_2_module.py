from myscelium import MysceliumClient, ClientPatterns, callback_pattern, CommandInstruction
import os
import time
import signal

client_patterns = ClientPatterns()

from multiprocessing import Process, Event, Manager
from ..Logs.test_logs_manager import Events_Manager, System_Status

CLIENT_ID = "randomsclientids"
CLIENT_NAME = "TestClient2"

def shutdown ():
    print("Receive order to stop client 2")
    System_Status(path="Logs").change_unit_status(Unit="Host", Status=False)
    System_Status(path="Logs").change_unit_status(
        Unit="Client2", Status=False
    )
    return

class Senders:
    @staticmethod
    def send_some_data_to_redirect():
        Events_Manager(Unit="Client2", path="Logs").Set_Event(
            "Try To Schedule Data To Redirect",
            event_type="Default",
        )

        print("Try to schedule!1")

        try:
            
            mys_client = MysceliumClient(
                name=CLIENT_NAME, client_uid=CLIENT_ID, buffer_path="Temp/Client2Data/", is_main_process = False
            )

            mys_client.running = True

            try: #! Here is required see if client is ready
                mys_client.ensure_client_ready(max_attempts=25, sleep_time=1)
            except Exception as e:
                Events_Manager(Unit="Client2", path="Logs").Set_Event(
                    f"{e}",
                    event_type="Default",
                )
                shutdown() 
                return

            Events_Manager(Unit="Client2", path="Logs").Set_Event(
                "Client 2 is ready",
                event_type="Default",
            )

            TARGET_KEY = "some_client_id"

            try: #! This require the target to be ready
                mys_client.ensure_target_ready(target_key=TARGET_KEY, max_attempts=10, sleep_time=10)
            except Exception as e: 
                Events_Manager(Unit="Client2", path="Logs").Set_Event(
                    f"{e}",
                    event_type="Default",
                )
                shutdown() 
                return

            Events_Manager(Unit="Client2", path="Logs").Set_Event(
                "Target Client 1 is ready",
                event_type="Default",
            )   
            
            # command = client_patterns.command_pattern(
            #     command_function="test_redirect_handler",
            #     target_key=TARGET_KEY,  # This is part of the smart redirect mechanism to redirect commands
            #     kwargs={"data": 8},
            #     response_type="ExternalFunction",
            #     response_target= "Origin",
            #     response_actf="", # Isn't defined a handler to this here yet
            # )

            command = CommandInstruction(
                command_mode="Function",
                command_type="ExternalFunction",
                command_target=f"ClientKey({TARGET_KEY})",
                command_status="Success",
                command_actf="test_redirect_handler",
                command_kwargs={"data": 8},
                command_message="",
                response_type="ExternalFunction",
                response_target="Origin",
                response_actf="test_handler",
                auto_collect_response=True,
            ).format()

            try:
                result = mys_client.send(command, priority=10)
            except ValueError as e:
                Events_Manager(Unit="Client2", path="Logs").Set_Event(
                    f"Error: {e}", event_type="Exception"
                )
                shutdown() 
                return
            
        except e as e:
            Events_Manager(Unit="Client2", path="Logs").Set_Event(
                f"{e}",
                event_type="Exception",
            )
            shutdown() 
            return

        Events_Manager(Unit="Client2", path="Logs").Set_Event(
            "Data To Redirect Sended",
            event_type="Send",
            event_key="02V0P37Dz09zR3fL",
        )

        print(result)


class Receivers:
    @staticmethod
    def test_handler(info: dict):
        Events_Manager(Unit="Client2", path="Logs").Set_Event(
            "Activate Basic Response Test callback handler"
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

        # time.sleep(5)

        # System_Status(path="Logs").change_unit_status(Unit="Client2", Status=False)


class MyClient:
    def __init__(self, debug_level):
        self.debug_level = debug_level

    def initializer(self):
        mys_client = MysceliumClient(
            name=CLIENT_NAME,
            client_uid=CLIENT_ID,
            buffer_path="Temp/Client2Data/",
            log_level=self.debug_level,
            is_main_process = True
        )

        self.mys_client = mys_client

        receivers = Receivers()

        callbacks = [
            callback_pattern(callback=receivers.test_handler),
        ]

        mys_client.set_callbacks(callbacks=callbacks)
        mys_client.set_workers_num(n_workers=2)

        System_Status(path="Logs").change_unit_status(Unit="Client2", Status=True)

        mys_client.initialize_client("127.0.0.1", 4444)

        return

    def monitor_stop_event(self):
        System_Status(path="Logs").change_unit_status(Unit="Client2", Status=True)

        time.sleep(30)

        while True:
            client_status = System_Status(path="Logs").get_unit_status(Unit="Client1")
            host_status = System_Status(path="Logs").get_unit_status(Unit="Host")

            if (not client_status) or (not host_status):
                print("Receive order to stop client 2")
                System_Status(path="Logs").change_unit_status(Unit="Host", Status=False)
                System_Status(path="Logs").change_unit_status(
                    Unit="Client2", Status=False
                )
                break
            else:
                time.sleep(5)
                continue

        return

    def run(self):
        senders = Senders()

        t1 = Process(target=self.initializer, args=())
        t2 = Process(target=senders.send_some_data_to_redirect, args=())
        t3 = Process(target=self.monitor_stop_event, args=())

        t1.start()

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
