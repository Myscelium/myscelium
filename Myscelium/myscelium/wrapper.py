from . import (
    myscelium_engine as mys,
)  # Maybe change the rust myscelium lib to MysceliumEngine
from . import host_logs_retriever
from . import host_client_events_retriever
from . import client_logs_retriever

from multiprocessing import Process
import pandas as pd
import time
import os

from . import sql_pool

import inspect


# > Type Cast


def cast_command_instruction(
    command_mode: str,
    command_type: str,
    command_target: str,
    command_status: str,
    command_origin: str,
    command_actf: str,
    command_kwargs: dict,
    command_message: str,
) -> dict:
    """
    Constructs a command instruction dictionary from the given parameters.

    Validates the provided arguments against specific criteria for each command aspect
    (mode, type, target, status, origin, activation function). Raises an exception if
    any of the provided arguments do not meet the expected values or format.

    Parameters:
    - command_mode (str): Mode of the command, must be one of ['Function', 'Response'].
    - command_type (str): Type of the command, must be one of ['SpecialFunction', 'DirectFunction',
                      'InternalManagement', 'ExternalFunction'].
    - command_target (str): Target of the command, should follow the format 'Origin',
                            'ClientKey(String)', or 'Host'.
    - command_status (str): Status of the command, must be one of ['Success', 'Failure'].
    - command_origin (str): Origin of the command, must be 'Host' or 'ClientKey(String)'.
    - command_actf (str): Activation function for the command. Cannot be empty.
    - command_kwargs (dict): Additional keyword arguments for the command.
    - command_message (str): Message associated with the command.

    Returns:
    dict: A dictionary representing the constructed command instruction.

    Raises:
    Exception: If any parameter does not conform to its expected format or allowable values.

    Example:
    command_dict = cast_command_instruction("Function", "DirectFunction", "Host", "Success",
                                            "ClientKey(123)", "activate", {}, "Execute action")
    """

    if command_mode not in ["Function", "Response"]:
        raise ValueError(
            "Command mode needs to be one of those: ['Function', 'Response']"
        )

    if command_type not in [
        "SpecialFunction",
        "DirectFunction",
        "InternalManagement",
        "ExternalFunction",
    ]:
        raise ValueError(
            "Command type needs to be one of those: ['SpecialFunction', 'DirectFunction', 'InternalManagement', 'ExternalFunction',]"
        )

    if command_target in ["Origin", "Host"]:
        pass

    # -> Validate the redirect cases:
    elif command_target.startswith("ClientKey(") and command_target.endswith(")"):
        # Extracting the part inside 'ClientKey()'
        content = command_target[len("ClientKey(") : -1].strip()

        # Validate the content inside the parentheses
        if content == "":
            raise ValueError("Command target ClientKey needs a valid ClientKey!")

    else:
        raise ValueError(
            "Command target must be either 'Origin', 'Host', or 'ClientKey(some_value)'"
        )

    if command_status not in ["Success", "Failure"]:
        raise ValueError(
            "Command status can only be one of those: ['Success', 'Failure']"
        )

    if command_origin == "Host":
        pass

    # -> Validate the client cases:
    elif command_origin.startswith("ClientKey(") and command_origin.endswith(")"):
        # Extracting the part inside 'ClientKey()'
        content = command_origin[len("ClientKey(") : -1].strip()

        # Validate the content inside the parentheses
        if content == "":
            raise ValueError("Command target ClientKey needs a valid ClientKey!")

    else:
        raise ValueError(
            "Command origin must be either 'Host' or 'ClientKey(some_value)'"
        )

    if command_actf == "" or command_actf == None:
        raise ValueError("Response activation function can't be empty")

    command_instruction = {
        "mode": command_mode,
        "type": command_type,
        "target": command_target,
        "status": command_status,
        "origin": command_origin,
        "actf": command_actf,
        "kwargs": command_kwargs,
        "message": command_message,
    }

    return command_instruction


# >-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
# > Utilities

import pandas as pd
from . import sql_pool


