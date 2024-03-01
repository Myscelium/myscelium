from myscelium import MysceliumHost, HostPatterns, MysceliumHostInterface, CallbackCollector, callback_pattern
from multiprocessing import Process, Event, Manager
from ..Logs.test_logs_manager import Events_Manager, System_Status
import os
import signal
import time
import pandas as pd


from clients_retriever import SQLiteConnectionPool, Clients_Retriever

# ctual_to_compare['ClientName'], actual_to_compare['ClientKey'], actual_to_compare['LastContact']

# -> HANDLERS DEFINITION

def client_contact_event_handler (client_name:str, client_key:str, client_last_contact:float):
    
    # Contact handler is a special detached from core handler that works with database transposition to avoid 
    # Python interpreter multiprocess limitations like GIL dilema that don't allows to use smae process to call multiple
    # references of python internals from outside in safe way without drop anything in the way
    
    Events_Manager(Unit="Host", path="Logs").Set_Event(step=f"Contact received from Client: {client_key}")
    print(client_name, client_key, client_last_contact)
    pass

class Handlers:

    @staticmethod
    def test_add_client (info:dict, client_name:str, client_key:str, client_type:str, permission_group:str, is_super_user:bool, max_sub_channels:int, owned_sub_channels_keys:list):

        #! activation_function in this case is strictly defined as a internal function activated by callback responses

        new_client = HostPatterns().client_pattern(
                client_name=str(client_name),
                client_key=str(client_key),
                client_type=str(client_type),
                client_permission_group=str(permission_group),
                client_is_super_user=bool(is_super_user),
                max_sub_channels=int(max_sub_channels),
                owned_sub_channels_keys=list(owned_sub_channels_keys)
            )
        

        Events_Manager(Unit="Host", path="Logs").Set_Event(
            step="Active Test Add Client", 
            event_type="Receive", 
            event_key="94G2zy6cV54GN64O"
        )

        Events_Manager(Unit="Host", path="Logs").Set_Event(
            step=f"New Client: [{new_client}]"
        )

        response = HostPatterns().update_host_configs(activation_function="add_client", new_client=new_client)

        print(f"Response to send back to rust to inner management: {response}")

        return response
    
    @staticmethod
    def test_update_client (info:dict, actual_client_key:str,client_name:str, client_key:str, client_type:str, permission_group:str, is_super_user:bool, max_sub_channels:int, owned_sub_channels_keys:list):

        #! activation_function in this case is strictly defined as a internal function activated by callback responses

        updated_client = HostPatterns().client_pattern(
            client_name=str(client_name),
            client_key=str(client_key),
            client_type=str(client_type),
            client_permission_group=str(permission_group),
            client_is_super_user=bool(is_super_user),
            max_sub_channels=int(max_sub_channels),
            owned_sub_channels_keys=list(owned_sub_channels_keys)
        )	
        

        Events_Manager(Unit="Host", path="Logs").Set_Event(
            step="Active Test Update Client", 
            event_type="Receive", 
            event_key="3p7194Y33W6BnYlA"
        )

        Events_Manager(Unit="Host", path="Logs").Set_Event(
            step=f"Updated Client: [{updated_client}]"
        )
        
        return HostPatterns().update_host_configs(activation_function="update_client", actual_client_key=actual_client_key, updated_client=updated_client)

    @staticmethod
    def test_remove_client (info:dict, client_key:str):

        #! activation_function in this case is strictly defined as a internal function activated by callback responses

        Events_Manager(Unit="Host", path="Logs").Set_Event(
            step="Active Test Remove Client", 
            event_type="Receive", 
            event_key="30bt28u819A1QDpH"
        )

        Events_Manager(Unit="Host", path="Logs").Set_Event(
            step=f"Client Client: [{client_key}]"
        )

        return HostPatterns().update_host_configs(activation_function="remove_client", client_key=client_key)

# -> CLIENT DB CHANGES WATCHER

