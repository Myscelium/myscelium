
import sqlite3
import random
import os
import pandas as pd
import json
import time
from queue import Queue
from threading import Lock, Thread
from datetime import datetime

# TODO >>> Create a mechanism to create collections and dont need to insert them manually tha t may cause an error os writing sometime

import string

class ParityId: # -> ParityId to sync
    
    def __init__(self, length:int, registered_ids:list):
        self.length = length        # length of BufferId
        self.registered_ids = registered_ids

    def random_string(self) -> str:  
        
        str1 = ''.join((random.choice(string.ascii_letters) for x in range(int(self.length/2))))  
        str1 += ''.join((random.choice(string.digits) for x in range(int(self.length/2))))  
    
        char_list = list(str1) # it converts the string to list  
        random.shuffle(char_list) # function to shuffle the string.  

        return ''.join(char_list)   

    def Update_registered_ids (self, registered_ids:list):
        self.registered_ids = registered_ids
        return

    def Gen (self) -> str: # Gen a id to allocate data in the buffer
        while True:
            BufferId = self.random_string()
            if (self.Validate(BufferId)):
                break
            else:
                pass
        return BufferId

    def Validate (self, BufferId:int) -> bool:  # Validate the id to see if it already is registered if so gen another one.
        DataList = [i for i in self.registered_ids] #! verify if it works
        # DataList = [i[0] for i in self.registered_ids] 
        # DataList = self.dtr.list_schedule.iloc[:, ['Id']].to_list()
        for i in DataList:
            if BufferId == i :
                return False
            else:
                pass
        return True

class Interface_Unique_ID_Generator:
    def __init__(self, length:int, registered_ids:list):
        self.length = length
        self.registered_ids = registered_ids
        self.ids_memory = set()  # maintain a set of currently generated and used IDs

    def Update_registered_ids(self, registered_ids:list):
        self.registered_ids = registered_ids
        return

    def Gen(self) -> int:
        GenBufferId = lambda: random.randint(0, self.length)
        while True:
            BufferId = GenBufferId()
            if (self.Validate(BufferId)):
                self.ids_memory.add(BufferId)  # update memory cache with new ID
                break
        return BufferId

    def Validate(self, BufferId:int) -> bool:
        # First check the memory cache
        if BufferId in self.ids_memory:
            return False
        # Then check the registered IDs
        DataList = [i[0] for i in self.registered_ids]
        for i in DataList:
            if BufferId == i:
                return False
        return True

    def release_id(self, BufferId:int):
        # remove the ID from memory cache when it's no longer needed
        self.ids_memory.remove(BufferId)

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


class Test_Groups_Manager:

    def __init__ (self):

        pool = SQLiteConnectionPool(3, os.path.join("EventKeys", "Data.db"))
        self.connection = pool.get_connection()

        self.AutoId = Interface_Unique_ID_Generator(length=9999, registered_ids=[])

        cur = self.connection.cursor()
        cur.execute('''CREATE TABLE IF NOT EXISTS TestCollections (ID INT PRIMARY KEY, TestCollection TEXT)''')

        pass

    def list_tests_collections (self) -> dict:

        cur = self.connection.cursor()
        
        sqlite_select_query = """SELECT * FROM TestCollections"""
        
        cur.execute(sqlite_select_query)
        
        df = cur.fetchall()
        df = pd.DataFrame(df, columns=['ID', 'TestCollection'])
        dict_df = df.to_dict()
        
        return dict_df

    def registry_new_test_collection (self, collection_name:str):

        cur = self.connection.cursor()

        tests_collections_df = pd.DataFrame.from_dict(self.list_tests_collections())

        filtered_tests_collections_df = tests_collections_df[tests_collections_df["TestCollection"] == collection_name]

        if not filtered_tests_collections_df.empty:
            return 
        else:
            pass
        
        self.AutoId.Update_registered_ids(registered_ids = self.list_tests_collections())

        ID = self.AutoId.Gen()

        ts = time.time()

        sqlite_insert_with_param = """INSERT INTO TestCollections (ID, TestCollection) VALUES (?, ?);"""
        cur.execute(sqlite_insert_with_param, (ID, collection_name))
        self.connection.commit()

        pass

    def delete_test_collection (self, collection_id:int):

        cur = self.connection.cursor()

        tests_collections_df = pd.DataFrame.from_dict(self.list_tests_collections())

        test_collection_id = tests_collections_df.loc[collection_id, "ID"]

        sqlite_delete_with_param = """DELETE FROM TestCollections WHERE ID = ?"""
        cur.execute(sqlite_delete_with_param, (int(test_collection_id), ))
        self.connection.commit()

        print ("Test Collection successfully deleted!\n")


