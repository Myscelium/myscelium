from MysceliumWraper import MysceliumHost
from MysceliumWraper import callback_pattern


def python_function(age, birth, name):

    #! Don't forget to put the args in the alphabetic order
    # Your function logic here

    print (birth)
    print (name)
    print (age)

    return {"Response":"Hello!"}

# "data": {'localization': 'str', 'mail': 'str'},

callbacks = [

    callback_pattern(callback=python_function, args={
        "birth": "str",
        "name": "str",
        "age": "int",
    }),

]

if __name__ == '__main__':
    mys_host = MysceliumHost(callbacks=callbacks, client_id="xnsmdkeflerpfsa", buffer_path="Data/")

    # print(mys_host.get_registred_commands())
    mys_host.initialize_host(ip="127.0.0.1", port=4444)