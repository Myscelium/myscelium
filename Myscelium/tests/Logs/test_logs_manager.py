import sqlite3
import random
import os
import pandas as pd
import json
import time
from queue import Queue
from threading import Lock, Thread
from datetime import datetime

# TODO >>> Create a client table to set the logs and the client state and the host state
# TODO >>> if the host or client state in the table was set to false it will close the host or the client

class Interface_Unique_ID_Generator:

    def __init__(self, length:int, registered:list):
        self.length = length        # length of BufferId
        self.registered_ids = registered

    def Update_registered (self, registered:list):
        self.registered = registered
        return

    def Gen (self) -> int: # Gen a id for data allocation in buffer 
        GenBufferId = lambda: random.randint(0, self.length)
        while True:
            BufferId = GenBufferId()
            if (self.Validate(BufferId)):
                break
            else:
                pass
        return BufferId

    def Validate (self, BufferId:int) -> bool:  # Validate the id generated and verify if the case already exists, if exists then generate again
        DataList = [i[0] for i in self.registered]
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


def get_events_manager_units ():
    cur = self.connection.cursor()
        
    sqlite_select_query = """SELECT * FROM Events"""

    Data = ((self.Unit, ))
        
    cur.execute(sqlite_select_query, Data)
    
    df = cur.fetchall()
    df = pd.DataFrame(df, columns=['ID', 'Unit', 'StepCompleted', 'EventType', 'EventKey', 'Time'])
    dict_df = df.to_dict()
    
    return dict_df

class Events_Manager:

    def __init__(self, Unit:str, path:str):
        """
        if unit == "*" select all units data
        """

        pool = SQLiteConnectionPool(3, os.path.join(path, "Data.db"))
        self.connection = pool.get_connection()

        self.Unit = Unit

        self.AutoId = Interface_Unique_ID_Generator(length=9999, registered=[])

        cur = self.connection.cursor()
        cur.execute('''CREATE TABLE IF NOT EXISTS Events (ID INT PRIMARY KEY,
                                                        Unit TEXT,
                                                        StepCompleted TEXT,
                                                        EventType TEXT, 
                                                        EventKey TEXT,
                                                        Time NUMBER
                                                        )''')
        
    
    def drop_events_table(self) -> None:
        """Drop the Events table from the database."""
    
        try:
            cur = self.connection.cursor()
            
            sql_drop_table_query = """DROP TABLE IF EXISTS Events"""
            cur.execute(sql_drop_table_query)
            
            self.connection.commit()
            print("Events table has been removed.")
            
        except Exception as e:
            print(f"Error occurred while removing the Events table. Error: {e}")


    def List_Events(self) -> dict:
        
        cur = self.connection.cursor()
        
        if self.Unit == "*":
            sqlite_select_query = """SELECT * FROM Events"""
            cur.execute(sqlite_select_query)
        
        else:
            sqlite_select_query = """SELECT * FROM Events WHERE Unit = ?"""
            Data = ((self.Unit, ))
            cur.execute(sqlite_select_query, Data)
        
        df = cur.fetchall()
        df = pd.DataFrame(df, columns=['ID', 'Unit', 'StepCompleted', 'EventType', 'EventKey', 'Time'])
        dict_df = df.to_dict()
        
        return dict_df
    
    def Set_Event (self, step:str, event_type:str="Default", **kwargs):     

        """
        Set a event

        event_type: str - can be:
            - Default
            - Exception
            - Send
            - Receive
        
        if event_type is Send and Receive it needs a predefined `event_key:str` kwarg,
        the event key is a str and need to contain at least 16 digits that need to be random,
        to generate a valid key pair and have 100% sure that this isn't registered you can use the 
        helper `gen_valid_event_key.py` at Myscelium/tests/Logs/gen_valid_event_key.py

        """
        
        if self.Unit == "*":
            raise ValueError("Can't Set Event To Generalized Unit: '*'")

        if event_type in ["Default", "Exception", "Send", "Receive"]:
            pass
        else:
            raise ValueError("Event type can only be one of those: 'Default, Exception, Send, Receive'")

        events = pd.DataFrame.from_dict(self.List_Events())   

        for i in events.index:
            if events.loc[i, "StepCompleted"] == step:
                return
            else:
                continue

        cur = self.connection.cursor()

        self.AutoId.Update_registered(registered = self.List_Events())

        ID = self.AutoId.Gen()

        ts = time.time()

        event_key = ""

        if event_type == "Send" or event_type == "Receive":

            if not ("event_key" in kwargs):
                raise "You need to to specify a event code to Send an Receive event_types"
            else:   
                pass

            event_key = kwargs["event_key"]

        Data = ((ID, self.Unit, step, event_type, event_key, ts))

        sqlite_insert_with_param = """INSERT INTO Events (ID, Unit, StepCompleted, EventType, EventKey, Time) VALUES (?, ?, ?, ?, ?, ?);"""
        cur.execute(sqlite_insert_with_param, Data)
        self.connection.commit()

        return

 

class System_Status:

    def __init__(self, path:str):

        pool = SQLiteConnectionPool(3, os.path.join(path, "Data.db"))
        self.connection = pool.get_connection()

        self.AutoId = Interface_Unique_ID_Generator(length=9999, registered=[])
        
        cur = self.connection.cursor()
        cur.execute('''CREATE TABLE IF NOT EXISTS SystemStatus (ID INT PRIMARY KEY,
                                                                Unit TEXT,
                                                                RunningStatus BOOL)''')

    def list_units (self) -> dict:
        
        cur = self.connection.cursor()
        
        sqlite_select_query = """SELECT * FROM SystemStatus"""
        
        cur.execute(sqlite_select_query)
        
        df = cur.fetchall()
        df = pd.DataFrame(df, columns=['ID', 'Unit', 'RunningStatus'])
        dict_df = df.to_dict()
        
        return dict_df
    
    def create_unit (self, Unit:str):

        units = pd.DataFrame.from_dict(self.list_units())
        unit = units[units['Unit'] == Unit]   

        if not unit.empty:
            print("Unit already created!")
            return
        else:
            pass

        cur = self.connection.cursor()

        self.AutoId.Update_registered(registered = self.list_units())

        ID = self.AutoId.Gen()

        Data = ((ID, Unit, False))

        sqlite_insert_with_param = """INSERT INTO SystemStatus (ID, Unit, RunningStatus) VALUES (?, ?, ?);"""
        cur.execute(sqlite_insert_with_param, Data)
        self.connection.commit()

        return
    
    def get_unit_status (self, Unit:str) -> bool:
        
        units = pd.DataFrame.from_dict(self.list_units())
        unit = units[units['Unit'] == Unit]   

        if unit.empty:
            raise f"Unit: {Unit} doesn't exist!"

        unit = unit.reset_index(drop=True)

        status = unit.loc[0, 'RunningStatus']

        return status

    def change_unit_status (self, Unit:str, Status:bool):

        cur = self.connection.cursor()

        Data = (Status, Unit)

        sql_update_query = f"""UPDATE SystemStatus SET RunningStatus = ? WHERE Unit = ?"""
      
        cur.execute(sql_update_query, Data)
        self.connection.commit()

        return