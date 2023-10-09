import pytest
import shutil
import pandas as pd
import os
import signal
import time

# > Test Connection 
from .test_connection.host_module import MyHost as MyHostToTestCommunication
from .test_connection.client_1_module import MyClient as MyClient1ToTestCommunication

# > Test Redirect
from .test_redirect.host_module import MyHost as MyHostToTestRedirect
from .test_redirect.client_1_module import MyClient as MyClient1ToTestRedirect
from .test_redirect.client_2_module import MyClient as MyClient2ToTestRedirect

#> Test Inner Mannangement
from .test_mannangement.host_module import MyHost as MyHostToTestMannangement
from .test_mannangement.client_1_module import MyClient as MyClient1ToTestMannangement

#> Events mananger
from multiprocessing import Process
from .Logs.test_logs_mananger import Events_Mananger, System_Status

Events_Mananger(Unit="Client1", path="Logs").drop_events_table() # To reset in the next iteration
Events_Mananger(Unit="Host", path="Logs").drop_events_table() # To reset in the next iteration


# -> ----------------------------------------------------------------------------------------------------------------------------
# -> Tests:

#> Communication Test

def host_thread_to_test_communication(event_host_received):
    print("Starting host thread...")
    
    # TODO >>> Add a mecanism to test every event and then resume both the host and client returning the succssfully done events.

    host_instance = MyHostToTestCommunication().run(event=event_host_received)

    print("Host thread finished.")

def client_1_thread_to_test_communication(event_client_received):
    print("Waiting for host to be ready...")
    time.sleep(15)
    print("Starting client 1 thread...")
    
    client_instance = MyClient1ToTestCommunication()
    client_instance.run() 
    
    print("Client1 thread finished.")


def test_communication():
    
    # multiprocessing.set_start_method('spawn')
    # dill.settings['recurse'] = True

    # Instead of having separate events for client and host, we use a shared event for simplicity
    # The event_key 'main_event' will be used to identify this event

    System_Status(path="Logs").create_unit("Client1")
    System_Status(path="Logs").create_unit("Host")

    System_Status(path="Logs").change_unit_status(Unit="Client1", Status=True)
    System_Status(path="Logs").change_unit_status(Unit="Host", Status=True)

    if os.path.exists("Temp/Client1Data/"):
        shutil.rmtree("Temp/Client1Data/")

    if os.path.exists("Temp/Client2Data/"):
        shutil.rmtree("Temp/Client2Data/")

    if os.path.exists("Temp/Data/"):
        shutil.rmtree("Temp/Data/")

    t1 = Process(target=host_thread_to_test_communication, args=('main_event',)) # Passing event_key
    t2 = Process(target=client_1_thread_to_test_communication, args=('main_event',)) # Passing event_key

    t1.start()
    t2.start()

    t2.join()
    t1.join()  # Wait for the process to finish

    #* To test:

    #> Client1 Initializes
    #> Host initializes

    #> Client1 make contact
    #> Client1 sync commands avalaible
    #> Client1 schedule to send things
    #> Client1 send command
    
    #> Host received client comamnd
    #> Host Returned command to client

    #> Client1 Receive Host response
    
    #> Finish Client1
    #> Finish Host

    host_events = Events_Mananger(Unit="Host", path="Logs").List_Events()
    host_events_df = pd.DataFrame.from_dict(host_events)

    client_1_events = Events_Mananger(Unit="Client1", path="Logs").List_Events() 
    client_1_events_df = pd.DataFrame.from_dict(client_1_events)

    # -> Host events:

    client_contact  = False
    basic_callback  = False 

    # -> Client1 events:

    send_data               = False
    basic_response_handler  = False

    for i in host_events_df.index:
        event = host_events_df.loc[i, 'StepCompleted']

        if "Contact received from Client: some_client_id" in event:
            client_contact = True

        if "Active Basic Callback" in event:
            basic_callback = True

    for i in client_1_events_df.index:
        event = client_1_events_df.loc[i, 'StepCompleted']

        if "Data Sended" in event:
            send_data = True

        if "Activate Basic Response Test callback handler" in event:
            basic_response_handler = True
    
 
    # -> Client1

    assert send_data, "Cant send data"
    assert basic_response_handler, "Don't called basic response handler"
    
    # -> Host

    assert client_contact, "Client1 doesn't made any contact"
    assert basic_callback, "Baisc callback not called"

    # TODO >>> When add the client tables mecanism re add the client contact test unit
    # TODO >>> Add a test mecanism to check if the logs are being stored and transposing

    # TODO >>> Add a mecanism to call permission to realize the tests and give an advice that data in the buffers will be wipped of when do the test

    # event = my_host.get_event('client_contact')
    # assert event.is_set(), "Client1 contact event was not set!"

    # event = MyClient.get_event('main_event')
    # assert event.is_set() 

    # my_host.clear_events()
    # MyClient.clear_events()


