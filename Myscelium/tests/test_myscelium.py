import pytest
from multiprocessing import Process, Event
import time

from .host_module import MyHost
from .client_module import MyClient

def host_thread(event_host_received):
    print("Starting host thread...")

    my_host = MyHost()
    host = my_host.run(event=event_host_received)

    print("Host initialized.")
    print("Host thread finished.")

def client_thread(event_client_received):
    print("Waiting for host to be ready...")
    time.sleep(10)
    print("Starting client thread...")
    event=event_client_received
    client_instance = MyClient()
    client_instance.run(event)
    print("Client thread finished.")

def test_communication():
    event_host_received = Event()
    event_client_received = Event()

    t1 = Process(target=host_thread, args=(event_host_received,))
    t2 = Process(target=client_thread, args=(event_client_received,))

    t1.start()
    t2.start()

    t1.join()  # Wait for the process to finish
    t2.join()

if __name__ == '__main__':
    pytest.main()
