# SPDX-License-Identifier: MPL-2.0
# Copyright © 2021-2026 Cristian Camargo Filho

from ..common import sql_pool
from ..client import client_logs_retriever
from ..common.functions import split_dataframe
from ..common.logs_transposition import transpose, check_if_all_logs_was_transposed
from multiprocessing import Process
import pandas as pd
import time
import os

class MysceliumClientInterface:
    def __init__(self, buffer_path: str) -> None:
        """
        Initialize the MysceliumHostInterface.

        Parameters:
        - buffer_path: Path to the buffer for logs retrieval.
        """

        self.buffer_path = buffer_path
        self.log_callback = ""
        self.stats = False
        self.process = ""
        self.transposition_threads = 1

        return

    def retrieve_logs(self):
        """
        Retrieve logs and process them. If multiple threads are set, it will split the logs
        and process them in parallel.
        """

        pool = sql_pool.SQLiteConnectionPool(
            self.transposition_threads + 2, os.path.join(self.buffer_path, "Logs.db")
        )

        connection = pool.get_connection()

        logs_retriever_access = client_logs_retriever.Logs_Buffer_retriever(connection)

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

    def allow_multi_handlers(self, workers_num=2):
        """
        Activate multiple handlers for processing logs.

        Parameters:
        - threads_num: Number of threads to be used for processing logs.
        """

        self.transposition_threads = workers_num
        return

    def set_logs_callback(self, callback: str):
        """
        Set the callback function for logs.

        Parameters:
        - callback: Callback function to be invoked for each log.
        """

        self.log_callback = callback
        pass

    def stop_logs_retriever(self):
        """
        Stop the logs retriever process.
        """

        self.stats = False
        self.process.join()
        return

    def start_logs_retriever(self):
        """
        Start the logs retriever process in a separate process.
        """

        self.stats = True
        self.process = Process(target=self.retrieve_logs, args=())
        self.process.start()
        return