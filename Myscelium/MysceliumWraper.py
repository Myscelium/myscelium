import signal
import Myscelium as mys # Maybe change the rust myscelium lib to MysceliumEngine

class MysceliumHost:

    def __init__(self, callbacks:list, host_id:int, allowed_clients:list, buffer_path:str, n_workers=2, n_max_conns:int=5) -> None:

        self.allowed_clients = allowed_clients

        self.host_id = host_id

        special_functions = [{
            "function": get_registred_commands,
            "response_type":"same_as_origin",
            "args": "None",
        }, ]
        
        callbacks = callbacks + special_functions

        mys.registry_socket_host_callbacks(callbacks)
        mys.initalize_host_buffer_tables(buffer_path)
        mys.set_socket_host_allowed_clients(self.allowed_clients)
        mys.set_socket_host_transposer_num_of_workers(n_workers)
        mys.set_socket_host_max_connections(n_max_conns)

        self.host_thread = None

        pass
    
    def get_registred_commands (self) -> dict:
        print("Activated the get registred commands")
        return mys.get_socket_host_available_commands()

    def initialize_host (self, ip:str, port:int):
        mys.initialize_socket_host (ip, port, self.host_id)

    def stop_host (self, signal, frame):
        # This function will be called when a SIGINT signal is received
        mys.stop_socket_host()

    def send (self):
         pass
    
class HostPatterns:

    def __init__(self) -> None:
        pass

    def client_pattern (self, client_type:str, client_id:str) -> dict:
        return {"client_type":client_type, "client_id":client_id}

    def response_pattern (self, response:any, response_mode:str, redirect_to_client_id:str=None) -> dict:

        if response_mode == "redirect":

            if redirect_to_client_id != None:
                pass
            else:
                raise ("Invalid redirect! Missing client_id to redirect!")

            return {'response_mode':'redirect', 'response':response, 'redirect_to':redirect_to_client_id}

        elif response_mode == 'same_as_origin':
            
            return {'response_mode':'same_as_origin', 'response':response}
        
        else:
            raise ("Response mode invalid! Please use one of this: ('redirect', 'same_as_origin')")

    def callback_pattern (self, callback, args) -> dict:
            
            callback_pattern =  {
                "function": callback,
                "args": args,
            }
            
            return callback_pattern

class MysceliumClient:

    def __init__(self, callbacks:list, client_uid:int, buffer_path:str, n_workers=2) -> None:

        self.client_uid = client_uid

        special_functions = [{
            "function": get_registred_commands,
            "response_type":"same_as_origin",
            "args": "None",
        }, ]
        
        callbacks = callbacks + special_functions

        mys.registry_socket_client_callbacks(callbacks) #! We can change this to response handler in the future.
        mys.initalize_client_buffer_tables(buffer_path)
        mys.set_socket_client_transposer_num_of_workers(n_workers)

        self.host_thread = None

        pass
    
    def get_registred_commands (self) -> dict:
        print("Activated the get registred commands")
        return mys.get_socket_client_available_commands()

    def initialize_client (self, ip:str, port:int):
        mys.initialize_socket_client (ip, port, self.client_uid)

    def stop_client (self, signal, frame):
        # This function will be called when a SIGINT signal is received
        mys.stop_socket_client()

    def send (self, command:dict, priority:int):
        mys.client_send(command, priority)
        
class ClientPatterns:

    def __init__(self) -> None:
        pass

    def client_pattern (self, client_type:str, client_id:str) -> dict:
        return {"client_type":client_type, "client_id":client_id}

    def response_pattern (self, response:any, response_mode:str, retransmit_to_client_id:str=None) -> dict:

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
            
            callback_pattern =  {
                "function": callback,
                "args": args,
            }
            
            return callback_pattern

# -> Functions:

def get_registred_commands () -> dict:
    print("Activated the get registred commands")
    return mys.get_available_commands()