class Events_Keys_Manager:

    def __init__(self): 

        pool = SQLiteConnectionPool(3, os.path.join("EventKeys", "Data.db"))
        self.connection = pool.get_connection()

        self.AutoId = Interface_Unique_ID_Generator(length=9999, registered_ids=[])

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

        parityIdKeyGenerator = ParityId(length=16, registered_ids=keys)

        new_unique_parity_key = parityIdKeyGenerator.Gen()

        self.store_unique_key(new_unique_parity_key, test_collection)

        return new_unique_parity_key
    
    def store_unique_key (self, key:str, test_collection:str):    

        cur = self.connection.cursor()

        keys_dict_df = pd.DataFrame.from_dict(self.list_keys())

        registered_keys = keys_dict_df[keys_dict_df["TestCollection"] == test_collection]["EventKey"].to_list()

        for k in registered_keys:
            if k == key :
                raise f"Key {key} needs to be unique!"
            else:
                pass
            continue
        
        self.AutoId.Update_registered_ids(registered_ids = self.list_keys())
        ID = self.AutoId.Gen()

        ts = time.time()

        sqlite_insert_with_param = """INSERT INTO Keys (ID, EventKey, TestCollection, DateTime) VALUES (?, ?, ?, ?);"""
        cur.execute(sqlite_insert_with_param, (ID, key, test_collection, ts))
        self.connection.commit()

        return
    
    def remove_unique_key (self, key:str, test_collection:str):

        cur = self.connection.cursor()

        keys_dict_df = pd.DataFrame.from_dict(self.list_keys())

        filtered_keys_dict_df = keys_dict_df[keys_dict_df["TestCollection"] == test_collection]

        if filtered_keys_dict_df.empty:
            return
        else:
            pass

        keys = filtered_keys_dict_df["EventKey"].to_list()

        if not (key in keys):
            print (f"Key {key} already not registered!\n")

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

def manage_test_collections ():

    while True:

        print("-[Collections Menu]-!\n")

        print("Please select a option bellow:")
        print("0 - See available test collections")
        print("1 - Gen new test collection")
        print("2 - Remove test collection")
        print("3 - Exit to primary menu")

        imputed = input()

        try: 
            imputed = int(imputed)
        except:
            raise "\nThe input needs to be int\n"

        match imputed:

            case 0:

                test_collections_df = pd.DataFrame.from_dict(Test_Groups_Manager().list_tests_collections())

                print("\nID    TestCollection\n")

                for i in test_collections_df.index:
                    
                    print(f"{i:<3} - {str(test_collections_df.loc[i, 'TestCollection']):<15}")

                print("\n")

                continue

            case 1:

                imputed_test_collection = input("Insert the Test Collection Name: ")
                Test_Groups_Manager().registry_new_test_collection(imputed_test_collection)

                continue

            case 2:

                test_collections_df = pd.DataFrame.from_dict(Test_Groups_Manager().list_tests_collections())

                print("\nID    TestCollection\n")

                for i in test_collections_df.index:
                    
                    print(f"{i:<3} - {str(test_collections_df.loc[i, 'TestCollection']):<15}")

                print("\n")

                imputed_id = input("\nPlease insert the test collection id to remove:\n")

                selected_test_collection = test_collections_df.loc[i, 'TestCollection']

                EKManager = Events_Keys_Manager()

                keys_df = pd.DataFrame.from_dict(EKManager.list_keys())

                print(f"-[{selected_test_collection}]-")

                filtered_keys_df = keys_df[keys_df["TestCollection"] == selected_test_collection]

                for i in filtered_keys_df.index:
                    EKManager.remove_unique_key(filtered_keys_df.loc[i, "EventKey"], test_collection=selected_test_collection)

                Test_Groups_Manager().delete_test_collection(int(imputed_id))

                continue
            
            case 3:
                break

    pass

