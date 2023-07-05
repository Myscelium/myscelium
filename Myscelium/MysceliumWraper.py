import signal
import Myscelium as mys

class MysceliumHost:

    def __init__(self, callbacks:list, host_id:int, allowed_clients:list, buffer_path:str, n_workers=2, n_max_conns:int=5) -> None:

        self.host_id = host_id

        special_functions = [{
            "function": get_registred_commands,
            "response_type":"same_as_origin",
            "args": "None",
        }, ]
        
        callbacks = callbacks + special_functions

        mys.registry_socket_host_callbacks(callbacks)
        mys.initalize_buffer_tables(buffer_path)
        mys.set_allowed_clients(allowed_clients)
        mys.set_num_of_workers(n_workers)
        mys.set_max_connections(n_max_conns)

        self.host_thread = None

        pass
    
    def get_registred_commands (self) -> dict:
        print("Activated the get registred commands")
        return mys.get_available_commands()

    def initialize_host (self, ip:str, port:int):
        mys.initialize_socket_host (ip, port, self.host_id)

    def stop_host (self, signal, frame):
        # This function will be called when a SIGINT signal is received
        mys.stop_socket_host()

    def send (self):
         pass

# -> Functions:

def client_pattern (client_type:str, client_id:str):
     return {"client_type":client_type, "client_id":client_id}

def response_pattern (response:any, response_mode:str, redirect_to_client_id:str=None):

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

def callback_pattern (callback, args):
         
        callback_pattern =  {
            "function": callback,
            "args": args,
        }
        
        return callback_pattern

def get_registred_commands () -> dict:
        print("Activated the get registred commands")
        return mys.get_available_commands()


