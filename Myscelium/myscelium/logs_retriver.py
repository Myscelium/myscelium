# Movies Mananger bridge to sqlite 

import sqlite3
import random
import os
import pandas as pd
import json
from datetime import datetime

import sqlite3
from  queue import Queue
from threading import Lock, Thread

class SQLiteConnectionPool:
    def __init__ (self, max_connections:int, database_path:str):

        self.data_base = database_path

        self.max_connections = max_connections
        self.connections = Queue (max_connections)
        self.lock = Lock()

        for i in range (max_connections):
            connection = sqlite3.connect(self.data_base, check_same_thread=False)
            self.connections.put(connection)

    def get_connection (self):
        with self.lock:
            if self.connections.empty():
                raise Exception("No avaliable connections in the pool")
            connection = self.connections.get()
        return connection
    
    def release_connection (self, connection):
        with self.lock:
            self.connections.put(connection)

class Logs_Buffer_Retriver:

    def __init__ (self, database_path:str):

        self.pool = SQLiteConnectionPool(5, database_path)

        con = self.pool.get_connection()
        cur = con.cursor ()

        cur.execute('''CREATE TABLE IF NOT EXISTS Logs (ID INT PRIMARY KEY,
                                                               NodeName TEXT,
                                                               LogTime FLOAT,
                                                               LogName TEXT,
                                                               LogLevel TEXT,
                                                               LogMsg TEXT, 
                                                               )''')

        self.pool.release_connection(con) 

        return

    def List_Logs (self) -> dict:
        
        con = self.pool.get_connection()
        cur = con.cursor ()

        sqlite_select_query = """SELECT * FROM Logs"""

        cur.execute(sqlite_select_query)
        
        df = cur.fetchall()

        df = pd.DataFrame(df, columns=['ID', 'NodeName', 'LogTime', 'LogName', 'LogLevel', 'LogMsg'])

        # Convert JSON coluns to actual values and save in place on data-frame
        # df['File_Info']             = df['File_Info'].apply(lambda i: json.loads(i))

        dict_df = df.to_dict()

        self.pool.release_connection(con) 

        return dict_df

    def Remove_Log (self, ID:int):

        con = self.pool.get_connection()
        cur = con.cursor ()

        sql_update_query = """DELETE from Logs WHERE ID = ?"""
        cur.execute(sql_update_query, (ID, ))
        con.commit()

        self.pool.release_connection(con) 

        return  

