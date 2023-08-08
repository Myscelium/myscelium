import pytest
import shutil
import pandas as pd
from .host_module import MyHost
from .client_module import MyClient
from multiprocessing import Process
from .Logs.test_logs_mananger import Events_Mananger, System_Status

Events_Mananger(Unit="Client", path="Logs").drop_events_table() # To reset in the next iteration
Events_Mananger(Unit="Host", path="Logs").drop_events_table() # To reset in the next iteration

import os
import signal
import time

# -> Timeout handlers:

import psutil

def teardown_function(function):
    for proc in psutil.process_iter():
        try:
            # This is just an example; replace the condition with your specific criteria.
            if 'specific_marker_or_name' in proc.name():
                proc.terminate()
        except (psutil.NoSuchProcess, psutil.AccessDenied, psutil.ZombieProcess):
            pass


# -> Tests:

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
    client_instance.run() 
    
    print("Client thread finished.")


def test_communication_process ():

    # multiprocessing.set_start_method('spawn')
    # dill.settings['recurse'] = True

    # Instead of having separate events for client and host, we use a shared event for simplicity
    # The event_key 'main_event' will be used to identify this event

    System_Status(path="Logs").create_unit("Client")
    System_Status(path="Logs").create_unit("Host")

    System_Status(path="Logs").change_unit_status(Unit="Client", Status=True)
    System_Status(path="Logs").change_unit_status(Unit="Host", Status=True)

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

    #* To test:

    #> Client Initializes
    #> Host initializes

    #> Client make contact
    #> Client sync commands avalaible
    #> Client schedule to send things
    #> Client send command
    
    #> Host received client comamnd
    #> Host Returned command to client

    #> Client Receive Host response
    
    #> Finish Client
    #> Finish Host

    host_events = Events_Mananger(Unit="Client", path="Logs").List_Events()
    host_events_df = pd.DataFrame.from_dict(host_events)

    client_events = Events_Mananger(Unit="Client", path="Logs").List_Events() 
    client_events_df = pd.DataFrame.from_dict(client_events)

    # -> Host events:

    client_contact  = False
    basic_callback  = False 

    # -> Client events:

    send_data               = False
    basic_response_handler  = False

    for i in host_events_df.index:
        event = host_events_df.loc[i, 'StepCompleted']

        if "Contact received from Client: some_client_id" in event:
            client_contact = True

        if "Active Basic Callback" in event:
            basic_callback = True

    for i in client_events_df.index:
        event = client_events_df.loc[i, 'StepCompleted']

        if "Data Sended" in event:
            send_data = True

        if "Activate Basic Response Test callback handler" in event:
            basic_response_handler = True
    
 
    # -> Client

    assert send_data, "Cant send data"
    assert basic_response_handler, "Don't called basic response handler"
    
    # -> Host

    assert client_contact, "Client doesn't made any contact"
    assert basic_callback, "Baisc callback not called"


    # event = my_host.get_event('client_contact')
    # assert event.is_set(), "Client contact event was not set!"

    # event = MyClient.get_event('main_event')
    # assert event.is_set() 

    # my_host.clear_events()
    # MyClient.clear_events()

def test_communication():
    
    p = Process(target=test_communication_process)
    p.start()

    # Monitor the process for timeout
    p.join(timeout=5)  # Let's say a 5-second timeout

    # If process is alive after timeout, terminate it
    if p.is_alive():
        print("Test timed out! Terminating...")
        p.terminate()
        p.join()  # Make sure the process has finished cleanup

    # Asserts or any post-processing for the test can go here.
    # Note: if you have asserts within test_communication_process, those will not affect this outer test.
    # You'd need to communicate results using multiprocessing primitives or other methods to perform asserts here.

if __name__ == '__main__':
    pytest.main()
