# SPDX-License-Identifier: MPL-2.0
# Copyright © 2021-2026 Cristian Camargo Filho


import pandas as pd

class Logs_Buffer_Retriever:

    def __init__(self, connection):
    
        self.connection = connection
    
        cur = self.connection.cursor()
        cur.execute(
            '''CREATE TABLE IF NOT EXISTS ClientLogs (
                ID INT PRIMARY KEY,
                NodeName TEXT,
                LogTime FLOAT,
                LogName TEXT,
                LogLevel TEXT,
                LogMsg TEXT 
            )'''
        )

    def List_Logs(self) -> dict:
        
        cur = self.connection.cursor()
        sqlite_select_query = """SELECT * FROM ClientLogs"""
        cur.execute(sqlite_select_query)
        
        df = cur.fetchall()
        df = pd.DataFrame(df, columns=['ID', 'NodeName', 'LogTime', 'LogName', 'LogLevel', 'LogMsg'])
        dict_df = df.to_dict()
        
        return dict_df

    def Remove_Log(self, ID:int):
        
        cur = self.connection.cursor()
        sql_update_query = """DELETE from ClientLogs WHERE ID = ?"""
        cur.execute(sql_update_query, (int(ID),))
        self.connection.commit()

