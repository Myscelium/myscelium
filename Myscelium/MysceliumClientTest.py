from MysceliumWraper import MysceliumClient, ClientPatterns
from threading import Thread
from _thread import *


client_patterns = ClientPatterns ()


def test_handler (data):

    print ("Receive data: ", data)

    return None


callbacks = [

    client_patterns.callback_pattern(callback=test_handler, args={
        "data" : "dict"
    }),

]

def send_some_data (data):
    pass


if __name__ == '__main__':
    
    mys_host = MysceliumClient(callbacks=callbacks, client_uid="some_client_id", buffer_path="ClientData/", n_workers=2)

    # print(mys_host.get_registred_commands())

    t1 = Thread(target=mys_host.initialize_client, args=("127.0.0.1",4444)) 
    t1.daemon = True
    t1.start()

    t2 = Thread(target=send_some_data, args=());
    t2.daemon = True
    t2.start()

    t1.start()
    t2.start()

    while True:
        pass
