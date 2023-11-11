# Controls the tests history logs

# TODO >>> Make a history controller in a db to mark the history of the tests 

import sqlite3
import random
import os
import pandas as pd
import json
import time
from queue import Queue
from threading import Lock, Thread
import datetime

class Interface_Unique_ID_Generator:

    # There is a big mistake here, when we are dealing with IDs we need to consider multithreading, so ids need to be stored in a global since 
    # the first sync with the ids registered in the database, this because we need to ensure that we don't have two identical ids at once
    # what could cause an error in the database, and to don't have the possibility to do this basically we need to have a global list with the 
    # ids synchronized with the ids in the db, and to archive this the ideal is to have a list that sync with the db and then every new id
    # generated can be added to this list, in a way that each new input will have to check the registered ids in a ordered way, without two 
    # being able to do this at once.

    def __init__(self, length:int, registered_ids:list):
        self.length = length        # length of BufferId
        self.registered_ids = registered_ids

    def Update_registered_ids (self, registered_ids:list):
        self.registered_ids = registered_ids
        return

    def Gen (self) -> int: # Gen a id to allocate data in the buffer.
        GenBufferId = lambda: random.randint(0, self.length)
        while True:
            BufferId = GenBufferId()
            if (self.Validate(BufferId)):
                break
            else:
                pass
        return BufferId

    def Validate (self, BufferId:int) -> bool:  # Validate the id generated and see if it already exists, if so gen other id. 
        # DataList = self.dtr.list_schedule.iloc[:, ['Id']].to_list()
        for i in DataList:
            if BufferId == i :
                return False
            else:
                pass
        return True
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

THIS_DIR = os.path.dirname(__file__)
DATABASE_PATH =os.path.join(THIS_DIR, "Data")

if not os.path.exists(DATABASE_PATH):
    os.makedirs(DATABASE_PATH)
else:
    pass

DATABASE = os.path.join(DATABASE_PATH, "History.db")


class History_Manager:

    # > Date Time |        Test          | Communication speed | Test Speed | Test Status   
    #   102392039 |  test communication  |         1s          |     65s    |   PASSED


    def __init__(self): 

        pool = SQLiteConnectionPool(3, DATABASE)
        self.connection = pool.get_connection()

        self.AutoId = Interface_Unique_ID_Generator(length=9999999999, registered_ids=[])

        cur = self.connection.cursor()
        cur.execute('''CREATE TABLE IF NOT EXISTS History (ID INT PRIMARY KEY, Time NUMBER, TestName TEXT, CommunicationSpeed NUMBER, TestSpeed NUMBER, TestStatus TEXT, LogLevel TEXT)''')
        
        
    def drop_events_table(self) -> None:
        """Drop the History table from the database."""
    
        try:
            cur = self.connection.cursor()
            
            sql_drop_table_query = """DROP TABLE IF EXISTS History"""
            cur.execute(sql_drop_table_query)
            
            self.connection.commit()
            print("History table has been removed.")
            
        except Exception as e:
            print(f"Error occurred while removing the History table. Error: {e}")


    def list_history (self) -> dict:
        
        cur = self.connection.cursor()
        
        sqlite_select_query = """SELECT * FROM History"""
        
        cur.execute(sqlite_select_query)
        
        df = cur.fetchall()
        df = pd.DataFrame(df, columns=['ID', 'Time', 'TestName', 'CommunicationSpeed', 'TestSpeed', 'TestStatus', 'LogLevel'])
        dict_df = df.to_dict()
        
        return dict_df
    
    def store_history_point (self, test_name:str, communications_speed:float, test_speed:float, test_status:str, log_level:str):    

        cur = self.connection.cursor()
        
        self.AutoId.Update_registered_ids(registered_ids = self.list_history())

        if not isinstance(test_name, str):
            raise "test_name needs to be a string with the test name!"

        if not isinstance(communications_speed, float):
            raise "communications_speed needs to be a float!"
        
        if not isinstance(test_speed, float):
            raise "test_speed needs to be a float!"
        
        if not isinstance(test_status, str):
            raise "test_status needs to be a str!"
        
        if not (test_status in ['PASSED', 'FAILED']):
            raise "test_status needs to be a str 'PASSED' or 'FAILED'!"
        
        if not isinstance(log_level, str):
            raise "log_level needs to be a str!"
        
        if not (log_level in ['EXCEPTION', 'WARN', 'INFO', 'DEBUG']):
            raise "test_status needs to be a str like: 'EXCEPTION', 'WARN', 'INFO' or 'DEBUG'!"

        ID = self.AutoId.Gen() 
        ts = datetime.datetime.now()

        sqlite_insert_with_param = """INSERT INTO History (ID, Time, TestName, CommunicationSpeed, TestSpeed, TestStatus, LogLevel) VALUES (?, ?, ?, ?, ?, ?, ?);"""
        cur.execute(sqlite_insert_with_param, (ID, ts.timestamp(), test_name, communications_speed, test_speed, test_status, log_level))
        self.connection.commit()

        return
    
    # def remove_unique_key (self, key:str):

    #     cur = self.connection.cursor()

    #     keys_dict_df = pd.DataFrame.from_dict(self.list_history())

    #     keys = keys_dict_df["EventKey"].to_list()

    #     if not (key in keys):
    #         print (f"Key {key} already not registered!\n")

    #         return
        
    #     else:
    #         pass
        
    #     sqlite_insert_with_param = """DELETE FROM History WHERE EventKey = ?"""
    #     cur.execute(sqlite_insert_with_param, (key, ))
    #     self.connection.commit()

    #     print ("Key successfully deleted!\n")