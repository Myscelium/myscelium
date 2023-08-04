from . import myscelium_engine as mys # Maybe change the rust myscelium lib to MysceliumEngine
from . import logs_retriver

from multiprocessing import Process
import pandas as pd
import time

# >-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
# > HOST



class MysceliumHostInterface:

    def __init__(self, buffer_path:str) -> None:

        """
        Initialize the MysceliumHostInterface.

        Parameters:
        - buffer_path: Path to the buffer for logs retrieval.
        """

        self.buffer_path = buffer_path
    
        self.log_callback

        self.stats = False

        self.process

        self.transposition_threads = 1

        pass

    def retrive_logs (self):

        """
        Retrieve logs and process them. If multiple threads are set, it will split the logs 
        and process them in parallel.
        """

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
                # Distribute the remainder across the initial chunks
                if remainder:
                    end += 1
                    remainder -= 1
                chunks.append(df.iloc[start:end])
                start = end
            
            return chunks

        def transpose (logs_df):

            for i in logs_df.index:

                log_id          = logs_df.loc[i, 'ID']
                log_time        = logs_df.loc[i, 'LogTime']         
                log_from_node   = logs_df.loc[i, 'NodeName']         
                log_level       = logs_df.loc[i, 'LogLevel']         
                log_msg         = logs_df.loc[i, 'LogMsg']         

                self.log_callback(log_time, log_level, log_from_node, log_msg)

                self.logs_retriver.Remove_Log(log_id)
                
                continue

            return

        while self.stats:

            logs_dict_df = self.logs_retriver.List_Logs()
            logs_df = pd.DataFrame.from_dict(logs_dict_df)

            if logs_df.empty:
                time.sleep(5)
                continue
            else:
                pass

            logs_df = logs_df.sort_values('LogTime')
            logs_df = logs_df.reset_index(drop=True)

            if self.transposition_threads > 1:
                
                logs_df_chunks = split_dataframe(logs_df, self.transposition_threads)
                
                threads = []
                
                for chunk in logs_df_chunks:
                    threads.append(Process(target=self.transpose, args=(chunk)))
                    continue
                
                for t in threads:
                    t.start()
                    continue
                
                for t in threads:
                    t.join()
                    continue

                pass
            
            else:

                transpose(logs_df)
                pass
            
            time.sleep(5)

            continue
            
        return
    
    def active_multi_handlers (self, threads_num=2):

        """
        Activate multiple handlers for processing logs.

        Parameters:
        - threads_num: Number of threads to be used for processing logs.
        """

        self.transposition_threads = threads_num

        return

    def set_logs_callback (self, callback:str):

        """
        Set the callback function for logs.

        Parameters:
        - callback: Callback function to be invoked for each log.
        """

        self.log_callback = callback

        pass

    def stop_logs_reriver (self):

        """
        Stop the logs retriever process.
        """
        
        self.stats = False
        self.process.join()
        
        return

    def start_logs_retriver (self):

        """
        Start the logs retriever process in a separate process.
        """

        # This auto set the number of thread workes and sql pool workers, to each thread have one sql pool worker garanted
        self.logs_retriver = logs_retriver.Logs_Buffer_Retriver(self.buffer_path, workers=self.transposition_threads)

        self.stats = True

        self.process = Process(target=self.retrive_logs, args=())
        self.process.start()

        return

