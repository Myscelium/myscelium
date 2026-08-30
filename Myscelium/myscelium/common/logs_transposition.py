# SPDX-License-Identifier: MPL-2.0
# Copyright © 2021-2026 Cristian Camargo Filho

import os
import pandas as pd
from . import sql_pool
from ..server import host_logs_retriever

def transpose(logs_df, buffer_path, log_callback):
    pool = sql_pool.SQLiteConnectionPool(2, os.path.join(buffer_path, "Logs.db"))
    connection = pool.get_connection()
    logs_retriever_access = host_logs_retriever.Logs_Buffer_retriever(connection)

    for i in logs_df.index:
        try:
            log_id = logs_df.loc[i, "ID"]
            log_time = logs_df.loc[i, "LogTime"]
            log_from_node = logs_df.loc[i, "NodeName"]
            log_level = logs_df.loc[i, "LogLevel"]
            log_msg = logs_df.loc[i, "LogMsg"]

            log_callback(
                {
                    "log_time": log_time,
                    "log_level": log_level,
                    "log_from_node": log_from_node,
                    "log_msg": log_msg,
                }
            )
        except:
            pass

        logs_retriever_access.Remove_Log(log_id)
        continue

    pool.release_connection(connection)
    return

def check_if_all_logs_was_transposed(pool):
    connection = pool.get_connection()

    logs_retriever_access = host_logs_retriever.Logs_Buffer_retriever(connection)
    logs_dict_df = logs_retriever_access.List_Logs()

    pool.release_connection(connection)
    logs_df = pd.DataFrame.from_dict(logs_dict_df)

    return logs_df.empty
