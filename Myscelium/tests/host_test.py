from myscelium import MysceliumHost, HostPatterns

host_patterns = HostPatterns()

def python_function(age, birth, name):

    #! Don't forget to put the args in the alphabetic order
    # Your function logic here

    print (birth)
    print (name)
    print (age)

    response = host_patterns.response_pattern(response_mode='to_origin', response_activation_function="test_handler", response={"data":'hello!'})

    return response

def test_redirect (client_id, data):

    if isinstance(client_id, str):
    
        print (f"Redicrecting data: {data} to client: {client_id}")
        response = host_patterns.response_pattern(response=data, response_mode='redirect', redirect_to_client_id=client_id)
        return response
    
    else:

        print ("Client id isn't a string, failed to redirect data!")
        return None
    
def handle_client_contact (client_id:str):
    # print("Access heartbeat handler")
    # print(f"Client: {client_id}, made contact")
    return None

def logs_handler (node_name:str, log_time:float, log_name:str, log_msg:str):
    print(f"{log_time} - {log_name} - {log_msg}")
    pass

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
    
    mys_host = MysceliumHost(callbacks=callbacks, host_id="xnsmdkeflerpfsa", allowed_clients=allowed_clients, buffer_path="Data/", n_workers=2, log_level="INFO")

    client_heart_beat_handler = [host_patterns.callback_pattern(callback=handle_client_contact, args={
        "client_id": "str",
    }),]

    mys_host.set_client_heartbeat_handler(callback=client_heart_beat_handler)

    logs_handler = [host_patterns.callback_pattern(callback=logs_handler, args={
        "node_name":"str",
        "log_time":"float",
        "log_name":"str",
        "log_msg":"str",
    }),]

    mys_host.set_logs_callback_handler(logs_handler_callback=logs_handler)

    # print(mys_host.get_registred_commands())

    mys_host.initialize_host(ip="127.0.0.1", port=4444)