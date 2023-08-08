from myscelium import MysceliumHost, HostPatterns
from multiprocessing import Process, Event, Manager
from .Logs.test_logs_mananger import Events_Mananger, System_Status

import time

class MyHost:

    def __init__(self):
        self.host_patterns = HostPatterns()

    @staticmethod
    def python_function(age, birth, name):
        print("Access python function")
        print(birth)
        print(name)
        print(age)

        host_patterns = HostPatterns()
        response = host_patterns.response_pattern(
            response_mode='to_origin',
            response_activation_function="test_handler",
            response={"data": 'hello!'}
        )

        Events_Mananger(Unit="Host", path="Logs").Set_Event(Step="Active Basic Callback")
        Events_Mananger(Unit="Host", path="Logs").Set_Event(Step=f"Base callback - Receive Data: [{age}, {birth}, {name}]")

        #                                                            (callback name) - Receive Data: [Data received list for comparison]

        return response

    @staticmethod
    def test_redirect(client_id, data, event_key=None):
        if isinstance(client_id, str):
            print(f"Redirecting data: {data} to client: {client_id}")
            host_patterns = HostPatterns()
            response = host_patterns.response_pattern(
                response=data,
                response_mode='redirect',
                redirect_to_client_id=client_id
            )
            return response
        else:
            print("Client id isn't a string, failed to redirect data!")
            return None

    @staticmethod
    def handle_client_contact(client_id, event_key='client_contact'):
        print("Access heartbeat handler")
        print(f"Client: {client_id}, made contact")

        Events_Mananger(Unit="Host", path="Logs").Set_Event(f"Contact received from Client: {client_id}")

        # TODO >>> Save event in the test databse log

    def monitor_stop_event(self):

        time.sleep(5)
        
        while True:

            client_status = System_Status(path="Logs").get_unit_status(Unit="Client")

            if not client_status:
                print("Receive stop host")
                System_Status(path="Logs").change_unit_status(Unit="Host", Status=False)
                break
            else:
                time.sleep(5)
                continue

        return

    def run_host(self, ip, port):
        callbacks = [
            self.host_patterns.callback_pattern(callback=self.python_function,
                                                args={"birth": "str", "name": "str", "age": "int", "event_key": "str"}),
            self.host_patterns.callback_pattern(callback=self.test_redirect,
                                                args={"client_id": "str", "data": "dict", "event_key": "str"}),
        ]

        allowed_clients = [
            self.host_patterns.client_pattern(client_type="Interface", client_id="some_client_id"),
            self.host_patterns.client_pattern(client_type="Interface", client_id="randomsclientids"),
        ]

        mys_host = MysceliumHost(callbacks=callbacks, host_id="xnsmdkeflerpfsa",
                                 allowed_clients=allowed_clients, buffer_path="Data/", n_workers=2)

        self.mys_host = mys_host

        client_heart_beat_handler = [self.host_patterns.callback_pattern(callback=self.handle_client_contact,
                                                                         args={"client_id": "str", "event_key": "str"}), ]

        mys_host.set_client_heartbeat_handler(callback=client_heart_beat_handler)

        System_Status(path="Logs").change_unit_status(Unit="Host", Status=True)

        mys_host.initialize_host(ip=ip, port=port)

    def run(self, ip="127.0.0.1", port=4444, event=None):
        

        host_process = Process(target=self.run_host, args=(ip, port))
        monitor_process = Process(target=self.monitor_stop_event)

        host_process.start()
        monitor_process.start()

        monitor_process.join()

        host_process.kill()


        return 

            


        

    

