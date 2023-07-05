from MysceliumWraper import MysceliumHost
from MysceliumWraper import callback_pattern, response_pattern


def python_function(age, birth, name):

    #! Don't forget to put the args in the alphabetic order
    # Your function logic here

    print (birth)
    print (name)
    print (age)

    response = response_pattern(response_mode='same_as_origin', response='hello!')

    return response

def test_redirect (client_id, data):

    if isinstance(client_id, str):
    
        print (f"Redicrecting data: {data} to client: {client_id}")
        response = response_pattern(response_mode='redirect', redirect_to_client_id=client_id, response=data)
        return response
    
    else:

        print ("Client id isn't a string, failed to redirect data!")
        return None


# "data": {'localization': 'str', 'mail': 'str'},

callbacks = [

    callback_pattern(callback=python_function, args={
        "birth": "str",
        "name": "str",
        "age": "int",
    }),

    callback_pattern(callback=test_redirect, args={
        "client_id" : "str", 
        "data" : "dict",
    }),

]

if __name__ == '__main__':
    mys_host = MysceliumHost(callbacks=callbacks, client_id="xnsmdkeflerpfsa", buffer_path="Data/")

    # print(mys_host.get_registred_commands())
    mys_host.initialize_host(ip="127.0.0.1", port=4444)