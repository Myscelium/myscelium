# SPDX-License-Identifier: MPL-2.0
# Copyright © 2021-2026 Cristian Camargo Filho


#> Core:
from . import (
    myscelium_engine as mys,
)  # Maybe change the rust myscelium lib to MysceliumEngine

#> Modules:
from .common.patterns import ClientPattern
from .common.patterns import CommandInstruction
from .common.functions import cast_response_command_instruction

#> Extern:
import functools
import warnings
import pandas as pd
import time
import os
from typing import Dict

from .server.interfaces import MysceliumHostInterface
from .client.interfaces import MysceliumClientInterface

# >-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
# > HOST

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

            mys.setup_socket_host(buffer_path, log_level, n_workers, n_max_conns)

            mys.registry_socket_host_callbacks(callbacks)
            mys.set_socket_host_allowed_clients(self.allowed_clients)

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

    def response_pattern(
        self,
        activation_function: str,
        target_key: str = "",
        kwargs: dict = {},
        message="",
        auto_collect=True,
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
        - `response_type` (str, required): This is the type of the callback that you command will trigger, the availbale types are the following:
            - "DirectFunction"
            - "InternalManagement"
            - "ExternalFunction"
        - `response_target` (str, optional): This is the target that the response of your command will trigger, if not defined will be Origin as default, but you have the following options:
            - "Origin"
            - "Host"
            - "ClientKey(client_key_goes_here)"
        - `response_actf` (str, required): This is the target handler that the response of your function will activate when the target receives it, basically the name of this function

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

        if target_key == "":

            command_instructions = CommandInstruction(
                command_mode="Response",
                command_type="ExternalFunction",
                command_target="Origin",
                command_status="Success",
                command_actf=activation_function,
                command_kwargs=kwargs,
                command_message=message,
                response_type="ExternalFunction",
                response_target="Origin",
                response_actf=activation_function,
                auto_collect_response=auto_collect,
            ).format()

        else:  # Redirect case

            command_instructions = CommandInstruction(
                command_mode="Response",
                command_type="ExternalFunction",
                command_target=f"ClientKey({target_key})",
                command_status="Success",
                command_actf=activation_function,
                command_kwargs=kwargs,
                command_message=message,
                response_type="ExternalFunction",
                response_target="Origin",
                response_actf=activation_function,
                auto_collect_response=auto_collect,
            ).format()

        return command_instructions

    def error_response_pattern(
        self,
        error_message: str,
        error_handler: str = "",
        target="Origin",
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

        if not isinstance(error_handler, str):
            print("Expected remote error handler needs to be a string!")

        kwargs = {}
        command_instructions = {}

        if error_handler == "":

            command_instructions = CommandInstruction(
                command_mode="Response",
                command_type="DirectFunction",
                command_target="Origin",
                command_status="Failure",
                command_actf="error_handler",
                command_kwargs=kwargs,
                command_message=error_message,
                response_type="DirectFunction",
                response_target="Origin",
                response_actf="error_handler",
                auto_collect_response=True,
            ).format()

        else:

            command_instructions = CommandInstruction(
                command_mode="Response",
                command_type="ExternalFunction",
                command_target="Origin",
                command_status="Failure",
                command_actf=error_handler,
                command_kwargs=kwargs,
                command_message=error_message,
                response_type="ExternalFunction",
                response_target="Origin",
                response_actf=error_handler,
                auto_collect_response=True,
            ).format()

        return command_instructions

class HostConfigManager:
    def add_client(self, new_client: ClientPattern, response_actf: str) -> Dict:
        if not isinstance(new_client, ClientPattern):
            return self.error_response("new_client must be an instance of ClientPattern.")

        if not response_actf:
            return self.error_response("response_actf is required.")

        command = CommandInstruction(
            command_mode="Response",
            command_type="InternalManagement",
            command_target="Host",
            command_status="Success",
            command_actf="add_client",
            command_kwargs={"new_client": new_client.format()},
            response_type="InternalManagement",
            response_target="Origin",
            response_actf=response_actf
        )
        return command.format()

    def update_client(self, actual_client_key: str, updated_client: ClientPattern, response_actf: str) -> Dict:
        if not isinstance(updated_client, ClientPattern):
            return self.error_response("updated_client must be an instance of ClientPattern.")

        if not response_actf:
            return self.error_response("response_actf is required.")

        command = CommandInstruction(
            command_mode="Response",
            command_type="InternalManagement",
            command_target="Host",
            command_status="Success",
            command_actf="update_client",
            command_kwargs={"actual_client_key": actual_client_key, "updated_client": updated_client.format()},
            response_type="InternalManagement",
            response_target="Origin",
            response_actf=response_actf
        )
        return command.format()

    def remove_client(self, client_key: str, response_actf: str) -> Dict:
        if not response_actf:
            return self.error_response("response_actf is required.")

        command = CommandInstruction(
            command_mode="Response",
            command_type="InternalManagement",
            command_target="Host",
            command_status="Success",
            command_actf="remove_client",
            command_kwargs={"client_key": client_key},
            response_type="InternalManagement",
            response_target="Origin",
            response_actf=response_actf
        )
        return command.format()

    def error_response(self, message: str) -> Dict:
        return {"error": message, "status": "failed"}

# >-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
# > CLIENT


class MysceliumClient:
    _instance = None  # Singleton instance

    def __init__(
        self,
        name: str,
        client_uid: int,
        buffer_path: str,
        log_level: str = "DEBUG",
        is_main_process: bool = False,
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

        if log_level not in ["DEBUG", "INFO", "WARN", "EXCEPTION"]:
            raise ValueError(
                f"Log must be some of this: ('DEBUG', 'INFO', 'WARN', 'EXCEPTION') log level cant be: {log_level}"
            )
        else:
            pass

        mys.setup_client(name, client_uid, buffer_path, log_level, is_main_process)

        time.sleep(5)

        self.name = name
        self.host_thread = None
        self.initialized = True

        # mys.set_socket_client_log_level(log_level)

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

    def ensure_client_ready(self, max_attempts=10, sleep_time=1):
        attempts = 0
        while not self.is_client_ready():
            time.sleep(sleep_time)
            attempts += 1
            if attempts >= max_attempts:
                raise Exception("Take too long for the client to be ready")

    def is_target_ready(self, target_key: str):
        return mys.is_target_ready(target_key)

    def ensure_target_ready(self, target_key: str, max_attempts=10, sleep_time=1):
        attempts = 0
        while not self.is_target_ready(target_key):
            time.sleep(sleep_time)
            attempts += 1
            if attempts >= max_attempts:
                raise Exception(
                    f"Take too long for the target: {target_key} to be ready"
                )

    def wait_for_client_ready(max_attempts=10, sleep_time=1):
        def decorator(func):
            @functools.wraps(func)
            def wrapper(self, *args, **kwargs):
                # Here we use ensure_client_ready which will raise an Exception if the client isn't ready
                self.ensure_client_ready(max_attempts, sleep_time)
                return func(self, *args, **kwargs)

            return wrapper

        return decorator

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
        mys.initialize_socket_client(ip, port)

    def stop_client(self, signal, frame):
        """
        Stop the client. This function is intended to be called when a termination signal is received.

        Parameters:
        - signal: Signal received.
        - frame: Current stack frame.
        """

        # This function will be called when a SIGINT signal is received
        mys.stop_socket_client()

    @wait_for_client_ready(max_attempts=5, sleep_time=2)
    def send(self, command: dict, priority: int) -> str:
        """
        Send a command with a specified priority.

        Parameters:
        - command: The command to be sent.
        - priority: Priority level of the command.

        Returns:
        - ParityId assigned to the command scheduled, this helps to waith the response using this parity id when needed
        """

        # if not mys.is_client_ready():
        #     raise "Client need to be running before try to send something"
        # else:
        #     pass

        return mys.client_send(command, priority)

    @wait_for_client_ready(max_attempts=5, sleep_time=2)
    def wait_response(self, parity_id: str, timeout_in: int):
        """
        This method allows to waith a response by parity id, and the only requirement is;
        - parity id: this parity id is a unique string assigned to to command when sending, used to sincronize the command and response between the async system
        """
        return mys.wait_client_resp(parity_id, timeout_in)

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

    def response_pattern(
        self,
        activation_function: str,
        target_key: str = "",
        kwargs: dict = {},
        message="",
        auto_collect=True,
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
        - `response_type` (str, required): This is the type of the callback that you command will trigger, the availbale types are the following:
            - "DirectFunction"
            - "InternalManagement"
            - "ExternalFunction"
        - `response_target` (str, optional): This is the target that the response of your command will trigger, if not defined will be Origin as default, but you have the following options:
            - "Origin"
            - "Host"
            - "ClientKey(client_key_goes_here)"
        - `response_actf` (str, required): This is the target handler that the response of your function will activate when the target receives it, basically the name of this function

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

        if target_key == "":
            command_instructions = CommandInstruction(
                command_mode="Response",
                command_type="ExternalFunction",
                command_target="Origin",
                command_status="Success",
                command_actf=activation_function,
                command_kwargs=kwargs,
                command_message=message,
                response_type="ExternalFunction",
                response_target="Origin",
                response_actf=activation_function,
                auto_collect_response=auto_collect,
            ).format()

        else:  # Redirect case
            command_instructions = CommandInstruction(
                command_mode="Response",
                command_type="ExternalFunction",
                command_target=f"ClientKey({target_key})",
                command_status="Success",
                command_actf=activation_function,
                command_kwargs=kwargs,
                command_message=message,
                response_type="ExternalFunction",
                response_target="Origin",
                response_actf=activation_function,
                auto_collect_response=auto_collect,
            ).format()

        return command_instructions


    def command_pattern(
        self,
        command_function: str,
        target_key: str = "",
        kwargs: dict = {},
        message: str = "",
        response_type: str = "",
        response_target: str = "Origin",
        response_actf: str = "",
        auto_collect_response: bool = True,
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
        - response_type (str, required): This is the type of the response command, it needs to be one of the following:
            - "DirectFunction",
            - "InternalManagement",
            - "ExternalFunction",
        - response_target (strm Optional): This can be blank and will mean to send to Origin, however you can use the following options:
            - "Origin"
            - "Host"
            - "ClientKey(client_key_goes_here)"
            Just remmeber to not send to Client if in Client nor to Host if in Host, because this kind of operation isn't supported
        - response_actf (str, required): This is the Handler that the response that this command will generate that will be activated when the response arrives on target

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
            command_instruction = CommandInstruction(
                command_mode='Function',
                command_type="ExternalFunction",
                command_target="Host",
                command_status="Success",
                command_actf=command_function,
                command_kwargs=kwargs,
                command_message=message,
                response_type=response_type,
                response_target=response_target,
                response_actf=response_actf,
                auto_collect_response=auto_collect_response,
            ).format()
        else:
            command_instruction = CommandInstruction(
                command_mode='Function',
                command_type="ExternalFunction",
                command_target=f"ClientKey({target_key})",
                command_status="Success",
                command_actf=command_function,
                command_kwargs=kwargs,
                command_message=message,
                response_type=response_type,
                response_target=response_target,
                response_actf=response_actf,
                auto_collect_response=auto_collect_response,
            ).format()

        return command_instruction

    def inner_management_command_pattern(
        self,
        command_function: str,
        kwargs: dict = {},
        message: str = "",
        response_type: str = "",
        response_target: str = "Origin",
        response_actf: str = "",
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
        - response_type (str, required): This is the category of the handler that the response will trigger, it needs to be one of the options bellow:
            - "DirectFunction",
            - "InternalManagement",
            - "ExternalFunction",
        - response_target (str, optional): The default is Origin, to send back to origin, but you can use the bellow options:
            - "Origin"
            - "Host"
            - "ClientKey(client_key_goes_here)"

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

        command_instruction = CommandInstruction(
            command_mode='Function',
            command_type="DirectFunction",
            command_target="Host",
            command_status="Success",
            command_actf=command_function,
            command_kwargs=kwargs,
            command_message=message,
            response_type=response_type,
            response_target=response_target,
            response_actf=response_actf,
            auto_collect_response=True
        ).format()

        return command_instruction


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
