# Controls the tests history logs

# TODO >>> Make a history controler in a db to mark the history of the tests 

import sqlite3
import random
import os
import pandas as pd
import json
import time
from queue import Queue
from threading import Lock, Thread
from datetime import datetime

class Interface_Unique_ID_Generator:

    # There is a big mistake here, when we are deling with IDs we need to consider multithreading, so ids need to be stored in a global since 
    # the first sinc with the ids registred in the database, this because we need to ensure that we don't have two identiacal ids at once
    # what could cause an error in the database, and to don't have the possibility to do this basically we need to have a global list with the 
    # ids sincronized with the ids in the db, and to earchive this the ideal is to have a list that sinc with the db and then every new id
    # generated can be added to this list, in a way that each new input will have to check the registred ids in a ordenated way, without two 
    # being able to do this at once.

    def __init__(self, length:int, registred_ids:list):
        self.length = length        # length of BufferId
        self.registred_ids = registred_ids

    def Update_Registred_Ids (self, registred_ids:list):
        self.registred_ids = registred_ids
        return

    def Gen (self) -> int: # Gera um id para alocação dos dados no buffer de dados
        GenBufferId = lambda: random.randint(0, self.length)
        while True:
            BufferId = GenBufferId()
            if (self.Validate(BufferId)):
                break
            else:
                pass
        return BufferId

    def Validate (self, BufferId:int) -> bool:  # Valida o id gerado e verifica se já existe, caso exista um id novoé gerado até que seja valido
        DataList = [i[0] for i in self.registred_ids]
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

class History_Mannanger:

    # > Date Time |        Test          | Communication speed | Test Speed | Test Status   
    #   102392039 |  test communication  |         1s          |     65s    |   PASSED


    def __init__(self): 

        pool = SQLiteConnectionPool(3, os.path.join("Data", "History.db"))
        self.connection = pool.get_connection()

        self.AutoId = Interface_Unique_ID_Generator(length=9999, registred_ids=[])

        cur = self.connection.cursor()
        cur.execute('''CREATE TABLE IF NOT EXISTS History (ID INT PRIMARY KEY, Time NUMBER, TestName TEXT, CommunicationSpeed NUMBER, TestSpeed NUMBER, TestStatus TEXT)''')
        
        
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
        df = pd.DataFrame(df, columns=['ID', 'Time', 'TestName', 'CommunicationSpeed', 'TestSpeed', 'TestStatus'])
        dict_df = df.to_dict()
        
        return dict_df
    
    def store_history_point (self, test_name:str, communications_speed:float, test_speed:float, test_status:str):    

        cur = self.connection.cursor()
        
        self.AutoId.Update_Registred_Ids(registred_ids = self.list_keys())

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

        ID = self.AutoId.Gen() 
        ts = time.time()

        sqlite_insert_with_param = """INSERT INTO Keys (ID, Time, TestName, CommunicationSpeed, TestSpeed, TestStatus) VALUES (?, ?, ?, ?, ?, ?);"""
        cur.execute(sqlite_insert_with_param, (ID, ts, test_name, communications_speed, test_speed, test_status))
        self.connection.commit()

        return
    
    # def remove_unique_key (self, key:str):

    #     cur = self.connection.cursor()

    #     keys_dict_df = pd.DataFrame.from_dict(self.list_keys())

    #     keys = keys_dict_df["EventKey"].to_list()

    #     if not (key in keys):
    #         print (f"Key {key} alwready not registred!\n")

    #         return
        
    #     else:
    #         pass
        
    #     sqlite_insert_with_param = """DELETE FROM Keys WHERE EventKey = ?"""
    #     cur.execute(sqlite_insert_with_param, (key, ))
    #     self.connection.commit()

    #     print ("Key successfully deleted!\n")