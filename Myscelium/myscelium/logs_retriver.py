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

class Clients_SQL_Interface:

    def __init__ (self):

        self.ID_LENGHT = 999999

        self.AutoId = Interface_Unique_ID_Generator(self.ID_LENGHT, registred_ids = [])

        con = pool.get_connection()
        cur = con.cursor ()

        cur.execute('''CREATE TABLE IF NOT EXISTS Clients (ID INT PRIMARY KEY,
                                                               NAME TEXT,
                                                               KEY TEXT,
                                                               TYPE TEXT,
                                                               LASTCONTACT FLOAT
                                                               )''')
        

        pool.release_connection(con) 

        return

    def List_Clients (self) -> dict:
        
        con = pool.get_connection()
        cur = con.cursor ()

        sqlite_select_query = """SELECT * FROM Clients"""

        cur.execute(sqlite_select_query)
        
        df = cur.fetchall()

        df = pd.DataFrame(df, columns=['ID', 'Name', 'Key', 'Type', 'LastContact'])

        # Convert JSON coluns to actual values and save in place on data-frame
        # df['File_Info']             = df['File_Info'].apply(lambda i: json.loads(i))

        dict_df = df.to_dict()

        pool.release_connection(con) 

        return dict_df

    def Registry_New_Client (self, Name:str, Key:str, Type:str):

        con = pool.get_connection()
        cur = con.cursor ()

        self.AutoId.Update_Registred_Ids(registred_ids = self.List_Clients())

        ID = self.AutoId.Gen()

        # getting the timestamp
        LastContact = None

        Data = ((ID, Name, Key, Type, LastContact))

        sqlite_insert_with_param = """INSERT INTO Clients (ID, NAME, KEY, TYPE, LastContact) VALUES (?, ?, ?, ?, ?);"""
        cur.execute(sqlite_insert_with_param, Data)
        con.commit()

        pool.release_connection(con) 

        return
        
    def Update_Client (self, ID:int, Name:str, Key:str, Type:str): # TODO >>> Need veryfi the part that uses ID to update

        con = pool.get_connection()
        cur = con.cursor ()

        # Getting the current date and time
        dt = datetime.now()

        # getting the timestamp
        LastContact =  datetime.timestamp(dt)

        Data = (Name, Key, Type, LastContact, ID)

        sql_update_query = f"""Update Clients set NAME = ?, KEY = ?, TYPE = ?, LastContact = ? WHERE ID = ?"""
      
        cur.execute(sql_update_query, Data)
        con.commit()
        
        pool.release_connection(con) 

        return
    
    def Update_Client_Ts (self, Key:str):

        con = pool.get_connection()
        cur = con.cursor ()

        # Getting the current date and time
        dt = datetime.now()

        # getting the timestamp
        LastContact =  datetime.timestamp(dt)

        Data = (LastContact, Key)

        sql_update_query = f"""Update Clients set LastContact = ? WHERE KEY = ?"""
      
        cur.execute(sql_update_query, Data)
       
        con.commit()

        pool.release_connection(con) 

        return

    def Remove_Client_By_Id (self, ID:int):

        con = pool.get_connection()
        cur = con.cursor ()

        sql_update_query = """DELETE from Clients WHERE ID = ?"""
        cur.execute(sql_update_query, (ID, ))
        con.commit()

        pool.release_connection(con) 

        return  

    def Remove_Client_By_Key (self, Key:str):

        con = pool.get_connection()
        cur = con.cursor ()

        print (f"[Buffer][ClientInterface] - Removing Client by Key: {Key}")

        sql_update_query = """DELETE from Clients where KEY = ?"""
        cur.execute(sql_update_query, (Key, ))
        con.commit()

        pool.release_connection(con) 

        return  
