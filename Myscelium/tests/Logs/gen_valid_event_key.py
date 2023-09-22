
import sqlite3
import random
import os
import pandas as pd
import json
import time
from queue import Queue
from threading import Lock, Thread
from datetime import datetime

# TODO >>> Create a mecanism to create collections and dont need to insert them manually tha t may cause an error os writing sometime

import string

class ParityId: # -> ParityId to sinc
    
    def __init__(self, length:int, registred_ids:list):
        self.length = length        # length of BufferId
        self.registred_ids = registred_ids

    def random_string(self) -> str:  
        
        str1 = ''.join((random.choice(string.ascii_letters) for x in range(int(self.length/2))))  
        str1 += ''.join((random.choice(string.digits) for x in range(int(self.length/2))))  
    
        char_list = list(str1) # it converts the string to list  
        random.shuffle(char_list) # function to shuffle the string.  

        return ''.join(char_list)   

    def Update_Registred_Ids (self, registred_ids:list):
        self.registred_ids = registred_ids
        return

    def Gen (self) -> str: # Gera um id para alocação dos dados no buffer de dados
        while True:
            BufferId = self.random_string()
            if (self.Validate(BufferId)):
                break
            else:
                pass
        return BufferId

    def Validate (self, BufferId:int) -> bool:  # Valida o id gerado e verifica se já existe, caso exista um id novoé gerado até que seja valido
        DataList = [i for i in self.registred_ids] #! verify if it works
        # DataList = [i[0] for i in self.registred_ids] 
        # DataList = self.dtr.list_schedule.iloc[:, ['Id']].to_list()
        for i in DataList:
            if BufferId == i :
                return False
            else:
                pass
        return True

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


THIS_DIR = os.path.dirname(__file__)

class Events_Keys_Mannanger:

    def __init__(self): 

        pool = SQLiteConnectionPool(3, os.path.join("EventKeys", "Data.db"))
        self.connection = pool.get_connection()

        self.AutoId = Interface_Unique_ID_Generator(length=9999, registred_ids=[])

        cur = self.connection.cursor()
        cur.execute('''CREATE TABLE IF NOT EXISTS Keys (ID INT PRIMARY KEY, EventKey TEXT, TestCollection TEXT, DateTime NUMBER)''')
        
        
    def drop_events_table(self) -> None:
        """Drop the Keys table from the database."""
    
        try:
            cur = self.connection.cursor()
            
            sql_drop_table_query = """DROP TABLE IF EXISTS Keys"""
            cur.execute(sql_drop_table_query)
            
            self.connection.commit()
            print("Keys table has been removed.")
            
        except Exception as e:
            print(f"Error occurred while removing the Keys table. Error: {e}")


    def list_keys (self) -> dict:
        
        cur = self.connection.cursor()
        
        sqlite_select_query = """SELECT * FROM Keys"""
        
        cur.execute(sqlite_select_query)
        
        df = cur.fetchall()
        df = pd.DataFrame(df, columns=['ID', 'EventKey', 'TestCollection', 'DateTime'])
        dict_df = df.to_dict()
        
        return dict_df
    
    def gen_key (self, test_collection:str) -> str:

        cur = self.connection.cursor()

        keys_dict_df = pd.DataFrame.from_dict(self.list_keys())

        keys = keys_dict_df[keys_dict_df["TestCollection"] == test_collection]["EventKey"].to_list()

        parityIdKeyGenerator = ParityId(length=16, registred_ids=keys)

        new_unique_parity_key = parityIdKeyGenerator.Gen()

        self.store_unique_key(new_unique_parity_key, test_collection)

        return new_unique_parity_key
    
    def store_unique_key (self, key:str, test_collection:str):    

        cur = self.connection.cursor()

        keys_dict_df = pd.DataFrame.from_dict(self.list_keys())

        registred_keys = keys_dict_df[keys_dict_df["TestCollection"] == test_collection]["EventKey"].to_list()

        for k in registred_keys:
            if k == key :
                raise f"Key {key} needs to be unique!"
            else:
                pass
            continue
        
        self.AutoId.Update_Registred_Ids(registred_ids = self.list_keys())

        ID = self.AutoId.Gen()

        ts = time.time()

        sqlite_insert_with_param = """INSERT INTO Keys (ID, EventKey, TestCollection, DateTime) VALUES (?, ?, ?, ?);"""
        cur.execute(sqlite_insert_with_param, (ID, key, test_collection, ts))
        self.connection.commit()

        return
    
    def remove_unique_key (self, key:str):

        cur = self.connection.cursor()

        keys_dict_df = pd.DataFrame.from_dict(self.list_keys())

        keys = keys_dict_df["EventKey"].to_list()

        if not (key in keys):
            print (f"Key {key} alwready not registred!\n")

            return
        
        else:
            pass
        
        sqlite_insert_with_param = """DELETE FROM Keys WHERE EventKey = ?"""
        cur.execute(sqlite_insert_with_param, (key, ))
        self.connection.commit()

        print ("Key successfully deleted!\n")

def format_timestamp(timestamp):
    dt = datetime.fromtimestamp(timestamp)
    return dt.strftime('%d//%m/%Y - %H::%M:%S')

def main ():

    print("Welcome to unique key generator!\n")

    EKMannanger = Events_Keys_Mannanger()

    while True:

        print("Please select a option bellow:")
        print("1 - Gen a unique parity key")
        print("2 - See unique parity key")
        print("3 - Remove unique parity key")
        print("4 - Calculate timestamp diference")
        print("5 - Exit")

        inputed = input()

        try: 
            inputed = int(inputed)
        except:
            raise "\nThe imput needs to be int\n"

        match inputed:

            case 1:
                inputed_test_collection = input("Insert the Test Collection Name: ")
                key = EKMannanger.gen_key(inputed_test_collection)
                print(f"\nYour unique key are: {key}\n")

                continue

            case 2:

                EKMannanger.list_keys()

                keys_dict_df = pd.DataFrame.from_dict(EKMannanger.list_keys())

                print("\nCreateAt              TestCollection               Key\n")

                for i in keys_dict_df.index:
                    
                    print(f"{i:<3} - {format_timestamp(float(keys_dict_df.loc[i, 'DateTime'])):<15} - {str(keys_dict_df.loc[i, 'TestCollection'])} - {str(keys_dict_df.loc[i, 'EventKey'])}")

                print("\n")

                continue

            case 3:

                inputed_key = input("\nPlease insert a key\n")
                EKMannanger.remove_unique_key(inputed_key)

                continue

            case 4:

                first_ts = float(input("\nPlease Insert the start ts: "))
                second_ts = float(input("Please Insert the end ts: "))

                # Calculating the difference in seconds
                seconds_difference = second_ts - first_ts
                print(f"\nThe diference is: {seconds_difference} seconds\n")

            case 5:
                return


        continue

    return 

if __name__ == "__main__":
    main()
