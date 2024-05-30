from ..common import sql_pool
import os
import pandas as pd
from . import host_logs_retriever
from ..common.logs_transposition import transpose, check_if_all_logs_was_transposed
import time
from ..common.functions import split_dataframe
from multiprocessing import Process
from . import host_client_events_retriever

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
    

class MysceliumHostInterface:
    def __init__(self, buffer_path: str) -> None:
        """
        Initialize the MysceliumHostInterface.

        Parameters:
        - buffer_path: Path to the buffer for logs retrieval.
        """

        self.client_events_retriever_stats = False
        self.buffer_path = buffer_path
        self.clients_contact_retriever_callback = ""
        self.log_callback = ""
        self.stats = False
        self.process = ""
        self.transposition_threads = 1

        return

    # -> -------------------------------------------------------------------------------------------
    # -> LOGS RETRIEVER

    def _retrieve_logs(self):
        """
        Retrieve logs and process them. If multiple threads are set, it will split the logs
        and process them in parallel.
        """

        pool = sql_pool.SQLiteConnectionPool(
            self.transposition_threads + 2, os.path.join(self.buffer_path, "Logs.db")
        )

        connection = pool.get_connection()

        logs_retriever_access = host_logs_retriever.Logs_Buffer_retriever(connection)

        while True:
            if not self.stats:
                while True:
                    if check_if_all_logs_was_transposed:
                        break
                    else:
                        continue
                break
            else:
                pass

            logs_dict_df = logs_retriever_access.List_Logs()
            logs_df = pd.DataFrame.from_dict(logs_dict_df)

            if logs_df.empty:
                time.sleep(2)
                continue
            else:
                pass

            logs_df = logs_df.sort_values("LogTime")
            logs_df = logs_df.reset_index(drop=True)

            if self.transposition_threads > 1:
                logs_df_chunks = split_dataframe(logs_df, self.transposition_threads)

                threads = []

                for chunk in logs_df_chunks:
                    threads.append(
                        Process(
                            target=transpose,
                            args=(chunk, self.buffer_path, self.log_callback),
                        )
                    )
                    continue

                for t in threads:
                    t.start()
                    continue

                for t in threads:
                    t.join()
                    continue

                pass

            else:
                transpose(logs_df, self.buffer_path, self.log_callback)
                pass

            time.sleep(1)

            continue

        pool.release_connection(connection)

        return

    def set_logs_callback(self, callback: str):
        """
        Set the callback function for logs.

        Parameters:
        - callback: Callback function to be invoked for each log.
        """

        self.log_callback = callback

        pass

    def start_logs_retriever(self):
        """
        Start the logs retriever process in a separate process.
        """

        self.stats = True
        self.process = Process(target=self._retrieve_logs, args=())
        self.process.start()

        return

    def stop_logs_retriever(self):
        """
        Stop the logs retriever process.
        """

        self.stats = False
        self.process.join()

        return

    # -> -------------------------------------------------------------------------------------------
    # -> CLIENT CONTACT EVENT RETRIEVER

    def watch_client_contact(self):
        control = []

        pool = sql_pool.SQLiteConnectionPool(
            2, os.path.join(self.buffer_path, "Data.db")
        )

        while True:
            time.sleep(2)

            if not self.client_events_retriever_stats:
                break
            else:
                pass

            connection = pool.get_connection()

            client_events_retriever = host_client_events_retriever.Clients_Retriever(
                connection
            )

            clients_df = client_events_retriever.get_clients()
            clients_pd_df = pd.DataFrame.from_dict(clients_df)

            if clients_pd_df.empty:
                print(
                    "[Event retriever] - No clients to transpose contact, next checking in 10s"
                )

                pool.release_connection(connection)

                continue

            else:
                pass

            actual_control = clients_pd_df.values.tolist()

            # print(f"Control group: {control}\n New group: {actual_control}")

            if len(control) != len(actual_control):
                control = actual_control
            else:
                pass

            for i, n in enumerate(control):
                actual_to_compare = actual_control[i]

                if (n[6] != "" and actual_to_compare[6] != "") and (
                    n[6] < actual_to_compare[6]
                ):
                    if not isinstance(self.clients_contact_retriever_callback, str):
                        pass
                    else:
                        print(
                            f"Client: {actual_to_compare[1]} of key: {actual_to_compare[2]} made contact but not find any valid callback to transpose it!"
                        )

                        pool.release_connection(connection)

                        continue

                    self.clients_contact_retriever_callback(
                        actual_to_compare[1], actual_to_compare[2], actual_to_compare[6]
                    )
                    print(
                        f"Client: {actual_to_compare[1]} of key: {actual_to_compare[2]} made contact"
                    )

                else:
                    pass

            control = actual_control
            pool.release_connection(connection)

            continue

        return

    def set_client_contact_retriever_callback(self, callback: str):
        """
        Set the callback function for client contacts transposition.

        Parameters:
        - callback: Callback function to be invoked for each client contact.
        """

        self.clients_contact_retriever_callback = callback

        pass

    def start_client_events_retriever(self):
        """
        Start the clients event retriever process.
        """

        print("client_events_retriever started!")

        self.client_events_retriever_stats = True

        self.client_events_retriever_process = Process(
            target=self.watch_client_contact, args=()
        )
        self.client_events_retriever_process.start()

        return

    def stop_client_events_retriever(self):
        """
        Stop the clients event retriever process.
        """

        self.client_events_retriever_stats = False

        self.client_events_retriever_process.kill()
        self.client_events_retriever_process.join()

        return

    def allow_multi_handlers(self, workers_num:int=2):
        """
        Activate multiple handlers for processing logs.

        Parameters:
        - threads_num: Number of threads to be used for processing logs.
        """

        self.transposition_threads = workers_num

        return