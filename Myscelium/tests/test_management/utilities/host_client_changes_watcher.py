import sqlite3
import random
import os
import pandas as pd
import json
import time
from datetime import datetime
from queue import Queue
from threading import Lock, Thread

class SQLiteConnectionPool:
    def __init__(self, max_connections:int, database_path:str):

        self.data_base = database_path
        self.max_connections = max_connections
        self.connections = Queue(max_connections)
        self.lock = Lock()

        for i in range(max_connections):
            connection = sqlite3.connect(self.data_base, check_same_thread=False)
            self.connections.put(connection)

    def get_connection(self):
        with self.lock:
            if self.connections.empty():
                raise Exception("No available connections in the pool")
            connection = self.connections.get()
        return connection
    
    def release_connection(self, connection):
        with self.lock:
            self.connections.put(connection)

    def close_all_connections(self):
        while not self.connections.empty():
            connection = self.connections.get()
            connection.close()
        


class Clients_Retriever:

    def __init__(self, connection):
    
        self.connection = connection
    
        cur = self.connection.cursor()
        cur.execute('''CREATE TABLE IF NOT EXISTS Clients (ID INT PRIMARY KEY, 
                                                           ClientName TEXT, 
                                                           ClientKey TEXT, 
                                                           ClientType TEXT, 
                                                           PermissionGroup TEXT, 
                                                           SuperUser BOOL, 
                                                           LastContact NUMBER, 
                                                           MaxSubChannels NUMBER, 
                                                           OwnedSubChannelsKeys TEXT, 
                                                           SubChannelsInUse NUMBER,
                                                           Handlers TEXT
                                                            )''')

    def get_clients(self) -> dict:
        
        cur = self.connection.cursor()
        
        sqlite_select_query = """SELECT * FROM Clients"""
        
        cur.execute(sqlite_select_query)
        
        df = cur.fetchall()
        df = pd.DataFrame(df, columns=['ID', 'ClientName', 'ClientKey', 'ClientType', 'PermissionGroup', 'SuperUser', 'LastContact', 'MaxSubChannels', 'OwnedSubChannelsKeys', 'SubChannelsInUse', 'Handlers', 'Syncronized'])
        dict_df = df.to_dict()
        
        return dict_df    
    
class ClientsChangesTracker:
    
    def __init__(self, path_to_db) -> None:
        
        # Possible changes:
        # - ClientAdded
        # - Client Updated
        # - ClientRemotion
        
        self.change_expected
        self.change_track # tracs the changess done
        
        self.pool = SQLiteConnectionPool(
            1 + 2, path_to_db
        )
        
        # store old db
        self.old_df_before_changes
        
        pass
    
    def _get_clients (self):
        
        connection = self.pool.get_connection()
        
        retriever = Clients_Retriever(connection)
        clients = retriever.get_clients()
        
        self.pool.release_connection(connection)
        
        return clients
        
    def prepare_for_change (self, change_expected):
        
        self.old_df_before_changes = self._get_clients()
        self.change_expected = change_expected
        
        return
    
    def change_happened (self) -> bool:
        
        current_clients = self._get_clients()
        
        # TODO >>> Verify changes in relation to `self.old_df_before_changes` if find some change save in self.change_track and return true
        
        return False
    
# Example of usage

changes_tracker = ClientsChangesTracker("/path_to_db/Data.db")
changes_tracker.prepare_for_change()

# Do what needs to be done

changes_tracker.change_happened()

# ---

# This should work for case where we are managing clients by callbacks
# however when comes to internal functions this needs to be done without 
# any previously preparations.

# For this cases we can do a loop that do this process automatically and save into the events the cahnges

def client_manipulation_watcher (client_db__path):
    
    pool = SQLiteConnectionPool(
        1 + 2, os.path.join(client_db__path, "Data.db")
    )

    connection = pool.get_connection()
    
    retriever = Clients_Retriever(connection)
    
    # TODO >>> Create a mechanism to compare old dfs to new dfs from time to time and detect changes
    # > This doesn't need to be a loop, it can be like a CLASS that have a method 
    # > Lets say, check changes and we pass a change that it should check the old df to the new one
    
    pass