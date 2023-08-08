from myscelium import MysceliumHost, HostPatterns

host_patterns = HostPatterns()

def python_function(age, birth, name):
    print(birth)
    print(name)
    print(age)
    response = host_patterns.response_pattern(response_mode='to_origin', response_activation_function="test_handler", response={"data":'hello!'})
    return response

def test_redirect(client_id, data):
    if isinstance(client_id, str):
        print(f"Redirecting data: {data} to client: {client_id}")
        response = host_patterns.response_pattern(response=data, response_mode='redirect', redirect_to_client_id=client_id)
        return response
    else:
        print("Client id isn't a string, failed to redirect data!")
        return None

def handle_client_contact(client_id:str):
    print("Access heartbeat handler")
    print(f"Client: {client_id}, made contact")
    return None

def run_host(ip="127.0.0.1", port=4444):
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

    mys_host = MysceliumHost(callbacks=callbacks, host_id="xnsmdkeflerpfsa", allowed_clients=allowed_clients, buffer_path="Data/", n_workers=2)

    client_heart_beat_handler = [host_patterns.callback_pattern(callback=handle_client_contact, args={
        "client_id": "str",
    }),]

    mys_host.set_client_heartbeat_handler(callback=client_heart_beat_handler)
    mys_host.initialize_host(ip=ip, port=port)
    return mys_host