#> ------------------------------------------------------------------------------------------------------------------------------------
#> Redirect Test:

def host_thread_to_test_redirect(event_host_received):
    print("Starting host thread...")
    
    # TODO >>> Add a mecanism to test every event and then resume both the host and client returning the succssfully done events.

    host_instance = MyHostToTestRedirect().run(event=event_host_received)

    print("Host thread finished.")

def client_1_thread_to_test_redirect(event_client_received):
    print("Waiting for host to be ready...")
    time.sleep(5)
    print("Starting client 1 thread...")
    
    client_instance = MyClient1ToTestRedirect()
    client_instance.run() 
    
    print("Client1 thread finished.")

def client_2_thread_to_test_redirect(event_client_received):
    print("Waiting for host to be ready...")
    time.sleep(5)
    print("Starting client 2 thread...")
    
    client_instance = MyClient2ToTestRedirect()
    client_instance.run() 
    
    print("Client2 thread finished.")

def test_redirect ():

    time.sleep(5)

    System_Status(path="Logs").create_unit("Client1")
    System_Status(path="Logs").create_unit("Client2")
    System_Status(path="Logs").create_unit("Host")

    System_Status(path="Logs").change_unit_status(Unit="Client1", Status=True)
    System_Status(path="Logs").change_unit_status(Unit="Client2", Status=True)
    System_Status(path="Logs").change_unit_status(Unit="Host", Status=True)

    if os.path.exists("Temp/Client1Data/"):
        shutil.rmtree("Temp/Client1Data/")

    if os.path.exists("Temp/Client2Data/"):
        shutil.rmtree("Temp/Client2Data/")

    if os.path.exists("Temp/Data/"):
        shutil.rmtree("Temp/Data/")

    t1 = Process(target=host_thread_to_test_redirect, args=('main_event',)) # Passing event_key
    t2 = Process(target=client_1_thread_to_test_redirect, args=('main_event',)) # Passing event_key
    t3 = Process(target=client_2_thread_to_test_redirect, args=('main_event',)) # Passing event_key

    t1.start()
    t2.start()
    t3.start()

    t2.join()
    t3.join()
    t1.join()  # Wait for the process to finish

    host_events = Events_Mananger(Unit="Host", path="Logs").List_Events()
    host_events_df = pd.DataFrame.from_dict(host_events)

    client_1_events = Events_Mananger(Unit="Client1", path="Logs").List_Events() 
    client_1_events_df = pd.DataFrame.from_dict(client_1_events)

    client_2_events = Events_Mananger(Unit="Client2", path="Logs").List_Events() 
    client_2_events_df = pd.DataFrame.from_dict(client_2_events)

    #>----------------------------------------------------------------------------------------------------
    #> Tests Controler

    #> Host events:

    client_2_contact            = False
    client_1_contact            = False
    client_contact              = False
    basic_callback              = False 
    host_redirect_callback      = False

    #> Client 1 events:

    send_data                   = False
    basic_response_handler      = False
    active_callback_remotely    = False #* Active callback from another client
    remote_act_response_sended  = False #* Response of the remote activation (Another Redirect to client)

    # > Client 2 events:

    send_data_to_redirect       = False
    # redirected_request_response = False #* Response from the remote callback activated

    #>----------------------------------------------------------------------------------------------------

    # -> Host Tests
    for i in host_events_df.index:
        event = host_events_df.loc[i, 'StepCompleted']

        if "Contact received from Client: some_client_id" in event:
            client_1_contact = True

        if "Contact received from Client: randomsclientids" in event:
            client_2_contact = True

        if "Active Host Redirect Callback" in event:
            host_redirect_callback = True

    # -> Client 1 Tests
    for i in client_1_events_df.index:
        event = client_1_events_df.loc[i, 'StepCompleted']

        # if "Data Sended" in event:
        #     send_data = True

        # if "Activate Basic Response Test callback handler" in event:
        #     basic_response_handler = True

        if "Activate Basic Redirect Test callback handler" in event:
            active_callback_remotely = True

    # -> Client 2 Tests
    for i in client_2_events_df.index:
        event = client_2_events_df.loc[i, 'StepCompleted']

        if "Data To Redirect Sended" in event:
            send_data_to_redirect = True
 
    # -> Client 1

    # assert send_data, "Cant send data"
    # assert basic_response_handler, "Don't called basic response handler"
    assert active_callback_remotely, "Don't received redirect response!"

    # -> Client 2

    assert send_data_to_redirect, "Don't could send data to redirect!"

    # -> Host
    assert client_1_contact, "Client 1 doesn't make any contact with host!"
    assert client_2_contact, "Client 2 doesn't make any contact with host!"
    # assert client_contact, "Client1 doesn't made any contact"
    assert host_redirect_callback, "Baisc redirect callback not called!"

    # TODO >>> When add the client tables mecanism re add the client contact test unit
    # TODO >>> Add a test mecanism to check if the logs are being stored and transposing

    # TODO >>> Add a mecanism to call permission to realize the tests and give an advice that data in the buffers will be wipped of when do the test

    # event = my_host.get_event('client_contact')
    # assert event.is_set(), "Client1 contact event was not set!"

    # event = MyClient.get_event('main_event')
    # assert event.is_set() 

    # my_host.clear_events()
    # MyClient.clear_events()


    pass


