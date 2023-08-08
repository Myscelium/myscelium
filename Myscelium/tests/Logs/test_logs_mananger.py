import sqlite3
import random
import os
import pandas as pd
import json
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

class Logs_Buffer_Retriver:

    def __init__(self, connection):
    
        self.connection = connection
    
        cur = self.connection.cursor()
        cur.execute('''CREATE TABLE IF NOT EXISTS Logs (ID INT PRIMARY KEY,
                                                        NodeName TEXT,
                                                        LogTime FLOAT,
                                                        LogName TEXT,
                                                        LogLevel TEXT,
                                                        LogMsg TEXT 
                                                        )''')

    def List_Logs(self) -> dict:
        
        cur = self.connection.cursor()
        
        sqlite_select_query = """SELECT * FROM Logs"""
        
        cur.execute(sqlite_select_query)
        
        df = cur.fetchall()
        df = pd.DataFrame(df, columns=['ID', 'NodeName', 'LogTime', 'LogName', 'LogLevel', 'LogMsg'])
        dict_df = df.to_dict()
        
        return dict_df

    def Remove_Log(self, ID:int):
        
        cur = self.connection.cursor()
        
        sql_update_query = """DELETE from Logs WHERE ID = ?"""
        
        cur.execute(sql_update_query, (int(ID),))
        
        self.connection.commit()

def transpose(logs_df, buffer_path, log_callback):
    pool = SQLiteConnectionPool(2, os.path.join(buffer_path, "Logs.db"))
    connection = pool.get_connection()
    logs_retriever_access = Logs_Buffer_Retriver(connection)

    for i in logs_df.index:
        try:
            log_id = logs_df.loc[i, 'ID']
            log_time = logs_df.loc[i, 'LogTime']
            log_from_node = logs_df.loc[i, 'NodeName']
            log_level = logs_df.loc[i, 'LogLevel']
            log_msg = logs_df.loc[i, 'LogMsg']

            log_callback({"log_time": log_time, "log_level": log_level, "log_from_node": log_from_node, "log_msg": log_msg})
        except:
            pass

        logs_retriever_access.Remove_Log(log_id)
        continue

    pool.release_connection(connection)
    return