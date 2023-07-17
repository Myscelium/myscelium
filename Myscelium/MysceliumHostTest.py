from MysceliumWraper import MysceliumHost, HostPatterns

host_patterns = HostPatterns()

def python_function(age, birth, name):

    #! Don't forget to put the args in the alphabetic order
    # Your function logic here

    print (birth)
    print (name)
    print (age)

    response = host_patterns.response_pattern(response_mode='same_as_origin', response='hello!')

    return response

def test_redirect (client_id, data):

    if isinstance(client_id, str):
    
        print (f"Redicrecting data: {data} to client: {client_id}")
        response = host_patterns.response_pattern(response=data, response_mode='redirect', redirect_to_client_id=client_id)
        return response
    
    else:

        print ("Client id isn't a string, failed to redirect data!")
        return None

callbacks = [

    host_patterns.callback_pattern(callback=python_function, args={
        "birth": "str",
        "name": "str",
        "age": "int",
    }),

    host_patterns.callback_pattern(callback=test_redirect, args={
        "client_id" : "str", 
        "data" : "dict",
    }),

]

allowed_clients = [

    host_patterns.client_pattern(client_type="Interface", client_id="some_client_id"),
    host_patterns.client_pattern(client_type="Interface", client_id="randomsclientids"),

]

if __name__ == '__main__':
    
    mys_host = MysceliumHost(callbacks=callbacks, host_id="xnsmdkeflerpfsa", allowed_clients=allowed_clients, buffer_path="Data/", n_workers=2)

    # print(mys_host.get_registred_commands())

    mys_host.initialize_host(ip="127.0.0.1", port=4444)