def main ():

    print("Welcome to unique key generator!\n")

    EKManager = Events_Keys_Manager()

    while True:

        print("Please select a option bellow:")
        print("0 - Manage test collections")
        print("1 - Gen a unique parity key")
        print("2 - See unique parity key")
        print("3 - Remove unique parity key")
        print("4 - Calculate timestamp difference")
        print("5 - Exit")

        imputed = input()

        try: 
            imputed = int(imputed)
        except:
            raise "\nThe input needs to be int\n"

        match imputed:

            case 0:
                manage_test_collections()

            case 1:

                test_collections_df = pd.DataFrame.from_dict(Test_Groups_Manager().list_tests_collections())

                print("Please select one of above:")
                print("\nID    TestCollection\n")

                for i in test_collections_df.index:
                    
                    print(f"{i:<3} - {str(test_collections_df.loc[i, 'TestCollection']):<15}")

                print("\n")

                imputed_test_collection_idx = input("Insert the Test Collection idx: ")

                selected_test_collection = test_collections_df.loc[int(imputed_test_collection_idx), "TestCollection"]

                key = EKManager.gen_key(selected_test_collection)
                print(f"\nYour unique key are: {key}\n")

                continue

            case 2:

                test_collections_df = pd.DataFrame.from_dict(Test_Groups_Manager().list_tests_collections())

                print("Please select one of above:")
                print("\nID    TestCollection\n")

                for i in test_collections_df.index:
                    
                    print(f"{i:<3} - {str(test_collections_df.loc[i, 'TestCollection']):<15}")

                print("\n")

                imputed_test_collection_idx = input("Insert the Test Collection idx: ")

                selected_test_collection = test_collections_df.loc[int(imputed_test_collection_idx), "TestCollection"]

                keys_df = pd.DataFrame.from_dict(EKManager.list_keys())

                print(f"-[{selected_test_collection}]-")

                filtered_keys_df = keys_df[keys_df["TestCollection"] == selected_test_collection]

                print("\nID              CreateAt        TestCollection         Key\n")

                for i in filtered_keys_df.index:
                    
                    print(f"{i:<3} - {format_timestamp(float(filtered_keys_df.loc[i, 'DateTime'])):<15} - {str(filtered_keys_df.loc[i, 'TestCollection'])} - {str(filtered_keys_df.loc[i, 'EventKey'])}")

                print("\n")

                continue

            case 3:

                test_collections_df = pd.DataFrame.from_dict(Test_Groups_Manager().list_tests_collections())

                print("Please select one of above:")
                print("\nID    TestCollection\n")

                for i in test_collections_df.index:
                    
                    print(f"{i:<3} - {str(test_collections_df.loc[i, 'TestCollection']):<15}")

                print("\n")

                imputed_test_collection_idx = input("Insert the Test Collection idx: ")

                selected_test_collection = test_collections_df.loc[int(imputed_test_collection_idx), "TestCollection"]

                print(f"\nTest collection: {selected_test_collection} selected!")
                imputed_key = input("Now please insert a key\n")
                EKManager.remove_unique_key(imputed_key, test_collection=selected_test_collection)

                continue

            case 4:

                first_ts = float(input("\nPlease Insert the start ts: "))
                second_ts = float(input("Please Insert the end ts: "))

                # Calculating the difference in seconds
                seconds_difference = second_ts - first_ts
                print(f"\nThe difference is: {seconds_difference} seconds\n")

            case 5:
                return


        continue

    return 

if __name__ == "__main__":
    main()
