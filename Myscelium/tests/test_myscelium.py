import pytest
import dill
import multiprocessing
from multiprocessing import Process, Event
import time
import os
import shutil


from .host_module import MyHost
from .client_module import MyClient

def host_thread(event_host_received):
    print("Starting host thread...")
    
    # TODO >>> Add a mecanism to test every event and then resume both the host and client returning the succssfully done events.

    host_instance = MyHost().run(event=event_host_received)

    print("Host thread finished.")

def client_thread(event_client_received):
    print("Waiting for host to be ready...")
    time.sleep(10)
    print("Starting client thread...")
    
    client_instance = MyClient()
    client_instance.run(event_key=event_client_received) 
    
    print("Client thread finished.")

def test_communication():

    # multiprocessing.set_start_method('spawn')
    # dill.settings['recurse'] = True

    # Instead of having separate events for client and host, we use a shared event for simplicity
    # The event_key 'main_event' will be used to identify this event

    if os.path.exists("ClientData/"):
        shutil.rmtree("ClientData/")

    if os.path.exists("Data/"):
        shutil.rmtree("Data/")

    t1 = Process(target=host_thread, args=('main_event',)) # Passing event_key
    t2 = Process(target=client_thread, args=('main_event',)) # Passing event_key

    t1.start()
    t2.start()

    t2.join()
    t1.join()  # Wait for the process to finish

    # event = my_host.get_event('client_contact')
    # assert event.is_set(), "Client contact event was not set!"

    # event = MyClient.get_event('main_event')
    # assert event.is_set() 

    # my_host.clear_events()
    # MyClient.clear_events()

if __name__ == '__main__':
    pytest.main()
