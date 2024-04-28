from ..common import sql_pool
import os
import pandas as pd

class GetHostClients:
    def __init__(self, db_path: str):
        self.pool = sql_pool.SQLiteConnectionPool(2, os.path.join(db_path, "Data.db"))
        connection = self.pool.get_connection()
        cur = connection.cursor()
        cur.execute(
            """CREATE TABLE IF NOT EXISTS Clients (
                ID INT PRIMARY KEY,
                ClientName TEXT,
                ClientKey TEXT,
                ClientType TEXT,
                PermissionGroup TEXT,
                SuperUser BOOL,
                LastContact FLOAT,
                MaxSubChannels NUMBER,
                OwnedSubChannelsKeys TEXT,
                SubChannelsInUse NUMBER
            )"""
        )

        self.pool.release_connection(connection)

    def list_clients(self) -> dict:

        connection = self.pool.get_connection()
        cur = connection.cursor()
        sqlite_select_query = """SELECT * FROM Clients"""
        cur.execute(sqlite_select_query)
        df = cur.fetchall()

        self.pool.release_connection(connection)

        df = pd.DataFrame(
            df,
            columns=[
                "ID",
                "ClientName",
                "ClientKey",
                "ClientType",
                "PermissionGroup",
                "SuperUser",
                "LastContact",
                "MaxSubChannels",
                "OwnedSubChannelsKeys",
                "SubChannelsInUse",
            ],
        )

        dict_df = df.to_dict()

        return dict_df