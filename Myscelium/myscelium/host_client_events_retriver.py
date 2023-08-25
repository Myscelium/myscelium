
import pandas as pd
import time
from sql_pool import SQLiteConnectionPool

class Clients_Retriver:

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
                                                           SubChannelsInUse NUMBER)''')

    def get_clients(self) -> dict:
        
        cur = self.connection.cursor()
        
        sqlite_select_query = """SELECT * FROM Clients"""
        
        cur.execute(sqlite_select_query)
        
        df = cur.fetchall()
        df = pd.DataFrame(df, columns=['ID', 'ClientName', 'ClientKey', 'ClientType', 'PermissionGroup', 'SuperUser', 'LastContact', 'MaxSubChannels', 'OwnedSubChannelsKeys', 'SubChannelsInUse'])
        dict_df = df.to_dict()
        
        return dict_df

    def watch_client_contact (self, calback):

        control = []

        while True:

            clients_df = self.get_clients()
            clients_pd_df = pd.DataFrame.from_dict(clients_df)

            if clients_pd_df.empty:
                
                print("[Event Retriver] - No clients to transpose contact, next checking in 10s")
                time.sleep(10)
          
                continue
            
            else:
                pass

            actual_control = clients_pd_df.values.tolist()

            if len(control) == 0:
                control = actual_control
            else:
                pass

            for i, n in enumerate(control):

                actual_to_compare = actual_control[i]

                if n['LastContact'] > actual_to_compare['LastContact']:
                    calback(actual_to_compare['ClientName'], actual_to_compare['ClientKey'], actual_to_compare['LastContact'])
                else:
                    pass

                continue
            
            control = actual_control

        return  