def client_changes_event_watcher (db_path):
    
    pool = SQLiteConnectionPool(
        1 + 2, db_path
    )

    previous_df = pd.DataFrame()

    while True:

        connection = pool.get_connection()
        retriever = Clients_Retriever(connection)
        current_df = retriever.get_clients
        
        # Ensure DataFrames are sorted by 'ID' for direct comparison
        dfA = previous_df.sort_values('ID').reset_index(drop=True)
        dfB = current_df.sort_values('ID').reset_index(drop=True)
        
        # Find added clients
        added = dfB[~dfB['ClientKey'].isin(dfA['ClientKey'])]

        # Find removed clients
        removed = dfA[~dfA['ClientKey'].isin(dfB['ClientKey'])]

        # Find common clients to check for updates
        common_a = dfA[dfA['ClientKey'].isin(dfB['ClientKey'])]
        common_b = dfB[dfB['ClientKey'].isin(dfA['ClientKey'])]

        # Sort to align for comparison
        common_a = common_a.sort_values('ClientKey').reset_index(drop=True)
        common_b = common_b.sort_values('ClientKey').reset_index(drop=True)

        # Check for updates by comparing rows
        updates = common_a[common_a != common_b].dropna(how='all')

        # Print messages
        for key in added['ClientKey']:

            Events_Manager(Unit="Host", path="Logs").Set_Event(
                step=f"Client key {key} was added.", 
                event_type="Receive", 
                event_key="30bt28u819A1QDpH"
            )
            
            print(f"Client key {key} was added.")

        for key in removed['ClientKey']:
            
            Events_Manager(Unit="Host", path="Logs").Set_Event(
                step=f"Client key {key} was removed.", 
                event_type="Receive", 
                event_key="30bt28u819A1QDpH"
            )
            
            print(f"Client key {key} was removed.")

        for index, row in updates.iterrows():
            
            Events_Manager(Unit="Host", path="Logs").Set_Event(
                step=f"Client key {common_a.at[index, 'ClientKey']} was updated.", 
                event_type="Receive", 
                event_key="30bt28u819A1QDpH"
            )
            
            print(f"Client key {common_a.at[index, 'ClientKey']} was updated.")
            
        time.sleep(2) #> Prevent thread overflow
    
    return

# -> HOST MAIN CLASS

class MyHost:

    def __init__(self, debug_level):
        self.host_patterns = HostPatterns()
        # self.my_callbacks = Handlers()
        self.debug_level = debug_level

    # @staticmethod
    # def handle_client_contact(client_id, event_key='client_contact'):
    #     print("Access heartbeat handler")
    #     print(f"Client: {client_id}, made contact")

    #     Events_Manager(Unit="Host", path="Logs").Set_Event(f"Contact received from Client: {client_id}")

    #     # TODO >>> Save event in the test database log

    def monitor_stop_event(self):

        time.sleep(5)

        # -> Define how much time host will be alive!
        # TODO >>> In the future change to use 100% timeout

        n = 0 
        # > In default conditions the test only can last 78 seconds (78/5 = 15,6) so round and add +1 for tolerance
        COUNTER = 17 # Each counter is 5 secs of waiting 

        mys_host_interface = MysceliumHostInterface("Temp/Data/")
        mys_host_interface.set_client_contact_retriever_callback(client_contact_event_handler)
        mys_host_interface.start_client_events_retriever()

        while True:

            client_status = System_Status(path="Logs").get_unit_status(Unit="Client1")

            if (not client_status) or (n >= COUNTER):
                
                print("Receive stop host")
                mys_host_interface.stop_client_events_retriever()
                
                System_Status(path="Logs").change_unit_status(
                    Unit="Host", 
                    Status=False
                )

                break

            else:
                time.sleep(5)
                n += 1
                continue

        return

    def run_host(self, ip, port):

        # callbacks = CallbackCollector([Handlers,]).get_callbacks()

        allowed_clients = [
            self.host_patterns.client_pattern(
                client_name="TestClient1", 
                client_type="Interface", 
                client_key="some_client_id", 
                client_permission_group="", 
                client_is_super_user=True, 
                max_sub_channels=5
            ),
            self.host_patterns.client_pattern(
                client_name="TestClient2", 
                client_type="Interface", 
                client_key="randomsclientids", 
                client_permission_group="", 
                client_is_super_user=True, 
                max_sub_channels=5
            ),
        ]

        print(allowed_clients)

        # client_name:str, client_key:str, client_permission_group:str, client_is_super_user:bool, max_sub_channels:int, client_owned_sub_channels_keys:list

        mys_host = MysceliumHost(
            callbacks=[], 
            host_id="xnsmdkeflerpfsa",
            allowed_clients=allowed_clients, 
            buffer_path="Temp/Data/", 
            n_workers=2, 
            log_level=self.debug_level
        )

        self.mys_host = mys_host

        # TODO >>> Add callback handler to handle client contact (need to be like the logs transposer {Based on BufferDbTecnologie})

        System_Status(path="Logs").change_unit_status(Unit="Host", Status=True)

        mys_host.initialize_host(ip=ip, port=port)

        return

    def run(self, ip="127.0.0.1", port=4444, event=None):

        host_process = Process(target=self.run_host, args=(ip, port))
        monitor_process = Process(target=self.monitor_stop_event)
        monitor_db_changes = Process(target=client_changes_event_watcher)
        
        host_process.start()
        monitor_process.start()
        monitor_db_changes.start()

        monitor_process.join()
        monitor_db_changes.kill() # Kill db monitor after monitor process resumes
        monitor_db_changes.join() # Join bellow to avoid infinit cicles

        # Send SIGINT to the process
        os.kill(host_process.pid, signal.SIGINT)

        host_process.join()

        return 
