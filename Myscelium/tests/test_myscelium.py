import pytest
from multiprocessing import Process, Event
import time

from .host_module import run_host
from .client_module import run_client

def host_thread(event_host_received):
    print("Starting host thread...")
    host = run_host(event=event_host_received)
    print("Host initialized.")
    print("Host thread finished.")

def client_thread(event_client_received):
    print("Waiting for host to be ready...")
    time.sleep(10)
    print("Starting client thread...")
    client = run_client(event=event_client_received)
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
