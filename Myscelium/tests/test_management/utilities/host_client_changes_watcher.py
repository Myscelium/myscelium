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
            
def client_manipulation_watcher (client_db__path):
    
    pool = SQLiteConnectionPool(
        1 + 2, os.path.join(client_db__path, "Data.db")
    )

    connection = pool.get_connection()
    
    retriever = Clients_Retriever(connection)
    
    pass