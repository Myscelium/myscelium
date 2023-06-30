import Myscelium as mys

class MysceliumHost:

    def __init__(self, callcks:list, client_id:int, buffer_path:str, n_workers=2, n_max_conns:int=5) -> None:

        self.client_id = client_id

        mys.registry_socket_host_callbacks(callcks)
        mys.initalize_buffer_tables(buffer_path)
        mys.set_num_of_workers(n_workers)
        mys.set_max_connections(n_max_conns)

        pass

    def get_registred_commands (self) -> dict:
        return mys.get_available_commands()
    
    def initialize_host (self, ip:str, port:int):
        mys.initialize_socket_host (ip, port, self.client_id)







