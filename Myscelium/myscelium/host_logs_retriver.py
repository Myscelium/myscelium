import sqlite3
import random
import os
import pandas as pd
import json
from datetime import datetime
from queue import Queue
from threading import Lock, Thread

from . import sql_pool 

class Logs_Buffer_Retriver:

    def __init__(self, connection):
    
        self.connection = connection
    
        cur = self.connection.cursor()
        cur.execute('''CREATE TABLE IF NOT EXISTS HostLogs (ID INT PRIMARY KEY,
                                                        NodeName TEXT,
                                                        LogTime FLOAT,
                                                        LogName TEXT,
                                                        LogLevel TEXT,
                                                        LogMsg TEXT 
                                                        )''')

    def List_Logs(self) -> dict:
        
        cur = self.connection.cursor()
        
        sqlite_select_query = """SELECT * FROM HostLogs"""
        
        cur.execute(sqlite_select_query)
        
        df = cur.fetchall()
        df = pd.DataFrame(df, columns=['ID', 'NodeName', 'LogTime', 'LogName', 'LogLevel', 'LogMsg'])
        dict_df = df.to_dict()
        
        return dict_df

    def Remove_Log(self, ID:int):
        
        cur = self.connection.cursor()
        
        sql_update_query = """DELETE from HostLogs WHERE ID = ?"""
        
        cur.execute(sql_update_query, (int(ID),))
        
        self.connection.commit()

