# SPDX-License-Identifier: MPL-2.0
# Copyright © 2021-2026 Cristian Camargo Filho


import pandas as pd
import time
from . import sql_pool 

class Clients_Retriever:

    def __init__(self, connection):
    
        self.connection = connection
    
        cur = self.connection.cursor()
        cur.execute('''CREATE TABLE IF NOT EXISTS Clients (ID INT PRIMARY KEY, 
                                                           ClientName TEXT, 
                                                           ClientKey TEXT, 
                                                           ClientType TEXT, 
                                                           PermissionGroup TEXT, 
                                                           SuperUser BOOL, 
                                                           LastContact NUMBER, 
                                                           MaxSubChannels NUMBER, 
                                                           OwnedSubChannelsKeys TEXT, 
                                                           SubChannelsInUse NUMBER,
                                                           Handlers TEXT,
                                                            )''')

    def get_clients(self) -> dict:
        
        cur = self.connection.cursor()
        
        sqlite_select_query = """SELECT * FROM Clients"""
        
        cur.execute(sqlite_select_query)
        
        df = cur.fetchall()
        df = pd.DataFrame(df, columns=['ID', 'ClientName', 'ClientKey', 'ClientType', 'PermissionGroup', 'SuperUser', 'LastContact', 'MaxSubChannels', 'OwnedSubChannelsKeys', 'SubChannelsInUse', 'Handlers'])
        dict_df = df.to_dict()
        
        return dict_df

    