class GetHostClients:
    def __init__(self, db_path: str):
        self.pool = sql_pool.SQLiteConnectionPool(2, os.path.join(db_path, "Data.db"))
        connection = self.pool.get_connection()

        cur = connection.cursor()
        cur.execute(
            """CREATE TABLE IF NOT EXISTS Clients (ID INT PRIMARY KEY,
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


import inspect


class CallbackCollector:

    """
    Extract from a list of classes (Handlers, Receivers and Retransmiters) the methods and callbacks in it
    And automatically creates the callback list, simplifiing even more the process of create new callbacks and
    Methods for you host or to you client.

    Usage:

    ```
    callbacks_list = CallbackCollector([Handlers, Receivers, Retransmiters]).get_callbacks()
    ```
    """

    def __init__(self, callback_containers):
        self.callbacks = []
        for container in callback_containers:
            self._get_methods(container)

    def _get_methods(self, callback_class):
        # Get all attributes of Class
        for name, obj in inspect.getmembers(callback_class):
            # Check if it is a function/method
            if (
                inspect.isfunction(obj)
                or inspect.ismethod(obj)
                or isinstance(obj, staticmethod)
            ):
                # If it's a static method, get the underlying function
                if isinstance(obj, staticmethod):
                    obj = obj.__get__(None, None)

                # Check if obj is not None before proceeding
                if obj is not None:
                    callback_pattern_result = host_patterns.callback_pattern(
                        callback=obj
                    )
                    # Check if callback_pattern_result is not None before appending
                    if callback_pattern_result is not None:
                        self.callbacks.append(callback_pattern_result)

    def get_callbacks(self):
        """
        Extract from a list of classes (Handlers, Receivers and Retransmiters) the methods and callbacks in it
        And automatically creates the callback list, simplifiing even more the process of create new callbacks and
        Methods for you host or to you client.

        Usage:

        ```
        callbacks_list = CallbackCollector([Handlers, Receivers, Retransmiters]).get_callbacks()
        ```
        """
        return self.callbacks


# >-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
# > HOST


def split_dataframe(df, num_chunks):
    """
    Split a DataFrame into num_chunks parts.

    If the DataFrame cannot be split exactly into num_chunks, the remainder will be
    distributed among the chunks.

    Parameters:
    - df: DataFrame to be split
    - num_chunks: Number of chunks

    Returns:
    - List of DataFrames
    """

    n = len(df)
    chunk_size = n // num_chunks
    remainder = n % num_chunks

    chunks = []
    start = 0

    for i in range(num_chunks):
        end = start + chunk_size

        if remainder:  # Distribute the remainder across the initial chunks
            end += 1
            remainder -= 1

        chunks.append(df.iloc[start:end])
        start = end

    return chunks


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

    def retrive_logs(self):
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

    def allow_multi_handlers(self, workers_num=2):
        """
        Activate multiple handlers for processing logs.

        Parameters:
        - threads_num: Number of threads to be used for processing logs.
        """

        self.transposition_threads = workers_num

        return

    def set_client_contact_retriever_callback(self, callback: str):
        """
        Set the callback function for client contacts transposition.

        Parameters:
        - callback: Callback function to be invoked for each client contact.
        """

        self.clients_contact_retriever_callback = callback

        pass

    def set_logs_callback(self, callback: str):
        """
        Set the callback function for logs.

        Parameters:
        - callback: Callback function to be invoked for each log.
        """

        self.log_callback = callback

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


class MysceliumHost:
    _instance = None  # Singleton instance

    def __init__(
        self,
        callbacks: list,
        host_id: int,
        allowed_clients: list,
        buffer_path: str,
        n_workers=2,
        n_max_conns: int = 5,
        log_level: str = "DEBUG",
    ) -> None:
        """
        Initialize the MysceliumHost.

        Parameters:
        - callbacks: List of callback functions.
        - host_id: Unique identifier for the host.
        - allowed_clients: List of clients allowed to connect.
        - buffer_path: Path to the buffer.
        - n_workers: Number of workers.
        - n_max_conns: Maximum number of connections.
        - log_level: Logging level.

        Obs:
        - If you don't se logging level it will be deactivated by default!

        """

        if not hasattr(self, "initialized"):
            self.logging_level = log_level

            self.allowed_clients = allowed_clients

            self.host_id = host_id

            self.buffer_path = buffer_path

            special_functions = [
                {
                    "function": get_registered_commands,
                    "response_type": "same_as_origin",
                    "args": "None",
                },
            ]

            if callbacks is None:
                callbacks = []

            callbacks = callbacks + special_functions

            if log_level not in ["DEBUG", "INFO", "WARN", "EXCEPTION", ""]:
                raise ValueError(
                    f"Client log must be some of this: ('DEBUG', 'INFO', 'WARN', 'EXCEPTION') log level cant be: {log_level}"
                )
            else:
                pass

            mys.initialize_host_buffer_tables(buffer_path)

            mys.set_socket_host_log_level(log_level)

            mys.registry_socket_host_callbacks(callbacks)
            mys.set_socket_host_allowed_clients(self.allowed_clients)
            mys.set_socket_host_transposer_num_of_workers(n_workers)
            mys.set_socket_host_max_connections(n_max_conns)

            self.host_thread = None

            pass

    def __new__(cls, *args, **kwargs):
        if not cls._instance:
            cls._instance = super(MysceliumHost, cls).__new__(cls)
            # This will call your __init__, so you don't have to duplicate code
        return cls._instance

    def registry_new_allowed_clients(
        self,
        allowed_clients: list,
    ):
        """
        Registry New Clients Allowed Into Clients Table.

        Parameters:
        - allowed_clients: List of clients allowed to connect. This is obtained by put HostPatterns.client_pattern into a list

        """

        mys.registry_new_allowed_clients(allowed_clients)

    @classmethod
    def get_instance(cls):
        if not cls._instance:
            raise ValueError("MysceliumHost instance has not been initialized")
        return cls._instance

    def clone(self):
        return self

    def set_logs_callback_handler(
        self,
        logs_handler_callback: object,
        active_multi_handlers: bool = False,
        workers_num: int = 2,
    ) -> None:
        """
        Set the logs callback handler.

        Parameters:
        - logs_handler_callback: Callback function to handle logs.
        - active_multi_handlers: Flag to activate multiple handlers.
        - workers_num: Number of workers for handling logs.
        """

        self.host_interface = MysceliumHostInterface(self.buffer_path)

        if self.logging_level == "":
            raise "To use logging you need to set a logging level, the current logging status is deactivated!"
        else:
            pass

        if active_multi_handlers:
            self.host_interface.active_multi_handlers(workers_num)
        else:
            pass

        self.host_interface.set_logs_callback(logs_handler_callback)

        return

    # def set_client_heartbeat_handler (self, callback): #! THIS WILL NOT WORK UNTIL PYTHON POOL IS FINISHED
    #     mys.registry_socket_host_client_heartbeat_contact_callback(callback)

    def get_registered_commands(self) -> dict:
        """
        Retrieve the registered commands.

        Returns:
        - Dictionary of registered commands.
        """

        print("Activated the get registered commands")

        return mys.get_socket_host_available_commands()

    def initialize_host(self, ip: str, port: int):
        """
        Initialize the host with the given IP and port.FResponse pattern

        Parameters:
        - ip: IP address for the host.
        - port: Port number for the host.
        """
        if hasattr(self, "host_interface"):
            if self.logging_level != "":
                self.host_interface.start_logs_retriever()
            else:
                pass
        else:
            pass

        mys.initialize_socket_host(ip, port, self.host_id)

        return

    def stop_host(self, signal, frame):
        """
        Stop the host. This function is intended to be called when a termination signal is received.

        Parameters:
        - signal: Signal received.
        - frame: Current stack frame.
        """

        # This function will be called when a SIGINT signal is received

        mys.stop_socket_host()

        if hasattr(self, "host_interface"):
            if self.logging_level != "":
                self.host_interface.stop_logs_retriever()
            else:
                pass
        else:
            pass

        return

    def send(self):
        """
        Send data. (This method is currently a placeholder and needs to be implemented.)
        """

        pass


class HostPatterns:
    def __init__(self) -> None:
        """
        Initialize the HostPatterns class.
        """

        pass

    def client_pattern(
        self,
        client_name: str,
        client_key: str,
        client_type: str,
        client_permission_group: str,
        client_is_super_user: bool,
        max_sub_channels: int,
        owned_sub_channels_keys: list = [],
    ) -> dict:
        """
        Create a client pattern.

        Parameters:
        - client_name: Name of the client (user).
        - client_key: Unique Key of the client.
        - client_type: Client purpose.
        - client_permission_group: Group that client inherit permission.
        - client_is_super_user: If client has root privileges on myscelium.
        - client_max_sub_channels: Max sub-channels of stream that client are allowed to create and manage.
        - client_owned_sub_channels_keys: Optional parameter to pre initialize host with client sub-channels keys allowed.

        Returns:
        - Dictionary representing the client pattern.
        """

        return {
            "client_name": client_name,
            "client_key": client_key,
            "client_type": client_type,
            "permission_group": client_permission_group,
            "is_super_user": client_is_super_user,
            "max_sub_channels": max_sub_channels,
            "owned_sub_channels_keys": owned_sub_channels_keys,
        }

    def response_pattern(
        self,
        activation_function: str,
        target_key: str = None,
        kwargs: dict = {},
        message="",
    ) -> dict:
        """
        Creates a response pattern for sending back to a client or for retransmission.

        This function handles two main cases:
        1. Simple send to origin: The response is sent back to the originating client.
        2. Retransmit to another client: The response is retransmitted to a different client specified by `target_key`.

        Parameters:
        - `activation_function` (str): The activation function to be triggered upon response.
        - `target_key` (str, optional): The key of the target client for retransmission. ExternalFunction is None.
        - `kwargs` (dict, optional): Additional keyword arguments for the command. ExternalFunction is an empty dict.
        - `message` (str, optional): A message to be sent to the client. ExternalFunction is an empty string.

        Returns:
        dict: A dictionary representing the command instructions based on the specified pattern.

        Note:
        - In the case of 'Simple send to origin', the response is scheduled to be sent back to the client
        that originated the command.
        - In the case of 'Retransmit to another client', the response is redirected to a different client
        specified by `target_key`. The function then triggers the specified `activation_function` on the
        target client. If the target client does not exist, an error is returned.

        Example:
        ```Python
        command = response_pattern("some_function", target_key="client456", kwargs={"arg1": "value1"}, message="Example message")
        ```
        """

        # > The idea of this pattern is to create a response to send back to a client or to retransmit

        # -> Case 1 (Simple send to origin)

        # >
        # > (Client 1)       [Host]
        # >    |                |
        # >    |--------------> |
        # >    |               (|) (schedule to send response back)
        # >    |<-------------- |
        # >    |                |
        # > (Client 1)     (Client 2)
        # >
        # > (|) This is this pattern
        # >

        # -> ---------------------------------------------------------------------------------------------------------------
        # ->
        # -> Case 2 (retransmit to)

        # TODO >>> Change this when impl the new redirect mechanism:

        # >
        # > (Client 1)     (Client 2)   [Host]
        # >    |                |         |
        # >    |--------------- | ------> |
        # >    |                |        (|) (retransmit the command from client 1 to client 2 via retransmiters)
        # >    |                | <------ |
        # >    |               [|]        |
        # >    |                | ------> |
        # >    |                |        (|) (retransmit the response of client 2 to client 1)
        # >    |<-------------- | ------- |
        # >    |                |         |
        # > (Client 1)     (Client 2)   [Host]
        # >
        # > (|) This is this pattern
        # > [|] This is a client process
        # >

        # * When retransmit is used, the response will use the redirect_to var, that is a client_id of the target
        # * That you want to send the command, now the response_activation_function in this case is the function that need to be
        # * Triggered in the target, the engine will get the response and redirect to the other client by this id, if client exists.
        # * Else this will return a error saying that client doesn't exists

        command_instructions = {}

        if target_key == None:
            command_instructions = cast_command_instruction(
                "Response",
                "ExternalFunction",  # TODO >>> Change this case to PythonFunction or somehting like ExternFunction
                "Origin",
                "Success",
                "Host",
                activation_function,
                kwargs,
                message,
            )
        else:  # Redirect case
            command_instructions = cast_command_instruction(
                "Response",
                "ExternalFunction",  # TODO >>> Change this case to PythonFunction or somehting like ExternFunction
                f"ClientKey({target_key})",
                "Success",
                "Host",
                activation_function,
                kwargs,
                message,
            )

        return command_instructions

    def error_response_pattern(
        self, error_message: str, expected_remote_error_handler: str = ""
    ):
        # TODO >>> Implement a Error Handler Callback Caller in client, to allow personalize how errors will be treated or change the way that client handles responses

        # > A possible impl is something like a data arg and in this arg will have sub kwargs in the dict that will formulate the response\
        # > So the client response will be something like:

        # "command" {
        #     "command_type":"response",
        #     "status": "success"
        #     "response_activation_function":"",
        #     "message":"",
        #     "kwargs":{"arg1": [], "arg2": "", "arg3": {}}
        #     "response_mode":"",
        # }

        # so basically what we will do in this case is to send all the command or remove like the response_activation_function
        # and keep the other things to alow the client Handler to extract the status, message, kwargs and response from the function
        # This aay host dont need to keep track of the client args in the handler, cause if this give a exception will be the client mistake
        # also if we need to check the status to take some action in case of error it also will be possible, and then if receive a act_fn that doesn't
        # we simple can give it a exception.

        # In a more extreme case we can only require one handler named router in the client side and then if this doesn't exist dont even start client
        # This router will be responsible to receive a entire command, so he will decide how to process it and what activation function to call
        # Then this will be able to send a response for host redirect if something is wrong redirecting the error for the client tha cause it and keep going

        # -> This pattern is used to manipulate host configs remotely
        # >
        # > (Client 1)       [Host]
        # >    |                |
        # >    |--------------> |  (receive command)
        # >    |               (|) (do something that results in a error and return this pattern error)
        # >    |<-------------- |  (return exception)
        # >    |                |
        # > (Client 1)        [host]
        # >
        # > (|) This is this pattern
        # >

        if not isinstance(error_message, str):
            print("Error message needs to be a string!")

        if not isinstance(expected_remote_error_handler, str):
            print("Expected remote error handler needs to be a string!")

        kwargs = {}
        command_instructions = {}

        if expected_remote_error_handler == "":
            command_instructions = cast_command_instruction(
                "Response",
                "DirectFunction",
                "Origin",
                "Failure",
                "Host",
                "error_handler",  # ExternalFunction error handler
                kwargs,
                error_message,
            )

        else:
            command_instructions = cast_command_instruction(
                "Response",
                "ExternalFunction",
                "Origin",
                "Failure",
                "Host",
                expected_remote_error_handler,
                kwargs,
                error_message,
            )

        return command_instructions

    # TODO >>> Add a version of this to client since now it is possible to activate this types of fns remotelly
    # TODO >>> See if is worth to turn the update host configs functions DirectFunctions and not Internal Management functions
    def update_host_configs(
        self, activation_function: str, **kwargs
    ):  # TODO >>> Need rust backend implementation!
        """
        Create a response pattern.


        Parameters:

            - add_client, needed kwrags:

                ```python
                update_host_configs (self,
                                     activation_function="add_client",
                                     new_client=[client_patter])

                # - new_client:list[client_pattern] -> This is a list that contains the new client to add!
                ```

            - update_client, needed kwargs:

                ```python
                update_host_configs (self,
                                     activation_function="update_client",
                                     actual_client_key="xMsndkdlenfjedLj",
                                     updated_client=[client_patter])

                # - actual_client_key:str
                # - updated_client:list[client_pattern] -> This is a list that contains the new client updated!
                ```

            - remove_client, need kwargs:

                ```
                update_host_configs (self,
                                     activation_function="remove_client",
                                     actual_client_key="xMsndkdlenfjedLj")

                # - client_key:str -> The client key of the client that you want to remove.
                ```

        """

        # -> This pattern is used to manipulate host configs remotely
        # >
        # > (Client 1)       [Host]
        # >    |                |
        # >    |--------------> |  (receive command)
        # >    |               (|) (update some config)
        # >    |<-------------- |  (return confirmation or exception)
        # >    |                |
        # > (Client 1)        [host]
        # >
        # > (|) This is this pattern
        # >
        # > The usage of this pattern is siple, you create a endpoint, then in the return
        # > you create a response with this pattern and send back to the engine, remember, every response of
        # > the endpoints will be sended to the engine again, if you don't want to send nothing just return None

        if activation_function == "add_client":
            if "new_client" in kwargs:
                pass
            else:
                return self.error_response_pattern(
                    "new client isn't in kwargs, so can't add client!",
                    "add_client_handler",
                )

            new_client = kwargs["new_client"]

            if isinstance(new_client, dict):
                pass
            else:
                return self.error_response_pattern(
                    "New client needs to be a dict generated by client_pattern",
                    "add_client_handler",
                )

            kwargs = {"new_client": new_client}

            command_instructions = cast_command_instruction(
                "Response",
                "InternalManagement",
                "Host",
                "Success",
                "Host",
                "add_client",
                kwargs,
                "",
            )

            return command_instructions

        elif activation_function == "update_client":
            if "actual_client_key" in kwargs:
                pass
            else:
                return self.error_response_pattern(
                    "actual_client_key isn't in kwargs, so can't update client!",
                    "update_client_handler",
                )

            actual_client_key = kwargs["actual_client_key"]

            if isinstance(actual_client_key, str):
                pass
            else:
                return self.error_response_pattern(
                    "client key needs to be a string!", "update_client_handler"
                )

            if "updated_client" in kwargs:
                pass
            else:
                return self.error_response_pattern(
                    "new client isn't in kwargs, so can't edit client!",
                    "update_client_handler",
                )

            updated_client = kwargs["updated_client"]

            if isinstance(updated_client, dict):
                pass
            else:
                return self.error_response_pattern(
                    "New client needs to be a dict generated by client_pattern",
                    "update_client_handler",
                )

            kwargs = {
                "actual_client_key": actual_client_key,
                "updated_client": updated_client,
            }

            command_instructions = cast_command_instruction(
                "Response",
                "InternalManagement",
                "Host",
                "Success",
                "Host",
                "update_client",
                kwargs,
                "",
            )

            return command_instructions

        elif activation_function == "remove_client":
            if "client_key" in kwargs:
                pass
            else:
                self.error_response_pattern(
                    "client_key isn't in kwargs, so can't remove client!",
                    "remove_client_handler",
                )

            client_key = kwargs["client_key"]

            if isinstance(client_key, str):
                pass
            else:
                return self.error_response_pattern(
                    "client key needs to be a string!", "remove_client_handler"
                )

            kwargs = {"client_key": client_key}

            command_instructions = cast_command_instruction(
                "Response",
                "InternalManagement",
                "Host",
                "Success",
                "Host",
                "remove_client",
                kwargs,
                "",
            )

            return command_instructions

        else:
            return self.error_response_pattern(
                f"activation_function: {activation_function} doesn't registered in the available host internal management commands!",
                "remove_client_handler",
            )

    def callback_pattern(self, callback) -> dict:
        """
        Create a callback pattern.

        Parameters:
        - callback: The callback function.

        args and kwargs: Will be auto inferred by the wrapper, just add the types to your functions.

        Returns:
        - Dictionary representing the callback pattern.
        """

        # -> This is used to create a endpoint in the host that is visible to every client that has permission to see it
        # >
        # > (Client 1)       [Host]
        # >    |                |
        # >    |-------------> (|)
        # >    |                |
        # >    |<-------------- |
        # >    |                |
        # > (Client 1)        [host]
        # >
        # > (|) This is this pattern
        # >

        sig = inspect.signature(callback)
        params = sig.parameters

        args = {}

        for name, param in params.items():
            if param.annotation is inspect._empty:
                print(
                    f"function: {callback.__name__} has args or kwargs without the required types!"
                )
                return
            else:
                pass

            args[name] = str(param.annotation.__name__)

        else:
            pass

        callback_pattern = {
            "function": callback,
            "args": args,
        }

        return callback_pattern


# >-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
# > CLIENT


class MysceliumClient:
    _instance = None  # Singleton instance

    def __init__(
        self, name: str, client_uid: int, buffer_path: str, log_level: str = "DEBUG"
    ):
        """
        Initialize the MysceliumClient.

        Parameters:
        - name: The client name, doesn't need to be unique
        - client_uid: Unique identifier for the client.
        - buffer_path: Path to the buffer.
        - log_level: Logging level.
        """

        self.client_uid = client_uid

        mys.initialize_client_buffer_tables(buffer_path)
        mys.set_client_key(client_uid)

        time.sleep(5)

        self.name = name
        self.host_thread = None
        self.initialized = True

        if log_level not in ["DEBUG", "INFO", "WARN", "EXCEPTION"]:
            raise f"Log must be some of this: ('DEBUG', 'INFO', 'WARN', 'EXCEPTION') log level cant be: {log_level}"
        else:
            pass

        mys.set_socket_client_log_level(log_level)

    def __new__(cls, *args, **kwargs):
        if not cls._instance:
            cls._instance = super(MysceliumClient, cls).__new__(cls)
            # This will call your __init__, so you don't have to duplicate code
        return cls._instance

    @classmethod
    def get_instance(cls):
        if not cls._instance:
            raise ValueError("MysceliumClient instance has not been initialized")
        return cls._instance

    def clone(self):
        return self

    # def set_logs_callback_handler (self, logs_handler_callback:list):
    #     print("active py set log callback")
    #     mys.registry_client_logs_handler(logs_handler_callback)

    # def set_client_uid (self, client_uid):

    #     """
    #     Set the client's unique identifier.

    #     Parameters:
    #     - client_uid: Unique identifier for the client.
    #     """

    #     mys.set_client_uid(client_uid)

    #     return

    def is_client_ready(self):
        return mys.is_client_ready()

    def is_target_ready(self, target_key: str):
        return mys.is_target_ready(target_key)

    def set_workers_num(self, n_workers=2):
        """
        Set the number of workers for the client.

        Parameters:
        - n_workers: Number of workers.
        """

        mys.set_socket_client_transposer_num_of_workers(n_workers)

        return

    def set_callbacks(self, callbacks: list):
        """
        Register callback functions for the client.

        Parameters:
        - callbacks: List of callback functions.
        """

        special_functions = [
            {
                "function": get_registered_commands,
                "response_type": "same_as_origin",
                "args": "None",
            },
        ]

        callbacks = callbacks + special_functions

        mys.registry_socket_client_callbacks(
            callbacks
        )  #! We can change this to response handler in the future.

        return

    def get_registered_commands(self) -> dict:
        """
        Retrieve the registered commands.

        Returns:
        - Dictionary of registered commands.
        """

        print("Activated the get registered commands")

        return mys.get_socket_client_available_handlers()

    def initialize_client(self, ip: str, port: int):
        """
        Initialize the client with the given IP and port.

        Parameters:
        - ip: IP address for the client connect in host.
        - port: Port number for the client connect in host.
        """

        self.running = True
        mys.initialize_socket_client(ip, port, self.client_uid, self.name)

    def stop_client(self, signal, frame):
        """
        Stop the client. This function is intended to be called when a termination signal is received.

        Parameters:
        - signal: Signal received.
        - frame: Current stack frame.
        """

        # This function will be called when a SIGINT signal is received
        mys.stop_socket_client()

    def send(self, command: dict, priority: int):
        """
        Send a command with a specified priority.

        Parameters:
        - command: The command to be sent.
        - priority: Priority level of the command.

        Returns:
        - Response from the send operation.
        """

        print(self.running)

        if not self.running:
            raise "Client need to be running before try to send something"
        else:
            pass
        return mys.client_send(command, priority)


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


class ClientPatterns:
    def __init__(self) -> None:
        """
        Initialize the ClientPatterns class.
        """

        pass

    # def redirect_error_pattern (self, error_message:str, expected_remote_error_handler:str, redirect_to:str):

    #    #! This isn't a good idea to be implemented wright now since this can create confusions
    #    #! The way that the redirects are working now is in a restrict mode, using the host Retransmiters in a declarative way
    #    #> To use redirect create a Retransmiters, then send data from the client 1 to Retransmiters retransmit to client 2
    #    #* In the future, Retransmiters will be non-declarative with the impl of the smart redirects, this feature will allow to smart manage the clients that has
    #    #* Permission to retransmit and those that now, and then Myscelium will auto retransmit commands and responses without the need to create a Retransmiters manualy
    #    #* But this will come only when these feature of Smart Retransmiters come out, that will allow to each client see all the other clients functions if this client has permission and all the other clients handlers that this client has permission
    #    #* This also will provide a nice interface that allow to see what Sender are compatible with what receiver and connect them remotely via software like a wire in block programing, allowing to easy manage the myscelium network in flight

    #     # TODO >>> Create a test for this

    #     # > A possible impl is something like a data arg and in this arg will have sub kwargs in the dict that will formulate the response\
    #     # > So the client response will be something like:

    #     # "command" {
    #     #     "command_type":"response",
    #     #     "status": "success"
    #     #     "response_activation_function":"",
    #     #     "message":"",
    #     #     "kwargs":{"arg1": [], "arg2": "", "arg3": {}}
    #     #     "response_mode":"",
    #     # }

    #     # so basically what we will do in this case is to send all the command or remove like the response_activation_function
    #     # and keep the other things to alow the client Handler to extract the status, message, kwargs and response from the function
    #     # This aay host dont need to keep track of the client args in the handler, cause if this give a exception will be the client mistake
    #     # also if we need to check the status to take some action in case of error it also will be possible, and then if receive a act_fn that doesn't
    #     # we simple can give it a exception.

    #     # In a more extreme case we can only require one handler named router in the client side and then if this doesn't exist dont even start client
    #     # This router will be responsible to receive a entire command, so he will decide how to process it and what activation function to call
    #     # Then this will be able to send a response for host redirect if something is wrong redirecting the error for the client tha cause it and keep going

    #     # > The idea of this pattern
    #     # >
    #     # > (Client 1)     (Client 2)   [Host]
    #     # >    |                |         |
    #     # >    |--------------- | ------> |
    #     # >    |                |        [|] (retransmit the command from client 1 to client 2 via retransmiters)
    #     # >    |                | <------ |
    #     # >    | return rd err (|)        |
    #     # >    |                | ------> |
    #     # >    |                |        [|] (retransmit error - This is an internal thing)
    #     # >    |<-------------- | ------- |
    #     # >    |                |         |
    #     # > (Client 1)     (Client 2)   [Host]
    #     # >
    #     # > (|) This is this pattern
    #     # > [|] This is a host process
    #     # >

    #     if not isinstance(error_message, str):
    #         print("Error message needs to be a string!")

    #     if not isinstance(expected_remote_error_handler, str):
    #         print("Expected remote error handler needs to be a string!")

    #     # {"command_type":"response", "response_mode":"retransmit", "kwargs":kwargs, "redirect_to":retransmit_to_client_id}

    #     kwargs = []

    #     response = {
    #         "command_type":"response",
    #         "response_mode":"retransmit", # > Retransmit to the origin client
    #         "redirect_to":redirect_to,
    #         "status": "error",
    #         "activation_function":expected_remote_error_handler,
    #         "message":error_message,
    #         "kwargs":kwargs,
    #     }

    #     #! Here Doesn't need to add the origin because for cases of redirect this is done inside the host engine

    #     return response

    def client_pattern(self, client_type: str, client_id: str) -> dict:
        """
        Create a client pattern.

        Parameters:
        - client_type: Type of the client.
        - client_id: Unique identifier for the client.

        Returns:
        - Dictionary representing the client pattern.
        """

        return {"client_type": client_type, "client_id": client_id}

    def command_pattern(
        self,
        origin_key: str,
        command_function: str,
        target_key: str = "",
        kwargs: dict = {},
        message: str = "",
    ):
        """
        Constructs a command instruction for communication between clients and a host in a network system.

        This function primarily serves two scenarios:
        1. Response to Origin: The command instructs the host to send a response back to the originating client.
        2. Retransmission to Another Client: The command instructs the host to forward the response to a different client, identified by the `target_key`.

        Parameters:
        - origin_key (str): Identifier for the client that initiates the command.
        - command_function (str): The function to be executed in response to the command.
        - target_key (str, optional): Identifier for the target client to whom the response should be forwarded. Defaults to an empty string.
        - kwargs (dict, optional): Additional keyword arguments to pass along with the command. Defaults to an empty dictionary.
        - message (str, optional): A message accompanying the command. Defaults to an empty string.

        Returns:
        dict: A dictionary representing the command instruction, tailored to the specified interaction pattern.

        Notes:
        - 'Response to Origin' scenario: The response is directed back to the client who originated the command.
        - 'Retransmission to Another Client' scenario: The response is directed to a different client, specified by `target_key`, and the specified `command_function` is executed on that client. If the target client is not found, an error is returned.

        Example:
        command = command_pattern("client123", "activate", target_key="client456",
                                kwargs={"arg1": "value1"}, message="Example message")

        """

        # > The idea of this pattern
        # >
        # > (Client 1)        [Host]
        # >    |                |
        # >   (|) ------------> |
        # >    |               [|]
        # >    | <------------- |
        # >    |                |
        # > (Client 1)        [Host]
        # >
        # > (|) This is this pattern
        # > [|] This is a host process
        # >
        # > basically creates a command to send to host, when the command arrives in host the command will execute something

        command_instruction = {}

        if target_key == "":
            command_instruction = cast_command_instruction(
                "Function",
                "ExternalFunction",
                "Host",
                "Success",
                f"ClientKey({origin_key})",
                command_function,
                kwargs,
                message,
            )
        else:
            command_instruction = cast_command_instruction(
                "Function",
                "ExternalFunction",
                f"ClientKey({target_key})",
                "Success",
                f"ClientKey({origin_key})",
                command_function,
                kwargs,
                message,
            )

        return command_instruction

    def inner_management_command_pattern(
        self,
        origin_key: str,
        command_function: str,
        kwargs: dict = {},
        message: str = "",
    ):
        """
        Constructs a command instruction for communication between clients and a host in a network system.

        This function primarily serves two scenarios:
        1. Response to Origin: The command instructs the host to send a response back to the originating client.
        2. Retransmission to Another Client: The command instructs the host to forward the response to a different client, identified by the `target_key`.

        Parameters:
        - origin_key (str): Identifier for the client that initiates the command.
        - command_function (str): The function to be executed in response to the command.
        - kwargs (dict, optional): Additional keyword arguments to pass along with the command. Defaults to an empty dictionary.
        - message (str, optional): A message accompanying the command. Defaults to an empty string.

        Returns:
        dict: A dictionary representing the command instruction, tailored to the specified interaction pattern.

        Notes:
        - 'Response to Origin' scenario: The response is directed back to the client who originated the command.
        - 'Retransmission to Another Client' scenario: The response is directed to a different client, specified by `target_key`, and the specified `command_function` is executed on that client. If the target client is not found, an error is returned.

        Example:
        ```python
        command = inner_management_command_pattern(
            "client123",
            "activate",
            target_key="client456",
            kwargs={"arg1": "value1"},
            message="Example message"
        )
        ```

        """

        # > The idea of this pattern
        # >
        # > (Client 1)        [Host]
        # >    |                |
        # >   (|) ------------> |
        # >    |               [|]
        # >    | <------------- |
        # >    |                |
        # > (Client 1)        [Host]
        # >
        # > (|) This is this pattern
        # > [|] This is a host process
        # >
        # > basically creates a command to send to host, when the command arrives in host the command will execute something

        command_instruction = cast_command_instruction(
            "Function",  # Command Mode
            "DirectFunction",  # Command Type
            "Host",  # Command Target
            "Success",  # Status
            f"ClientKey({origin_key})",  # Origin
            command_function,  # act function
            kwargs,
            message,
        )

        return command_instruction

    def callback_pattern(self, callback) -> dict:
        """
        Create a callback pattern.

        Parameters:
        - callback: The callback function.

        args and kwargs: Will be auto inferred by the wrapper, just add the notes to your functions.

        Returns:
        - Dictionary representing the callback pattern.
        """

        # > The idea of this pattern
        # >
        # > (Client 1)         [Host]
        # >    |                 |
        # >    | --------------> |
        # >    |                [|]
        # >   (|) <------------- |
        # >    |                 |
        # > (Client 1)         [Host]
        # >
        # > (|) This is this callback
        # > [|] This is a host process
        # >
        # > Basically this creates a callable, that host can execute remotely by redirecting some command or sending some command

        sig = inspect.signature(callback)
        params = sig.parameters

        args = {}

        for name, param in params.items():
            if param.annotation is inspect._empty:
                function_name = callback.__name__
                raise f"function: {function_name} has args or kwargs without the required notes!"
            else:
                pass

            args[name] = str(param.annotation.__name__)

        else:
            pass

        callback_pattern = {
            "function": callback,
            "args": args,
        }

        return callback_pattern


# >-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
# > FUNCTIONS

host_patterns = HostPatterns()


def get_registered_commands() -> dict:
    """
    Retrieve the registered commands and format the response.

    Returns:
    - Dictionary representing the response to be returned to the engine.
    """

    print("Activated the get registered commands")
    response = mys.get_socket_host_available_commands()

    print(f"\nAvailable commands:\n{response}\n")

    response = host_patterns.response_pattern(
        "update_available_host_commands",
        "",  # means origin
        response,
        "",
    )

    print(f"Response to return to rust myscelium engine: {response}")

    return response


# def get_registered_commands (self) -> dict:

#         """
#         Retrieve the registered commands.

#         Returns:
#         - Dictionary of registered commands.
#         """

#         print("Activated the get registered commands")

#         return mys.get_socket_client_available_handlers()
