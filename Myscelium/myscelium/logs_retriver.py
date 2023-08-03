# Movies Mananger bridge to sqlite 

import sqlite3
import random
import os
import pandas as pd
import json
from datetime import datetime


ThisFile    = os.path.dirname(__file__)
Data        = os.path.join(ThisFile, 'Data', 'Data.db')

# class DtrBufferID:

#     def __init__(self, length:int):
#         self.length = length        # length of BufferId
#         self.dtr = BufferDownInterface()

#     def Gen (self) -> int: # Gera um id para alocação dos dados no buffer de dados
#         GenBufferId = lambda: random.randint(0, self.length)
#         while True:
#             BufferId = GenBufferId()
#             if (self.Validate(BufferId)):
#                 break
#             else:
#                 pass
#         return BufferId

#     def Validate (self, BufferId:int) -> bool:  # Valida o id gerado e verifica se já existe, caso exista um id novoé gerado até que seja valido
#         DataList = [i[0] for i in self.dtr.list_schedule()]
#         # DataList = self.dtr.list_schedule.iloc[:, ['Id']].to_list()
#         for i in DataList:
#             if BufferId == i :
#                 return False
#             else:
#                 pass
#         return True

# Define the lock globally


import sqlite3
from  queue import Queue
from threading import Lock, Thread

class Interface_Unique_ID_Generator:

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
    def __init__ (self, max_connections):
        self.max_connections = max_connections
        self.connections = Queue (max_connections)
        self.lock = Lock()
        for i in range (max_connections):
            connection = sqlite3.connect(Data, check_same_thread=False)
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

pool = SQLiteConnectionPool(5)

def worker_1():

    #Get a connection from the pool and use it to execute a query
    connection = pool.get_connection()
    cursor = connection.cursor ()
    cursor.execute("SELECT * FROM your_table")

    result = cursor.fetchall()

    for row in result:
        print("Worker 1:", row)

    pool.release_connection(connection)

def worker_2():

    #Get a connection from the pool and use it to execute a query
    connection = pool.get_connection()
    cursor = connection.cursor ()
    cursor.execute("SELECT * FROM your_table")

    result = cursor.fetchall()

    for row in result:
        print("Worker 2:", row)

    pool.release_connection(connection) 

class Logs_Buffer_Retriver:

    def __init__ (self):

        self.ID_LENGHT = 999999

        self.AutoId = Interface_Unique_ID_Generator(self.ID_LENGHT, registred_ids = [])

        con = pool.get_connection()
        cur = con.cursor ()

        cur.execute('''CREATE TABLE IF NOT EXISTS Logs (ID INT PRIMARY KEY,
                                                               NodeName TEXT,
                                                               LogTime FLOAT,
                                                               LogName TEXT,
                                                               LogLevel TEXT,
                                                               LogMsg TEXT, 
                                                               )''')
        

        pool.release_connection(con) 

        return

    def List_Logs (self) -> dict:
        
        con = pool.get_connection()
        cur = con.cursor ()

        sqlite_select_query = """SELECT * FROM Logs"""

        cur.execute(sqlite_select_query)
        
        df = cur.fetchall()

        df = pd.DataFrame(df, columns=['ID', 'NodeName', 'LogTime', 'LogName', 'LogLevel', 'LogMsg'])

        # Convert JSON coluns to actual values and save in place on data-frame
        # df['File_Info']             = df['File_Info'].apply(lambda i: json.loads(i))

        dict_df = df.to_dict()

        pool.release_connection(con) 

        return dict_df

    def Remove_Log (self, ID:int):

        con = pool.get_connection()
        cur = con.cursor ()

        sql_update_query = """DELETE from Logs WHERE ID = ?"""
        cur.execute(sql_update_query, (ID, ))
        con.commit()

        pool.release_connection(con) 

        return  

