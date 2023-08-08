import sqlite3
import random
import os
import pandas as pd
import json
from datetime import datetime
from queue import Queue
from threading import Lock, Thread

# TODO >>> Create a client table to set the logs and the client state and the host state
# TODO >>> if the host or client state in the table was set to false it will close the host or the client

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

class Events_Mananger:

    def __init__(self, Unit:str, path:str):

        pool = SQLiteConnectionPool(3, os.path.join(path, "Data.db"))
        self.connection = pool.get_connection()

        self.Unit = Unit

        self.AutoId = Interface_Unique_ID_Generator(length=9999, registred_ids=[])
    
        cur = self.connection.cursor()
        cur.execute('''CREATE TABLE IF NOT EXISTS Events (ID INT PRIMARY KEY,
                                                        Unit TEXT,
                                                        StepCompleted TEXT
                                                        )''')

    def List_Events(self) -> dict:
        
        cur = self.connection.cursor()
        
        sqlite_select_query = """SELECT * FROM Events"""
        
        cur.execute(sqlite_select_query)
        
        df = cur.fetchall()
        df = pd.DataFrame(df, columns=['ID', 'Unit', 'StepCompleted'])
        dict_df = df.to_dict()
        
        return dict_df
    
    def Set_Event (self, Step:str):

        cur = self.connection.cursor()

        self.AutoId.Update_Registred_Ids(registred_ids = self.List_Events())

        ID = self.AutoId.Gen()

        Data = ((ID, self.Unit, Step))

        sqlite_insert_with_param = """INSERT INTO Events (ID, Unit, StepCompleted) VALUES (?, ?, ?);"""
        cur.execute(sqlite_insert_with_param, Data)
        self.connection.commit()

        return

    def Remove_Events(self, ID:int):
        
        cur = self.connection.cursor()
        
        sql_update_query = """DELETE FROM Events WHERE ID = ?"""
        
        cur.execute(sql_update_query, (int(ID),))
        
        self.connection.commit()

class System_Status:

    def __init__(self, path:str):

        pool = SQLiteConnectionPool(3, os.path.join(path, "Data.db"))
        self.connection = pool.get_connection()

        self.AutoId = Interface_Unique_ID_Generator(length=9999, registred_ids=[])
        
        cur = self.connection.cursor()
        cur.execute('''CREATE TABLE IF NOT EXISTS SystemStatus (ID INT PRIMARY KEY,
                                                                Unit TEXT,
                                                                RuningStatus BOOL)''')

    

    def list_units (self) -> dict:
        
        cur = self.connection.cursor()
        
        sqlite_select_query = """SELECT * FROM SystemStatus"""
        
        cur.execute(sqlite_select_query)
        
        df = cur.fetchall()
        df = pd.DataFrame(df, columns=['ID', 'Unit', 'RuningStatus'])
        dict_df = df.to_dict()
        
        return dict_df
    
    def create_unit (self, Unit:str):

        units = pd.DataFrame.from_dict(self.list_units())
        unit = units[units['Unit'] == Unit]   

        if not unit.empty:
            print("Unit alwready created!")
            return
        else:
            pass

        cur = self.connection.cursor()

        self.AutoId.Update_Registred_Ids(registred_ids = self.list_units())

        ID = self.AutoId.Gen()

        Data = ((ID, Unit, False))

        sqlite_insert_with_param = """INSERT INTO SystemStatus (ID, Unit, RuningStatus) VALUES (?, ?, ?);"""
        cur.execute(sqlite_insert_with_param, Data)
        self.connection.commit()

        return
    
    def get_unit_status (self, Unit:str) -> bool:
        
        units = pd.DataFrame.from_dict(self.list_units())
        unit = units[units['Unit'] == Unit]   

        if unit.empty:
            raise f"Unit: {Unit} doesn't exist!"

        unit = unit.reset_index(drop=True)

        status = unit.loc[0, 'RuningStatus']

        return status

    def change_unit_status (self, Unit:str, Status:bool):

        cur = self.connection.cursor()

        Data = (Status, Unit)

        sql_update_query = f"""UPDATE SystemStatus SET RuningStatus = ? WHERE Unit = ?"""
      
        cur.execute(sql_update_query, Data)
        self.connection.commit()

        return



