import pytest
from multiprocessing import Process, Event
import time

from .host_module import MyHost
from .client_module import MyClient

def host_thread(event_host_received):
    print("Starting host thread...")
    
    my_host = MyHost()
    my_host.set_event('client_contact', Event())
    host_instance = my_host.run(event=event_host_received)
    
    print("Host initialized.")
    
    # Wait for the client to finish before stopping the host
    event_host_received.wait()
    
    # Stop the host operations
    host_instance.stop()

    print("Host thread finished.")

def client_thread(event_client_received):
    print("Waiting for host to be ready...")
    time.sleep(10)
    print("Starting client thread...")
    
    client_instance = MyClient()
    client_instance.run(event_key=event_client_received) 
    
    print("Client thread finished.")
    
    # Notify the host thread that client has finished
    event_client_received.set()

def test_communication():
    # Instead of having separate events for client and host, we use a shared event for simplicity
    # The event_key 'main_event' will be used to identify this event

    MyHost.set_event('main_event', Event())  # Creating a new event for the host
    MyClient.set_event('main_event', Event()) # Creating a new event for the client

    t1 = Process(target=host_thread, args=('main_event',)) # Passing event_key
    t2 = Process(target=client_thread, args=('main_event',)) # Passing event_key

    t1.start()
    t2.start()

    t1.join()  # Wait for the process to finish
    t2.join()

    event = MyHost.get_event('client_contact')
    assert event.is_set(), "Client contact event was not set!"

    MyHost.clear_events()
    MyClient.clear_events()

if __name__ == '__main__':
    pytest.main()
