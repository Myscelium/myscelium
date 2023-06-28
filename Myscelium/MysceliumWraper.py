import Myscelium as ms


def python_function(name, age, birth):
    # Your function logic here
    pass

ms.registry_socket_host_callbacks([{
    "function": python_function,
    "args": {
        "name": "John",
        "age": 30,
        "birth": "1990-01-01",
    },
}, ])

ms.show_avaliable_commands()