class MysceliumHost:

    def __init__(self, callbacks:list, host_id:int, allowed_clients:list, buffer_path:str, n_workers=2, n_max_conns:int=5, log_level:str="") -> None:

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
        - If you don't se loggin level it will be deactivated by default!

        """ 

        self.logging_level = log_level

        self.host_interface = MysceliumHostInterface(buffer_path)

        self.allowed_clients = allowed_clients

        self.host_id = host_id

        special_functions = [{
            "function": get_registred_commands,
            "response_type":"same_as_origin",
            "args": "None",
        }, ]
        
        callbacks = callbacks + special_functions

        if log_level not in ["DEBUG", "INFO", "WARN", "EXCEPTION", ""]:
            raise f"Log must be some of this: ('DEBUG', 'INFO', 'WARN', 'EXCEPTION') log level cant be: {log_level}"
        else:
            pass

        mys.set_socket_host_log_level(log_level)

        mys.registry_socket_host_callbacks(callbacks)
        mys.initalize_host_buffer_tables(buffer_path)
        mys.set_socket_host_allowed_clients(self.allowed_clients)
        mys.set_socket_host_transposer_num_of_workers(n_workers)
        mys.set_socket_host_max_connections(n_max_conns)

        self.host_thread = None

        pass

    def set_logs_callback_handler (self, logs_handler_callback:object, active_multi_handlers:str=False, workers_num:str=2) -> None:

        """
        Set the logs callback handler.

        Parameters:
        - logs_handler_callback: Callback function to handle logs.
        - active_multi_handlers: Flag to activate multiple handlers.
        - workers_num: Number of workers for handling logs.
        """

        if self.logging_level == "":
            raise "To use logging you need to set a loggin level, the current logging status is deactivated!"
        else:
            pass

        if active_multi_handlers:
            self.host_interface.active_multi_handlers(workers_num)
        else:
            pass

        self.host_interface.set_logs_callback(logs_handler_callback)
        
        return
    
    # def set_client_heartbeat_handler (self, callback): #! THIS WILL NOT WORK UNTILL PYTHON POOL IS FINISHED
    #     mys.registry_socket_host_client_heartbeat_contact_callback(callback)
    
    def get_registred_commands (self) -> dict:

        """
        Retrieve the registered commands.

        Returns:
        - Dictionary of registered commands.
        """

        print("Activated the get registred commands")

        return mys.get_socket_host_available_commands()

    def initialize_host (self, ip:str, port:int):

        """
        Initialize the host with the given IP and port.

        Parameters:
        - ip: IP address for the host.
        - port: Port number for the host.
        """

        if self.logging_level != "":
            self.host_interface.start_logs_retriver()
        else:
            pass

        mys.initialize_socket_host (ip, port, self.host_id)
        
        return

    def stop_host (self, signal, frame):

        """
        Stop the host. This function is intended to be called when a termination signal is received.

        Parameters:
        - signal: Signal received.
        - frame: Current stack frame.
        """
        
        # This function will be called when a SIGINT signal is received

        mys.stop_socket_host()

        if self.logging_level != "":
            self.host_interface.stop_logs_reriver()
        else:
            pass

        return

    def send (self):
         
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

    def client_pattern (self, client_type:str, client_id:str) -> dict:

        """
        Create a client pattern.

        Parameters:
        - client_type: Type of the client.
        - client_id: Unique identifier for the client.

        Returns:
        - Dictionary representing the client pattern.
        """

        return {"client_type":client_type, "client_id":client_id}

    def response_pattern (self, response:any, response_mode:str, response_activation_function:str = None,  redirect_to_client_id:str=None) -> dict:

        """
        Create a response pattern.

        Parameters:
        - response: The actual response data.
        - response_mode: Mode of the response (e.g., 'redirect' or 'to_origin').
        - response_activation_function: Activation function for the response.
        - redirect_to_client_id: Client ID to redirect to (if response_mode is 'redirect').

        Returns:
        - Dictionary representing the response pattern.
        """

        if response_activation_function == "" or response_activation_function == None:
            raise ("Missing response_activation_function!")

        if response_mode == "redirect":

            if redirect_to_client_id != None:
                pass
            else:
                raise ("Invalid redirect! Missing client_id to redirect!")

            return {'response_mode':'redirect', 'response_activation_function':response_activation_function, 'response':response, 'redirect_to':redirect_to_client_id}

        elif response_mode == 'to_origin':

            print("Response mode set to origin")
            
            return {'response_mode':'to_origin', 'response_activation_function':response_activation_function, 'response':response}
        
        else:
            raise ("Response mode invalid! Please use one of this: ('redirect', 'to_origin')")

    def callback_pattern (self, callback, args) -> dict:

            """
            Create a callback pattern.

            Parameters:
            - callback: The callback function.
            - args: Arguments for the callback function.

            Returns:
            - Dictionary representing the callback pattern.
            """
            
            callback_pattern =  {
                "function": callback,
                "args": args,
            }
            
            return callback_pattern
    
# >-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
# > CLIENT

class MysceliumClient:

    def __init__(self, client_uid:int, buffer_path:str, log_level:str="WARN") -> None:

        """
        Initialize the MysceliumClient.

        Parameters:
        - client_uid: Unique identifier for the client.
        - buffer_path: Path to the buffer.
        - log_level: Logging level.
        """

        self.client_uid = client_uid

        self.runing = False

        mys.initalize_client_buffer_tables(buffer_path)

        if log_level not in ["DEBUG", "INFO", "WARN", "EXCEPTION"]:
            raise f"Log must be some of this: ('DEBUG', 'INFO', 'WARN', 'EXCEPTION') log level cant be: {log_level}"
        else:
            pass

        mys.set_socket_client_log_level(log_level)
        
        pass

    # def set_logs_callback_handler (self, logs_handler_callback:list):
    #     print("active py set log callback")
    #     mys.registry_client_logs_handler(logs_handler_callback)

    def set_client_uid (self, client_uid):

        """
        Set the client's unique identifier.

        Parameters:
        - client_uid: Unique identifier for the client.
        """

        mys.set_client_uid(client_uid)

        return 

    def set_workers_num (self, n_workers=2):

        """
        Set the number of workers for the client.

        Parameters:
        - n_workers: Number of workers.
        """

        mys.set_socket_client_transposer_num_of_workers(n_workers)

        return 

    def set_callbacks (self,callbacks:list):

        """
        Register callback functions for the client.

        Parameters:
        - callbacks: List of callback functions.
        """

        special_functions = [{
            "function": get_registred_commands,
            "response_type":"same_as_origin",
            "args": "None",
        }, ]

        callbacks = callbacks + special_functions

        mys.registry_socket_client_callbacks(callbacks) #! We can change this to response handler in the future.

        return 

    def get_registred_commands (self) -> dict:

        """
        Retrieve the registered commands for the client.

        Returns:
        - Dictionary of registered commands.
        """

        print("Activated the get registred commands")

        return mys.get_socket_client_available_commands()

    def initialize_client (self, ip:str, port:int):

        """
        Initialize the client with the given IP and port.

        Parameters:
        - ip: IP address for the client connect in host.
        - port: Port number for the client connect in host.
        """

        self.runing = True
        mys.initialize_socket_client (ip, port, self.client_uid)

    def stop_client (self, signal, frame):

        """
        Stop the client. This function is intended to be called when a termination signal is received.

        Parameters:
        - signal: Signal received.
        - frame: Current stack frame.
        """

        # This function will be called when a SIGINT signal is received
        mys.stop_socket_client()

    def send (self, command:dict, priority:int):

        """
        Send a command with a specified priority.

        Parameters:
        - command: The command to be sent.
        - priority: Priority level of the command.

        Returns:
        - Response from the send operation.
        """

        print(self.runing)

        if not self.runing:
            raise "Client need to be runing before try to send something"
        else:
            pass
        return mys.client_send(command, priority)
        
class ClientPatterns:

    def __init__(self) -> None:

        """
        Initialize the ClientPatterns class.
        """

        pass

    def client_pattern (self, client_type:str, client_id:str) -> dict:

        """
        Create a client pattern.

        Parameters:
        - client_type: Type of the client.
        - client_id: Unique identifier for the client.

        Returns:
        - Dictionary representing the client pattern.
        """

        return {"client_type":client_type, "client_id":client_id}

    def command_pattern (self, command_function:str, args=None):

        """
        Create a command pattern.

        Parameters:
        - command_function: Function name for the command.
        - args: Arguments for the command function (default is None).

        Returns:
        - Dictionary representing the command pattern.
        """

        if args != None:
            return {"function":command_function, "args":args}
        else:
            pass

        return {"function":command_function, "args":""}

    def response_pattern (self, response:any, response_mode:str, retransmit_to_client_id:str=None) -> dict:

        """
        Create a response pattern.

        Parameters:
        - response: The actual response data.
        - response_mode: Mode of the response (e.g., 'retransmit' or 'to_host').
        - retransmit_to_client_id: Client ID to retransmit to (if response_mode is 'retransmit').

        Returns:
        - Dictionary representing the response pattern.
        """

        if response_mode == "retransmit":

            if retransmit_to_client_id != None:
                pass
            else:
                raise ("Invalid redirect! Missing client_id to redirect!")

            return {'response_mode':'retransmit', 'response':response, 'redirect_to':retransmit_to_client_id}

        elif response_mode == 'to_host':
            
            return {'response_mode':'to_host', 'response':response}
        
        else:
            raise ("Response mode invalid! Please use one of this: ('redirect', 'same_as_origin')")

    def callback_pattern (self, callback, args) -> dict:
            
        """
        Create a callback pattern.

        Parameters:
        - callback: The callback function.
        - args: Arguments for the callback function.

        Returns:
        - Dictionary representing the callback pattern.
        """
            
        callback_pattern =  {
            "function": callback,
            "args": args,
        }
        
        return callback_pattern

# >-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
# > FUNCTIONS

host_patterns = HostPatterns()

def get_registred_commands () -> dict:

    """
    Retrieve the registered commands and format the response.

    Returns:
    - Dictionary representing the response to be returned to the engine.
    """

    print("Activated the get registred commands")
    response = mys.get_socket_host_available_commands()

    print(f"\nAvaliable commands:\n{response}\n")

    response = host_patterns.response_pattern(response=response, response_activation_function='update_avaliable_host_commands',  response_mode='to_origin')

    print(f"Response to return to rust myscelium engine: {response}")

    return response


