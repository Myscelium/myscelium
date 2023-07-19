from MysceliumWraper import MysceliumClient, ClientPatterns

client_patterns = ClientPatterns ()


def test_handler (data):

    print ("Receive data: ", data)

    return None


callbacks = [

    client_patterns.callback_pattern(callback=test_handler, args={
        "data" : "dict"
    }),

]


if __name__ == '__main__':
    
    mys_host = MysceliumClient(callbacks=callbacks, client_uid="some_client_id", buffer_path="ClientData/", n_workers=2)

    # print(mys_host.get_registred_commands())

    mys_host.initialize_client(ip="127.0.0.1", port=4444)