#> ------------------------------------------------------------------------------------------------------------------------------------
#> Inner Mannangement Test:

def host_thread_to_test_inner_mannangement (event_host_received):
    print("Starting host thread...")
    
    # TODO >>> Add a mecanism to test every event and then resume both the host and client returning the succssfully done events.

    host_instance = MyHostToTestMannangement().run(event=event_host_received)

    print("Host thread finished.")

def client_1_thread_to_test_inner_mannangement (event_client_received):
    print("Waiting for host to be ready...")
    time.sleep(5)
    print("Starting client 1 thread...")
    
    client_instance = MyClient1ToTestMannangement()
    client_instance.run() 
    
    print("Client1 thread finished.")

def test_mannangement ():

    time.sleep(5)

    System_Status(path="Logs").create_unit("Client1")
    System_Status(path="Logs").create_unit("Host")

    System_Status(path="Logs").change_unit_status(Unit="Client1", Status=True)
    System_Status(path="Logs").change_unit_status(Unit="Host", Status=True)

    if os.path.exists("Temp/Client1Data/"):
        shutil.rmtree("Temp/Client1Data/")

    if os.path.exists("Temp/Data/"):
        shutil.rmtree("Temp/Data/")

    t1 = Process(target=host_thread_to_test_inner_mannangement, args=('main_event',)) # Passing event_key
    t2 = Process(target=client_1_thread_to_test_inner_mannangement, args=('main_event',)) # Passing event_key

    t1.start()
    t2.start()

    t2.join()

    host_events = Events_Mananger(Unit="Host", path="Logs").List_Events()
    host_events_df = pd.DataFrame.from_dict(host_events)

    client_1_events = Events_Mananger(Unit="Client1", path="Logs").List_Events() 
    client_1_events_df = pd.DataFrame.from_dict(client_1_events)

    #>----------------------------------------------------------------------------------------------------
    #> Tests Controler

    #> Host events:

    # TODO >>> Continue to implement the tests to avaliate if the test inner mannangement is working!

    client_1_contact            = False
    client_contact              = False
    
    add_client_callback         = False 
    update_client_callback      = False 
    remove_client_callback      = False 

    #> Client 1 events:

    send_add_client             = False
    send_update_client          = False
    send_remove_client          = False

    receive_add_client_conf     = False
    receive_update_client_conf  = False
    receive_remove_client_conf  = False

    #>----------------------------------------------------------------------------------------------------

    # -> Host Tests
    for i in host_events_df.index:
        event = host_events_df.loc[i, 'StepCompleted']

        if "Active Test Add Client" in event:
            add_client_callback     = True

        if "Active Test Update Client" in event:
            update_client_callback  = True

        if "Active Test Remove Client" in event:
            remove_client_callback  = True

    # -> Client 1 Tests
    for i in client_1_events_df.index:
        event = client_1_events_df.loc[i, 'StepCompleted']

        # > Senders

        if "Send test add a client" in event:
            send_add_client     = True

        if "Send test update a client" in event:
            send_update_client  = True

        if "Send test remove a client" in event:
            send_remove_client  = True

        # > Receivers   
        
        if "Activate Basic Response Test Add Client" in event:
            receive_add_client_conf = True

        if "Activate Basic Response Test Update Client" in event:
            receive_update_client_conf = True

        if "Activate Basic Response Test Remove Client" in event:
            receive_remove_client_conf = True

 
    # -> Client 1

    assert send_add_client, "Can't send command to add client!"
    assert send_update_client, "Can't send command to update client!"
    assert send_remove_client, "Can't send command to remove client!"

    assert receive_add_client_conf, "Can't receive client creation conf"
    assert receive_update_client_conf, "Can't receive client update conf"
    assert receive_remove_client_conf, "Can't receive remove client conf"

    # -> Host

    assert add_client_callback, "Host don't receive add client!"
    assert update_client_callback, "Host don't receive update client!"
    assert remove_client_callback, "Host don't receive remove client!"

    

# def test_communication_resistance():
#     success_count = 0
#     total_attempts = 100

#     for _ in range(total_attempts):
#         try:
#             test_communication()
#             success_count += 1
#         except Exception as e:
#             print(f"Failed on attempt {_ + 1} with error: {e}")

#     success_porcentage = (success_count/total_attempts)*100
#     print(f"\n\nTest succeeded {success_count} out of {total_attempts} attempts. Test have {success_porcentage}% of success")
#     assert success_count == total_attempts, "Not all attempts were successful!"




if __name__ == '__main__':
    pytest.main()
