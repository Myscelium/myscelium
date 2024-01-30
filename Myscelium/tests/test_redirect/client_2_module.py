from myscelium import MysceliumClient, ClientPatterns
import os
import time
import signal

client_patterns = ClientPatterns()

from multiprocessing import Process, Event, Manager
from ..Logs.test_logs_manager import Events_Manager, System_Status

CLIENT_ID = "randomsclientids"
CLIENT_NAME = "TestClient2"


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
                name=CLIENT_NAME, client_uid=CLIENT_ID, buffer_path="Temp/Client2Data/"
            )
            mys_client.running = True

            max_attempts = 25
            attemtps = 0
            while True:
                if mys_client.is_client_ready():
                    break

                if attemtps >= max_attempts:
                    assert False, "Take too long to client be ready"

                time.sleep(1)
                attemtps += 1
                continue

            Events_Manager(Unit="Client2", path="Logs").Set_Event(
                "Client 2 is ready",
                event_type="Default",
            )

            TARGET_KEY = "some_client_id"

            target_attempts = 0
            while True:
                try:
                    ready = mys_client.is_target_ready(target_key=TARGET_KEY)
                    print(f"Target is ready? {ready}")
                    if ready:
                        break
                except e as e:
                    Events_Manager(Unit="Client2", path="Logs").Set_Event(
                        f"Error trying to verify if target Client 2 is ready, error was: {e}",
                        event_type="Default",
                    )
                    pass

                else:
                    pass

                if target_attempts >= 120:
                    Events_Manager(Unit="Client2", path="Logs").Set_Event(
                        f"Take too long to target be ready!",
                        event_type="Default",
                    )
                    assert False, "Take too long to target be ready!"

                Events_Manager(Unit="Client2", path="Logs").Set_Event(
                    "target isn't ready, so trying again in 5 secs",
                    event_type="Default",
                )

                print("target isn't ready, so trying again in 5 secs")

                time.sleep(1)

                target_attempts += 1
                continue

            Events_Manager(Unit="Client2", path="Logs").Set_Event(
                "Target Client 1 is ready",
                event_type="Default",
            )

            command = client_patterns.command_pattern(
                CLIENT_ID,
                "test_redirect_handler",
                target_key=TARGET_KEY,  # This is part of the smart redirect mechanism to redirect commands
                kwargs={"data": 8},
            )

            result = mys_client.send(command, priority=10)
        except e as e:
            Events_Manager(Unit="Client2", path="Logs").Set_Event(
                f"{e}",
                event_type="Exception",
            )

        Events_Manager(Unit="Client2", path="Logs").Set_Event(
            "Data To Redirect Scheduled",
            event_type="Send",
            event_key="02V0P37Dz09zR3fL",
        )

        print(result)


class Receivers:
    @staticmethod
    def test_handler(data: str):
        Events_Manager(Unit="Client2", path="Logs").Set_Event(
            "Activate Basic Response Test callback handler"
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
        )

        self.mys_client = mys_client

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
