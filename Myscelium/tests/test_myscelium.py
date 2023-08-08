import pytest
from multiprocessing import Process, Queue, set_start_method
import time

# Import the run_host and run_client functions from your modules
from .host_module import run_host
from .client_module import run_client

def host_process(queue):
    run_host()  # This will initialize and start the host
    queue.put("Host Done")

def client_process(queue):
    run_client()  # This will initialize and start the client
    queue.put("Client Done")

def test_communication():
    q1, q2 = Queue(), Queue()

    # Start the host and client processes
    p1 = Process(target=host_process, args=(q1,))
    p2 = Process(target=client_process, args=(q2,))

    p1.start()
    p2.start()

    # Set a timeout for the test
    timeout = time.time() + 10   # 10 seconds
    while True:
        if not q1.empty() and not q2.empty():
            p1.terminate()
            p2.terminate()
            break
        if time.time() > timeout:
            p1.terminate()
            p2.terminate()
            assert False, "Test timed out"
            return

    # Check if both processes completed successfully
    assert q1.get() == "Host Done"
    assert q2.get() == "Client Done"

# This ensures the multiprocessing code only runs if the script is the main point of execution.
if __name__ == '__main__':
    pytest.main()
