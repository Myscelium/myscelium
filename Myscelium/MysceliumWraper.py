import Myscelium as mys

class MysceliumHost:

    def __init__(self, callcks:list, client_id:int, buffer_path:str) -> None:

        mys.registry_socket_host_callbacks(callcks)
        mys.initalize_buffer_tables(buffer_path)

        pass

    def get_registred_commands (self) -> dict:
        return mys.get_available_commands()

def python_function(name, age, birth):
    # Your function logic here

    print (name)

    pass

callbacks = [{
                "function": python_function,
                "args": {
                    "name": "str",
                    "age": "int",
                    "birth": "str",
                    "data": {'localization': 'str', 'mail': 'str'},
                },
            }, ]

mys_host = MysceliumHost(callcks=callbacks, client_id="xnsmdkeflerpfsa", buffer_path="Data/")

print(mys_host.get_registred_commands())



# ms.initialize_socket_host(ip='127.0.0.1', port=4444)
