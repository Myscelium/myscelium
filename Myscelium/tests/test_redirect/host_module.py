from myscelium import MysceliumHost, HostPatterns, MysceliumHostInterface
from multiprocessing import Process, Event, Manager
from ..Logs.test_logs_mananger import Events_Mananger, System_Status
import os
import signal
import time

import time

# ctual_to_compare['ClientName'], actual_to_compare['ClientKey'], actual_to_compare['LastContact']

def client_contact_event_handler (client_name:str, client_key:str, client_last_contact:float):
    Events_Mananger(Unit="Host", path="Logs").Set_Event(Step=f"Contact received from Client: {client_key}")
    print(client_name, client_key, client_last_contact)
    pass

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
    def test_redirect(client_id, data):
        if isinstance(client_id, str):
            print(f"Redirecting data: {data} to client: {client_id}")
            host_patterns = HostPatterns()

            response_data = {'data':data}

            response = host_patterns.response_pattern(
                response=response_data,
                response_mode='redirect',
                redirect_to_client_id=client_id,
                response_activation_function="test_redirect_handler"
            )

            print(f"Response Before send to engine: {response}")

            return response
        else:
            print("Client id isn't a string, failed to redirect data!")
            return None

    # @staticmethod
    # def handle_client_contact(client_id, event_key='client_contact'):
    #     print("Access heartbeat handler")
    #     print(f"Client: {client_id}, made contact")

    #     Events_Mananger(Unit="Host", path="Logs").Set_Event(f"Contact received from Client: {client_id}")

    #     # TODO >>> Save event in the test databse log

    def monitor_stop_event(self):

        time.sleep(10)

        # -> Define how much time host will be alive!
        # TODO >>> In the future change to use 100% timeout
        n = 0 
        COUNTER = 62 # Each counter is 5 secs of waiting
        
        mys_host_interface = MysceliumHostInterface("Data/")

        mys_host_interface.set_client_contact_retriver_callback(client_contact_event_handler)

        mys_host_interface.start_client_events_retriver()
    
        while True:

            client_1_status = System_Status(path="Logs").get_unit_status(Unit="Client1")
            client_2_status = System_Status(path="Logs").get_unit_status(Unit="Client2")

            if (not client_1_status):
                print("Receive stop host from client 1")
                System_Status(path="Logs").change_unit_status(Unit="Client2", Status=False)
                mys_host_interface.stop_client_events_retriver()
                System_Status(path="Logs").change_unit_status(Unit="Host", Status=False)
                break
            
            elif (not client_2_status):
                print("Receive stop host from client 2")
                System_Status(path="Logs").change_unit_status(Unit="Client1", Status=False)
                mys_host_interface.stop_client_events_retriver()
                System_Status(path="Logs").change_unit_status(Unit="Host", Status=False)
                break

            else:
                time.sleep(5)
                n += 1
                continue

        return

    def run_host(self, ip, port):

        callbacks = [
            self.host_patterns.callback_pattern(callback=self.python_function,
                                                args={"birth": "str", "name": "str", "age": "int", "event_key": "str"}),
            self.host_patterns.callback_pattern(callback=self.test_redirect,
                                                args={"client_id": "str", "data": "int"}),
        ]

        allowed_clients = [
            self.host_patterns.client_pattern(client_name="TestClient1", client_type="Interface", client_key="some_client_id", client_permission_group="", client_is_super_user=True, client_max_sub_channes=5),
            self.host_patterns.client_pattern(client_name="TestClient2", client_type="Interface", client_key="randomsclientids", client_permission_group="", client_is_super_user=True, client_max_sub_channes=5),
        ]

        print(allowed_clients)

        # client_name:str, client_key:str, client_permission_group:str, client_is_super_user:bool, client_max_sub_channes:int, client_owned_sub_channels_keys:list

        mys_host = MysceliumHost(callbacks=callbacks, host_id="xnsmdkeflerpfsa",
                                 allowed_clients=allowed_clients, buffer_path="Data/", n_workers=2, log_level="INFO")

        self.mys_host = mys_host

        # client_heart_beat_handler = [self.host_patterns.callback_pattern(callback=self.handle_client_contact,
        #                                                                  args={"client_id": "str", "event_key": "str"}), ]

        # mys_host.set_client_heartbeat_handler(callback=client_heart_beat_handler)

        # TODO >>> Add callback handler to handle client contact (need to be like the logs transposer {Based on BufferDbTecnologie})

        System_Status(path="Logs").change_unit_status(Unit="Host", Status=True)

        mys_host.initialize_host(ip=ip, port=port)

        return

    def run(self, ip="127.0.0.1", port=4444, event=None):

        host_process = Process(target=self.run_host, args=(ip, port))
        monitor_process = Process(target=self.monitor_stop_event)

        host_process.start()
        monitor_process.start()

        monitor_process.join()

        # Send SIGINT to the process
        os.kill(host_process.pid, signal.SIGINT)

        host_process.join()

        return 

            


        

    

