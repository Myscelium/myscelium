from MysceliumWraper import MysceliumHost

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

mys_host.initialize_host(ip="127.0.0.1", port=4444)