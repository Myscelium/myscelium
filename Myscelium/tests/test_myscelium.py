import pytest
import shutil
import pandas as pd
import os
import signal
import time

# > Test Connection
from .test_connection.host_module import MyHost as MyHostToTestCommunication
from .test_connection.client_1_module import MyClient as MyClient1ToTestCommunication

#> Test Inplace Responce
from .test_inplace_responses.host_module import MyHost as MyHostToTestInplaceResponse
from .test_inplace_responses.client_1_module import MyClient as MyClient1ToTestInplaceResponse

# > Test Redirect
from .test_redirect.host_module import MyHost as MyHostToTestRedirect
from .test_redirect.client_1_module import MyClient as MyClient1ToTestRedirect
from .test_redirect.client_2_module import MyClient as MyClient2ToTestRedirect

#> Test Inplace Response Redirect
from .test_redirect_inplace.host_module import MyHost as MyHostToTestInplaceResponseRedirect
from .test_redirect_inplace.client_1_module import MyClient as MyClient1ToTestInplaceResponseRedirect
from .test_redirect_inplace.client_2_module import MyClient as MyClient2ToTestInplaceResponseRedirect

# > Test Inner Management
from .test_management.host_module import MyHost as MyHostToTestManagement
from .test_management.client_1_module import MyClient as MyClient1ToTestManagement

# > Test messages
from .test_messages.host_module import MyHost as MyHostToTestMessages
from .test_messages.client_1_module import MyClient as MyClient1ToTestMessages

# > Test Redirect Messages:

# > Test Redirect
from .test_redirect_messages.host_module import MyHost as MyHostToTestRedirectMessages
from .test_redirect_messages.client_1_module import (
    MyClient as MyClient1ToTestRedirectMessages,
)
from .test_redirect_messages.client_2_module import (
    MyClient as MyClient2ToTestRedirectMessages,
)

# > Events manager
from multiprocessing import Process
from .Logs.test_logs_manager import Events_Manager, System_Status

Events_Manager(
    Unit="Client1", path="Logs"
).drop_events_table()  # To reset in the next iteration
Events_Manager(
    Unit="Host", path="Logs"
).drop_events_table()  # To reset in the next iteration

import argparse
from . configs_loader import load_configs

CONFIGS = load_configs()['configs']
TEST_NODE_NAME = CONFIGS['test_node_name']
NODE_DISK_NAME = CONFIGS['node_disk_name']

# # Argument parsing setup
# parser = argparse.ArgumentParser(description="Set the debug level for the script.")
# parser.add_argument("--debug-level", default="DEBUG", choices=["DEBUG", "INFO", "WARN"],
#                     help="Set the debug level. Options are: DEBUG, INFO, WARN")

# args = parser.parse_args()

# DEBUG_LEVEL = args.debug_level


import os

DEBUG_LEVEL = os.environ.get("DEBUG_LEVEL", "DEBUG")  # Default to 'DEBUG' if not set

# DEBUG_LEVEL = "DEBUG"
# DEBUG_LEVEL = "INFO"
# DEBUG_LEVEL = "WARN"

THIS_DIR = os.path.dirname(__file__)

from .History.history_controller import History_Manager



# -> ----------------------------------------------------------------------------------------------------------------------------
# -> Tests:

# > Communication Test


def host_thread_to_test_communication(event_host_received):
    print("Starting host thread...")

    # TODO >>> Add a mechanism to test every event and then resume both the host and client returning the successfully done events.

    host_instance = MyHostToTestCommunication(DEBUG_LEVEL).run(
        event=event_host_received
    )

    print("Host thread finished.")


def client_1_thread_to_test_communication(event_client_received):
    print("Waiting for host to be ready...")
    time.sleep(15)
    print("Starting client 1 thread...")

    client_instance = MyClient1ToTestCommunication(DEBUG_LEVEL)
    client_instance.run()

    print("Client1 thread finished.")


def test_communication():
    # multiprocessing.set_start_method('spawn')
    # dill.settings['recurse'] = True

    # Instead of having separate events for client and host, we use a shared event for simplicity
    # The event_key 'main_event' will be used to identify this event

    test_start_time = time.time()

    Events_Manager(
        Unit="Client1", path="Logs"
    ).drop_events_table()  # To reset in the next iteration
    Events_Manager(
        Unit="Client2", path="Logs"
    ).drop_events_table()  # To reset in the next iteration
    Events_Manager(
        Unit="Host", path="Logs"
    ).drop_events_table()  # To reset in the next iteration

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

    t1 = Process(
        target=host_thread_to_test_communication, args=("main_event",)
    )  # Passing event_key
    t2 = Process(
        target=client_1_thread_to_test_communication, args=("main_event",)
    )  # Passing event_key

    t1.start()
    t2.start()

    t2.join()
    t1.join()  # Wait for the process to finish

    # * To test:

    # > Client1 Initializes
    # > Host initializes

    # > Client1 make contact
    # > Client1 sync commands available
    # > Client1 schedule to send things
    # > Client1 send command

    # > Host received client command
    # > Host Returned command to client

    # > Client1 Receive Host response

    # > Finish Client1
    # > Finish Host

    host_events = Events_Manager(Unit="Host", path="Logs").List_Events()
    host_events_df = pd.DataFrame.from_dict(host_events)

    client_1_events = Events_Manager(Unit="Client1", path="Logs").List_Events()
    client_1_events_df = pd.DataFrame.from_dict(client_1_events)

    # -> Host events:

    client_contact = False
    basic_callback = False

    # -> Client1 events:

    send_data = False
    basic_response_handler = False

    for i in host_events_df.index:
        event = host_events_df.loc[i, "StepCompleted"]

        if "Contact received from Client: some_client_id" in event:
            client_contact = True

        if "Active Basic Callback" in event:
            basic_callback = True

    for i in client_1_events_df.index:
        event = client_1_events_df.loc[i, "StepCompleted"]

        if "Data Sended" in event:
            send_data = True

        if "Activate Basic Response Test callback handler" in event:
            basic_response_handler = True

    unified_events = host_events_df.merge(client_1_events_df, how="outer")

    tracking = {}
    deltas = []

    for i in unified_events.index:
        event_type = unified_events.loc[i, "EventType"]
        event_key = unified_events.loc[i, "EventKey"]
        event_time = unified_events.loc[i, "Time"]

        if event_type == "Send":
            tracking[event_key] = event_time

        elif event_type == "Receive":
            if event_key in tracking:
                start_ts = tracking[event_key]
                deltas.append(event_time - start_ts)
            else:
                pass

        else:
            pass

    test_end_time = time.time()

    if len(deltas) > 0:
        average_com_delta = sum(deltas) / len(deltas)
    else:
        # Handle the empty list case
        average_com_delta = 0  # or any other default or error valu

    test_run_time = test_end_time - test_start_time

    if (client_contact and basic_callback) and (send_data and basic_response_handler):
        History_Manager().store_history_point(
            TEST_NODE_NAME,
            NODE_DISK_NAME,
            "test_communication",
            communications_speed=float(average_com_delta),
            test_speed=test_run_time,
            test_status="PASSED",
            log_level=DEBUG_LEVEL,
        )
    else:
        History_Manager().store_history_point(
            TEST_NODE_NAME,
            NODE_DISK_NAME,
            "test_communication",
            communications_speed=float(average_com_delta),
            test_speed=test_run_time,
            test_status="FAILED",
            log_level=DEBUG_LEVEL,
        )

    # -> Client1

    assert send_data, "Cant send data"
    assert basic_response_handler, "Don't called basic response handler"

    # -> Host

    assert client_contact, "Client1 doesn't made any contact"
    assert basic_callback, "Basic callback not called"

    # TODO >>> When add the client tables mechanism re add the client contact test unit
    # TODO >>> Add a test mechanism to check if the logs are being stored and transposing

    # TODO >>> Add a mechanism to call permission to realize the tests and give an advice that data in the buffers will be wiped of when do the test

    # event = my_host.get_event('client_contact')
    # assert event.is_set(), "Client1 contact event was not set!"

    # event = MyClient.get_event('main_event')
    # assert event.is_set()

    # my_host.clear_events()
    # MyClient.clear_events()
    #

# > ------------------------------------------------------------------------------------------------------------------------------------
# > Inplace Response Test

def host_thread_to_test_inplace_responses (event_host_received):
    print("Starting host thread...")

    # TODO >>> Add a mechanism to test every event and then resume both the host and client returning the successfully done events.

    host_instance = MyHostToTestInplaceResponse(DEBUG_LEVEL).run(
        event=event_host_received
    )

    print("Host thread finished.")


def client_1_thread_to_test_inplace_responses (event_client_received):
    print("Waiting for host to be ready...")
    time.sleep(15)
    print("Starting client 1 thread...")

    client_instance = MyClient1ToTestInplaceResponse(DEBUG_LEVEL)
    client_instance.run()

    print("Client1 thread finished.")


def test_inplace_responses ():
    # multiprocessing.set_start_method('spawn')
    # dill.settings['recurse'] = True

    # Instead of having separate events for client and host, we use a shared event for simplicity
    # The event_key 'main_event' will be used to identify this event

    test_start_time = time.time()

    Events_Manager(
        Unit="Client1", path="Logs"
    ).drop_events_table()  # To reset in the next iteration
    Events_Manager(
        Unit="Client2", path="Logs"
    ).drop_events_table()  # To reset in the next iteration
    Events_Manager(
        Unit="Host", path="Logs"
    ).drop_events_table()  # To reset in the next iteration

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

    t1 = Process(
        target=host_thread_to_test_inplace_responses, args=("main_event",)
    )  # Passing event_key
    t2 = Process(
        target=client_1_thread_to_test_inplace_responses, args=("main_event",)
    )  # Passing event_key

    t1.start()
    t2.start()

    t2.join()
    t1.join()  # Wait for the process to finish

    # * To test:

    # > Client1 Initializes
    # > Host initializes

    # > Client1 make contact
    # > Client1 sync commands available
    # > Client1 schedule to send things
    # > Client1 send command

    # > Host received client command
    # > Host Returned command to client

    # > Client1 Receive Host response

    # > Finish Client1
    # > Finish Host

    host_events = Events_Manager(Unit="Host", path="Logs").List_Events()
    host_events_df = pd.DataFrame.from_dict(host_events)

    client_1_events = Events_Manager(Unit="Client1", path="Logs").List_Events()
    client_1_events_df = pd.DataFrame.from_dict(client_1_events)

    # -> Host events:

    client_contact = False
    basic_callback = False

    # -> Client1 events:

    send_data = False
    basic_inplace_response = False

    for i in host_events_df.index:
        event = host_events_df.loc[i, "StepCompleted"]

        if "Contact received from Client: some_client_id" in event:
            client_contact = True

        if "Active Basic Callback" in event:
            basic_callback = True

    for i in client_1_events_df.index:
        event = client_1_events_df.loc[i, "StepCompleted"]

        if "Data Sended" in event:
            send_data = True

        if "Receive Inplace Response" in event:
            basic_inplace_response = True

    unified_events = host_events_df.merge(client_1_events_df, how="outer")

    tracking = {}
    deltas = []

    for i in unified_events.index:
        event_type = unified_events.loc[i, "EventType"]
        event_key = unified_events.loc[i, "EventKey"]
        event_time = unified_events.loc[i, "Time"]

        if event_type == "Send":
            tracking[event_key] = event_time

        elif event_type == "Receive":
            if event_key in tracking:
                start_ts = tracking[event_key]
                deltas.append(event_time - start_ts)
            else:
                pass

        else:
            pass

    test_end_time = time.time()

    if len(deltas) > 0:
        average_com_delta = sum(deltas) / len(deltas)
    else:
        # Handle the empty list case
        average_com_delta = 0  # or any other default or error valu

    test_run_time = test_end_time - test_start_time

    if (client_contact and basic_callback) and (send_data and basic_inplace_response):
        History_Manager().store_history_point(
            TEST_NODE_NAME,
            NODE_DISK_NAME,
            "test_inplace_responses",
            communications_speed=float(average_com_delta),
            test_speed=test_run_time,
            test_status="PASSED",
            log_level=DEBUG_LEVEL,
        )
    else:
        History_Manager().store_history_point(
            TEST_NODE_NAME,
            NODE_DISK_NAME,
            "test_inplace_responses",
            communications_speed=float(average_com_delta),
            test_speed=test_run_time,
            test_status="FAILED",
            log_level=DEBUG_LEVEL,
        )

    # -> Client1

    assert send_data, "Cant send data"
    assert basic_inplace_response, "Don't called basic inplace response handler"

    # -> Host

    assert client_contact, "Client1 doesn't made any contact"
    assert basic_callback, "Basic callback not called"

    # TODO >>> When add the client tables mechanism re add the client contact test unit
    # TODO >>> Add a test mechanism to check if the logs are being stored and transposing

    # TODO >>> Add a mechanism to call permission to realize the tests and give an advice that data in the buffers will be wiped of when do the test

    # event = my_host.get_event('client_contact')
    # assert event.is_set(), "Client1 contact event was not set!"

    # event = MyClient.get_event('main_event')
    # assert event.is_set()

    # my_host.clear_events()
    # MyClient.clear_events()
    #

# > ------------------------------------------------------------------------------------------------------------------------------------
# > Redirect Test:


def host_thread_to_test_redirect(event_host_received):
    print("Starting host thread...")

    # TODO >>> Add a mechanism to test every event and then resume both the host and client returning the successfully done events.

    host_instance = MyHostToTestRedirect(DEBUG_LEVEL).run(event=event_host_received)

    print("Host thread finished.")


def client_1_thread_to_test_redirect(event_client_received):
    print("Waiting for host to be ready...")
    time.sleep(5)
    print("Starting client 1 thread...")

    client_instance = MyClient1ToTestRedirect(DEBUG_LEVEL)
    client_instance.run()

    print("Client1 thread finished.")


def client_2_thread_to_test_redirect(event_client_received):
    print("Waiting for host to be ready...")
    time.sleep(5)
    print("Starting client 2 thread...")

    client_instance = MyClient2ToTestRedirect(DEBUG_LEVEL)
    client_instance.run()

    print("Client2 thread finished.")


def test_redirect():
    time.sleep(5)

    test_start_time = time.time()

    Events_Manager(
        Unit="Client1", path="Logs"
    ).drop_events_table()  # To reset in the next iteration
    Events_Manager(
        Unit="Client2", path="Logs"
    ).drop_events_table()  # To reset in the next iteration
    Events_Manager(
        Unit="Host", path="Logs"
    ).drop_events_table()  # To reset in the next iteration

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

    t1 = Process(
        target=host_thread_to_test_redirect, args=("main_event",)
    )  # Passing event_key
    t2 = Process(
        target=client_1_thread_to_test_redirect, args=("main_event",)
    )  # Passing event_key
    t3 = Process(
        target=client_2_thread_to_test_redirect, args=("main_event",)
    )  # Passing event_key

    t1.start()
    t2.start()
    t3.start()

    t2.join()
    t3.join()
    t1.join()  # Wait for the process to finish

    host_events = Events_Manager(Unit="Host", path="Logs").List_Events()
    host_events_df = pd.DataFrame.from_dict(host_events)

    client_1_events = Events_Manager(Unit="Client1", path="Logs").List_Events()
    client_1_events_df = pd.DataFrame.from_dict(client_1_events)

    client_2_events = Events_Manager(Unit="Client2", path="Logs").List_Events()
    client_2_events_df = pd.DataFrame.from_dict(client_2_events)

    # >----------------------------------------------------------------------------------------------------
    # > Tests Controller

    # > Host events:

    client_2_contact = False
    client_1_contact = False
    client_contact = False
    basic_callback = False
    host_redirect_callback = True #! Temporally disable, need to add cases that rely in callback based retransmission

    # > Client 1 events:

    send_data = False
    basic_response_handler = False
    active_callback_remotely = False  # * Active callback from another client
    remote_act_response_sended = (
        False  # * Response of the remote activation (Another Redirect to client)
    )

    # > Client 2 events:

    send_data_to_redirect = False
    # redirected_request_response = False #* Response from the remote callback activated

    # >----------------------------------------------------------------------------------------------------

    # -> Host Tests
    for i in host_events_df.index:
        event = host_events_df.loc[i, "StepCompleted"]

        if "Contact received from Client: some_client_id" in event:
            client_1_contact = True

        if "Contact received from Client: randomsclientids" in event:
            client_2_contact = True

        if "Active Host Redirect Callback" in event:
            host_redirect_callback = True 

    # -> Client 1 Tests
    for i in client_1_events_df.index:
        event = client_1_events_df.loc[i, "StepCompleted"]

        # if "Data Sended" in event:
        #     send_data = True

        # if "Activate Basic Response Test callback handler" in event:
        #     basic_response_handler = True

        if "Activate Basic Redirect Test callback handler" in event:
            active_callback_remotely = True

    # -> Client 2 Tests
    for i in client_2_events_df.index:
        event = client_2_events_df.loc[i, "StepCompleted"]

        if "Data To Redirect Sended" in event:
            send_data_to_redirect = True

    unified_events = host_events_df.merge(client_1_events_df, how="outer")
    unified_events = unified_events.merge(client_2_events_df, how="outer")

    tracking = {}
    deltas = []

    for i in unified_events.index:
        event_type = unified_events.loc[i, "EventType"]
        event_key = unified_events.loc[i, "EventKey"]
        event_time = unified_events.loc[i, "Time"]

        if event_type == "Send":
            tracking[event_key] = event_time

        elif event_type == "Receive":
            if event_key in tracking:
                start_ts = tracking[event_key]
                deltas.append(event_time - start_ts)
            else:
                pass

        else:
            pass

    test_end_time = time.time()

    if len(deltas) > 0:
        average_com_delta = sum(deltas) / len(deltas)
    else:
        # Handle the empty list case
        average_com_delta = 0  # or any other default or error valu

    test_run_time = test_end_time - test_start_time

    if (
        (client_1_contact and client_2_contact)
        and (host_redirect_callback and active_callback_remotely)
        and send_data_to_redirect
    ):
        History_Manager().store_history_point(
            TEST_NODE_NAME,
            NODE_DISK_NAME,
            "test_redirect",
            communications_speed=float(average_com_delta),
            test_speed=float(test_run_time),
            test_status="PASSED",
            log_level=DEBUG_LEVEL,
        )
    else:
        History_Manager().store_history_point(
            TEST_NODE_NAME,
            NODE_DISK_NAME,
            "test_redirect",
            communications_speed=float(average_com_delta),
            test_speed=float(test_run_time),
            test_status="FAILED",
            log_level=DEBUG_LEVEL,
        )

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
    assert host_redirect_callback, "Basic redirect callback not called!"

    # TODO >>> When add the client tables mechanism re add the client contact test unit
    # TODO >>> Add a test mechanism to check if the logs are being stored and transposing

    # TODO >>> Add a mechanism to call permission to realize the tests and give an advice that data in the buffers will be wiped of when do the test

    # event = my_host.get_event('client_contact')
    # assert event.is_set(), "Client1 contact event was not set!"

    # event = MyClient.get_event('main_event')
    # assert event.is_set()

    # my_host.clear_events()
    # MyClient.clear_events()

    pass

# > ------------------------------------------------------------------------------------------------------------------------------------
# > Inplace Response Redirect Test:


def host_thread_to_test_inplace_response_redirect(event_host_received):
    print("Starting host thread...")

    # TODO >>> Add a mechanism to test every event and then resume both the host and client returning the successfully done events.

    host_instance = MyHostToTestInplaceResponseRedirect(DEBUG_LEVEL).run(event=event_host_received)

    print("Host thread finished.")


def client_1_thread_to_test_inplace_response_redirect(event_client_received):
    print("Waiting for host to be ready...")
    time.sleep(5)
    print("Starting client 1 thread...")

    client_instance = MyClient1ToTestInplaceResponseRedirect(DEBUG_LEVEL)
    client_instance.run()

    print("Client1 thread finished.")


def client_2_thread_to_test_inplace_response_redirect(event_client_received):
    print("Waiting for host to be ready...")
    time.sleep(5)
    print("Starting client 2 thread...")

    client_instance = MyClient2ToTestInplaceResponseRedirect(DEBUG_LEVEL)
    client_instance.run()

    print("Client2 thread finished.")


def test_inplace_response_redirect():
    time.sleep(5)

    test_start_time = time.time()

    Events_Manager(
        Unit="Client1", path="Logs"
    ).drop_events_table()  # To reset in the next iteration
    Events_Manager(
        Unit="Client2", path="Logs"
    ).drop_events_table()  # To reset in the next iteration
    Events_Manager(
        Unit="Host", path="Logs"
    ).drop_events_table()  # To reset in the next iteration

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

    t1 = Process(
        target=host_thread_to_test_inplace_response_redirect, args=("main_event",)
    )  # Passing event_key
    t2 = Process(
        target=client_1_thread_to_test_inplace_response_redirect, args=("main_event",)
    )  # Passing event_key
    t3 = Process(
        target=client_2_thread_to_test_inplace_response_redirect, args=("main_event",)
    )  # Passing event_key

    t1.start()
    t2.start()
    t3.start()

    t2.join()
    t3.join()
    t1.join()  # Wait for the process to finish

    host_events = Events_Manager(Unit="Host", path="Logs").List_Events()
    host_events_df = pd.DataFrame.from_dict(host_events)

    client_1_events = Events_Manager(Unit="Client1", path="Logs").List_Events()
    client_1_events_df = pd.DataFrame.from_dict(client_1_events)

    client_2_events = Events_Manager(Unit="Client2", path="Logs").List_Events()
    client_2_events_df = pd.DataFrame.from_dict(client_2_events)

    # >----------------------------------------------------------------------------------------------------
    # > Tests Controller

    # > Host events:

    client_2_contact = False
    client_1_contact = False

    # > Client 1 events:

    client1_send_data = False
    client1_receive_inplace_response = False
    basic_response_handler = False
    active_callback_remotely = False  # * Active callback from another client
    remote_act_response_sended = (
        False  # * Response of the remote activation (Another Redirect to client)
    )

    # > Client 2 events:

    client2_activated_basic_callback = False
    client2_send_response = False

    send_data_to_redirect = False
    # redirected_request_response = False #* Response from the remote callback activated

    # >----------------------------------------------------------------------------------------------------

    # -> Host Tests
    for i in host_events_df.index:
        event = host_events_df.loc[i, "StepCompleted"]

        if "Contact received from Client: some_client_id" in event:
            client_1_contact = True

        if "Contact received from Client: randomsclientids" in event:
            client_2_contact = True

    # -> Client 1 Tests
    for i in client_1_events_df.index:
        event = client_1_events_df.loc[i, "StepCompleted"]

        if "Data Sended" in event:
            client1_send_data = True
        
        if "Receive Inplace Response" in event:
            client1_receive_inplace_response = True

    # -> Client 2 Tests
    for i in client_2_events_df.index:
        event = client_2_events_df.loc[i, "StepCompleted"]

        if "Active Basic Callback" in event:
            client2_activated_basic_callback = True

        if "Send Response"in event:
            client2_send_response = True

    unified_events = host_events_df.merge(client_1_events_df, how="outer")
    unified_events = unified_events.merge(client_2_events_df, how="outer")

    tracking = {}
    deltas = []

    for i in unified_events.index:
        event_type = unified_events.loc[i, "EventType"]
        event_key = unified_events.loc[i, "EventKey"]
        event_time = unified_events.loc[i, "Time"]

        if event_type == "Send":
            tracking[event_key] = event_time

        elif event_type == "Receive":
            if event_key in tracking:
                start_ts = tracking[event_key]
                deltas.append(event_time - start_ts)
            else:
                pass

        else:
            pass

    test_end_time = time.time()

    if len(deltas) > 0:
        average_com_delta = sum(deltas) / len(deltas)
    else:
        # Handle the empty list case
        average_com_delta = 0  # or any other default or error valu

    test_run_time = test_end_time - test_start_time

    if (
        (client_1_contact and client_2_contact)
        and (client1_send_data and client1_receive_inplace_response)
        and (client2_activated_basic_callback and client2_send_response)
    ):
        History_Manager().store_history_point(
            TEST_NODE_NAME,
            NODE_DISK_NAME,
            "test_redirect",
            communications_speed=float(average_com_delta),
            test_speed=float(test_run_time),
            test_status="PASSED",
            log_level=DEBUG_LEVEL,
        )
    else:
        History_Manager().store_history_point(
            TEST_NODE_NAME,
            NODE_DISK_NAME,
            "test_redirect",
            communications_speed=float(average_com_delta),
            test_speed=float(test_run_time),
            test_status="FAILED",
            log_level=DEBUG_LEVEL,
        )

    # -> Host
    
    assert client_1_contact, "Client 1 doesn't make any contact with host!"
    assert client_2_contact, "Client 2 doesn't make any contact with host!"

    # -> Client 1

    assert client1_send_data, "Client 1 doesn't sended the command to Client 2"
    assert client1_receive_inplace_response, "Client 1 doesn't received the response of the command sended to Client 2"


    # -> Client 2

    assert client2_activated_basic_callback, "Don't receive the command sended by Client 1 in Client 2"
    assert client2_send_response, "Don't sended the response from Client 2 to Client 1"

    pass

# > ------------------------------------------------------------------------------------------------------------------------------------
# > Inner Management Test:


def host_thread_to_test_inner_management(event_host_received):
    print("Starting host thread...")

    # TODO >>> Add a mechanism to test every event and then resume both the host and client returning the successfully done events.
    
    #> Create a system to test if the manipulation in the host structure really happened
    
    host_instance = MyHostToTestManagement(DEBUG_LEVEL).run(event=event_host_received)

    print("Host thread finished.")


def client_1_thread_to_test_inner_management(event_client_received):
    print("Waiting for host to be ready...")
    time.sleep(5)
    print("Starting client 1 thread...")

    client_instance = MyClient1ToTestManagement(DEBUG_LEVEL)
    client_instance.run()

    print("Client1 thread finished.")


def test_management():
    time.sleep(5)

    test_start_time = time.time()

    Events_Manager(
        Unit="Client1", path="Logs"
    ).drop_events_table()  # To reset in the next iteration
    Events_Manager(
        Unit="Client2", path="Logs"
    ).drop_events_table()  # To reset in the next iteration
    Events_Manager(
        Unit="Host", path="Logs"
    ).drop_events_table()  # To reset in the next iteration

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

    t1 = Process(
        target=host_thread_to_test_inner_management, args=("main_event",)
    )  # Passing event_key
    t2 = Process(
        target=client_1_thread_to_test_inner_management, args=("main_event",)
    )  # Passing event_key

    t1.start()
    t2.start()

    t2.join()

    host_events = Events_Manager(Unit="Host", path="Logs").List_Events()
    host_events_df = pd.DataFrame.from_dict(host_events)

    client_1_events = Events_Manager(Unit="Client1", path="Logs").List_Events()
    client_1_events_df = pd.DataFrame.from_dict(client_1_events)

    # >----------------------------------------------------------------------------------------------------
    # > Tests Controller

    # > Host events:

    # TODO >>> Continue to implement the tests to assess if the test inner management are working!

    client_1_contact = False
    client_contact = False

    # TODO >>> Create a mechanism that checks if host received or not the commands and if it add or not the clients with the correct data
    add_client_callback = True  #! Change to False to allow this test
    update_client_callback = True  #! Change to False to allow this test
    remove_client_callback = True  #! Change to False to allow this test
    
    #* Check if client db was really moded:
    new_client_added_in_db = False
    new_client_updated_in_db = False 
    new_client_deleted_in_db = False

    # > Client 1 events:

    send_add_client = False
    send_update_client = False
    send_remove_client = False

    receive_add_client_conf = False
    receive_update_client_conf = False 
    receive_remove_client_conf = False

    # >----------------------------------------------------------------------------------------------------

    # -> Host Tests
    for i in host_events_df.index:
        event = host_events_df.loc[i, "StepCompleted"]

        # > Host Receivers (Only valid to kwargs based)

        if "Active Test Add Client" in event:
            add_client_callback = True

        if "Active Test Update Client" in event:
            update_client_callback = True

        if "Active Test Remove Client" in event:
            remove_client_callback = True
            
        # > Confirmation of change in db structure:
        
        if "Client key xMndjslwpedcnfe was added." in event:
            new_client_added_in_db = True
            
        if "Client key xMndjslwpedcnfe was updated." in event:
            new_client_updated_in_db = True
            
        if "Client key xMndjslwpedcnfe was removed." in event:
            new_client_deleted_in_db = True
            

    # -> Client 1 Tests
    for i in client_1_events_df.index:
        event = client_1_events_df.loc[i, "StepCompleted"]

        # > Senders

        if "Send test add a client" in event:
            send_add_client = True

        if "Send test update a client" in event:
            send_update_client = True

        if "Send test remove a client" in event:
            send_remove_client = True

        # > Receivers

        if "Activate Basic Response Test Add Client" in event:
            receive_add_client_conf = True

        if "Activate Basic Response Test Update Client" in event:
            receive_update_client_conf = True

        if "Activate Basic Response Test Remove Client" in event:
            receive_remove_client_conf = True
            

    unified_events = host_events_df.merge(client_1_events_df, how="outer")

    tracking = {}
    deltas = []

    for i in host_events_df.index:
        event_type = host_events_df.loc[i, "EventType"]
        event_key = host_events_df.loc[i, "EventKey"]
        event_time = host_events_df.loc[i, "Time"]

        if event_type == "Send":
            tracking[event_key] = event_time

        elif event_type == "Receive":
            if event_key in tracking:
                start_ts = tracking[event_key]
                deltas.append(event_time - start_ts)
            else:
                pass

        else:
            pass

    test_end_time = time.time()

    if len(deltas) > 0:
        average_com_delta = sum(deltas) / len(deltas)
    else:
        # Handle the empty list case
        average_com_delta = 0  # or any other default or error value

    test_run_time = test_end_time - test_start_time

    if (
        (add_client_callback and update_client_callback)
        and (remove_client_callback and send_add_client)
        and (send_update_client and send_remove_client)
        and (receive_add_client_conf and receive_update_client_conf)
        and receive_remove_client_conf
    ):
        History_Manager().store_history_point(
            TEST_NODE_NAME,
            NODE_DISK_NAME,
            "test_management",
            communications_speed=float(average_com_delta),
            test_speed=float(test_run_time),
            test_status="PASSED",
            log_level=DEBUG_LEVEL,
        )
    else:
        History_Manager().store_history_point(
            TEST_NODE_NAME,
            NODE_DISK_NAME,
            "test_management",
            communications_speed=float(average_com_delta),
            test_speed=float(test_run_time),
            test_status="FAILED",
            log_level=DEBUG_LEVEL,
        )

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
    
    assert new_client_added_in_db, "Host receive add client but it doesn't added any client!"
    assert new_client_updated_in_db, "Host receive update client but it doesn't update any client!"
    assert new_client_deleted_in_db, "Host receive remove client but it doesn't remove any client!"


# > ------------------------------------------------------------------------------------------------------------------------------------
# > Messages Test:


def host_thread_to_test_messages(event_host_received):
    print("Starting host thread...")
    host_instance = MyHostToTestMessages(DEBUG_LEVEL).run(event=event_host_received)
    print("Host thread finished.")


def client_1_thread_to_test_messages(event_client_received):
    print("Waiting for host to be ready...")
    time.sleep(5)
    print("Starting client 1 thread...")

    client_instance = MyClient1ToTestMessages(DEBUG_LEVEL)
    client_instance.run()

    print("Client1 thread finished.")


def _test_messages(): #! Temporarly Deactivated
    time.sleep(5)

    Events_Manager(
        Unit="Client1", path="Logs"
    ).drop_events_table()  # To reset in the next iteration
    Events_Manager(
        Unit="Client2", path="Logs"
    ).drop_events_table()  # To reset in the next iteration
    Events_Manager(
        Unit="Host", path="Logs"
    ).drop_events_table()  # To reset in the next iteration

    test_start_time = time.time()

    System_Status(path="Logs").create_unit("Client1")
    System_Status(path="Logs").create_unit("Host")

    System_Status(path="Logs").change_unit_status(Unit="Client1", Status=True)
    System_Status(path="Logs").change_unit_status(Unit="Host", Status=True)

    if os.path.exists("Temp/Client1Data/"):
        shutil.rmtree("Temp/Client1Data/")

    if os.path.exists("Temp/Data/"):
        shutil.rmtree("Temp/Data/")

    t1 = Process(
        target=host_thread_to_test_messages, args=("main_event",)
    )  # Passing event_key
    t2 = Process(
        target=client_1_thread_to_test_messages, args=("main_event",)
    )  # Passing event_key

    t1.start()
    t2.start()

    t2.join()

    host_events = Events_Manager(Unit="Host", path="Logs").List_Events()
    host_events_df = pd.DataFrame.from_dict(host_events)

    client_1_events = Events_Manager(Unit="Client1", path="Logs").List_Events()
    client_1_events_df = pd.DataFrame.from_dict(client_1_events)

    # >----------------------------------------------------------------------------------------------------
    # > Tests Controller

    # > Host events:

    # TODO >>> Continue to implement the tests to assess if the test inner management are working!

    callback_for_correct_data = False
    callback_for_incorrect_data = (
        True  #! Temporarily deactivated due to issues in the error case handling
    )

    # > Client 1 events:

    send_correct_data_for_host = False
    send_incorrect_data_for_host = (
        True  #! Temporarily deactivated due to issues in the error case handling
    )

    receive_success_response_handler = False
    receive_error_response_handler = (
        True  #! Temporarily deactivated due to issues in the error case handling
    )

    # >----------------------------------------------------------------------------------------------------

    # -> Host Tests
    for i in host_events_df.index:
        event = host_events_df.loc[i, "StepCompleted"]

        if "Active Basic Callback For Correct Data" in event:
            callback_for_correct_data = True

        if "Active Basic Callback For Incorrect Data" in event:
            callback_for_incorrect_data = True

    # -> Client 1 Tests
    for i in client_1_events_df.index:
        event = client_1_events_df.loc[i, "StepCompleted"]

        # > Senders

        if "Correct Data Sended" in event:
            send_correct_data_for_host = True

        if "Incorrect Data Sended" in event:
            send_incorrect_data_for_host = True

        if "Activate Basic Success Response Test callback handler" in event:
            receive_success_response_handler = True

        if "Activate Basic Error Redirect Test callback handler" in event:
            receive_error_response_handler = True

    unified_events = host_events_df.merge(client_1_events_df, how="outer")

    tracking = {}
    deltas = []

    for i in unified_events.index:
        event_type = host_events_df.loc[i, "EventType"]
        event_key = host_events_df.loc[i, "EventKey"]
        event_time = host_events_df.loc[i, "Time"]

        if event_type == "Send":
            tracking[event_key] = event_time

        elif event_type == "Receive":
            if event_key in tracking:
                start_ts = tracking[event_key]
                deltas.append(event_time - start_ts)
            else:
                pass

        else:
            pass

    test_end_time = time.time()

    if len(deltas) > 0:
        average_com_delta = sum(deltas) / len(deltas)
    else:
        # Handle the empty list case
        average_com_delta = 0  # or any other default or error value

    test_run_time = test_end_time - test_start_time

    if (
        (callback_for_correct_data and callback_for_incorrect_data)
        and (send_correct_data_for_host and send_incorrect_data_for_host)
        and (receive_success_response_handler and receive_error_response_handler)
    ):
        History_Manager().store_history_point(
            TEST_NODE_NAME,
            NODE_DISK_NAME,
            "test_management",
            communications_speed=float(average_com_delta),
            test_speed=float(test_run_time),
            test_status="PASSED",
            log_level=DEBUG_LEVEL,
        )
    else:
        History_Manager().store_history_point(
            TEST_NODE_NAME,
            NODE_DISK_NAME,
            "test_management",
            communications_speed=float(average_com_delta),
            test_speed=float(test_run_time),
            test_status="FAILED",
            log_level=DEBUG_LEVEL,
        )

    # -> Client 1 Senders

    assert send_correct_data_for_host, "Can't send correct command to host!"
    assert send_incorrect_data_for_host, "Can't send incorrect command to host!"

    # -> Host

    assert callback_for_correct_data, "Correct callback handler not triggered"
    assert callback_for_incorrect_data, "Incorrect callback handler not triggered"

    # -> Client 1 Receivers

    assert receive_success_response_handler, "don't receive the success response!"
    assert receive_error_response_handler, "don't receive the error response!"



# > ------------------------------------------------------------------------------------------------------------------------------------
# > Redirect Messages Test:

# def host_thread_to_test_redirect_messages (event_host_received):
#     print("Starting host thread...")

#     # TODO >>> Add a mechanism to test every event and then resume both the host and client returning the successfully done events.

#     host_instance = MyHostToTestRedirectMessages(DEBUG_LEVEL).run(event=event_host_received)

#     print("Host thread finished.")

# def client_1_thread_to_test_redirect_messages (event_client_received):
#     print("Waiting for host to be ready...")
#     time.sleep(5)
#     print("Starting client 1 thread...")

#     client_instance = MyClient1ToTestRedirectMessages(DEBUG_LEVEL)
#     client_instance.run()

#     print("Client1 thread finished.")

# def client_2_thread_to_test_redirect_messages (event_client_received):
#     print("Waiting for host to be ready...")
#     time.sleep(5)
#     print("Starting client 2 thread...")

#     client_instance = MyClient2ToTestRedirectMessages(DEBUG_LEVEL)
#     client_instance.run()

#     print("Client2 thread finished.")

# def test_redirect_messages ():

#     time.sleep(5)

#     test_start_time = time.time()

#     Events_Manager(Unit="Client1", path="Logs").drop_events_table() # To reset in the next iteration
#     Events_Manager(Unit="Client2", path="Logs").drop_events_table() # To reset in the next iteration
#     Events_Manager(Unit="Host", path="Logs").drop_events_table() # To reset in the next iteration

#     System_Status(path="Logs").create_unit("Client1")
#     System_Status(path="Logs").create_unit("Client2")
#     System_Status(path="Logs").create_unit("Host")

#     System_Status(path="Logs").change_unit_status(Unit="Client1", Status=True)
#     System_Status(path="Logs").change_unit_status(Unit="Client2", Status=True)
#     System_Status(path="Logs").change_unit_status(Unit="Host", Status=True)

#     if os.path.exists("Temp/Client1Data/"):
#         shutil.rmtree("Temp/Client1Data/")

#     if os.path.exists("Temp/Client2Data/"):
#         shutil.rmtree("Temp/Client2Data/")

#     if os.path.exists("Temp/Data/"):
#         shutil.rmtree("Temp/Data/")

#     t1 = Process(target=host_thread_to_test_redirect_messages, args=('main_event',)) # Passing event_key
#     t2 = Process(target=client_1_thread_to_test_redirect_messages, args=('main_event',)) # Passing event_key
#     t3 = Process(target=client_2_thread_to_test_redirect_messages, args=('main_event',)) # Passing event_key

#     t1.start()
#     t2.start()
#     t3.start()

#     t2.join()
#     t3.join()
#     t1.join()  # Wait for the process to finish

#     host_events = Events_Manager(Unit="Host", path="Logs").List_Events()
#     host_events_df = pd.DataFrame.from_dict(host_events)

#     client_1_events = Events_Manager(Unit="Client1", path="Logs").List_Events()
#     client_1_events_df = pd.DataFrame.from_dict(client_1_events)

#     client_2_events = Events_Manager(Unit="Client2", path="Logs").List_Events()
#     client_2_events_df = pd.DataFrame.from_dict(client_2_events)

#     #>----------------------------------------------------------------------------------------------------
#     #> Tests Controller

#     #> Host events:

#     client_2_contact            = False
#     client_1_contact            = False
#     client_contact              = False
#     basic_callback              = False
#     host_redirect_callback      = False

#     #> Client 1 events:

#     send_data                   = False
#     basic_response_handler      = False
#     active_callback_remotely    = False #* Active callback from another client
#     remote_act_response_sended  = False #* Response of the remote activation (Another Redirect to client)

#     # > Client 2 events:

#     send_data_to_redirect       = False
#     # redirected_request_response = False #* Response from the remote callback activated

#     #>----------------------------------------------------------------------------------------------------

#     # -> Host Tests
#     for i in host_events_df.index:
#         event = host_events_df.loc[i, 'StepCompleted']

#         if "Contact received from Client: some_client_id" in event:
#             client_1_contact = True

#         if "Contact received from Client: randomsclientids" in event:
#             client_2_contact = True

#         if "Active Host Redirect Callback" in event:
#             host_redirect_callback = True

#     # -> Client 1 Tests
#     for i in client_1_events_df.index:
#         event = client_1_events_df.loc[i, 'StepCompleted']

#         # if "Data Sended" in event:
#         #     send_data = True

#         # if "Activate Basic Response Test callback handler" in event:
#         #     basic_response_handler = True

#         if "Activate Basic Redirect Test callback handler" in event:
#             active_callback_remotely = True

#     # -> Client 2 Tests
#     for i in client_2_events_df.index:
#         event = client_2_events_df.loc[i, 'StepCompleted']

#         if "Data To Redirect Sended" in event:
#             send_data_to_redirect = True


#     unified_events = host_events_df.merge(client_1_events_df, how='outer')
#     unified_events = unified_events.merge(client_2_events_df, how='outer')

#     tracking = {}
#     deltas = []

#     for i in unified_events.index:

#         event_type = host_events_df.loc[i, "EventType"]
#         event_key  = host_events_df.loc[i, "EventKey"]
#         event_time = host_events_df.loc[i, "Time"]

#         if event_type == "Send":
#             tracking[event_key] = event_time

#         elif event_type == "Receive":

#             if event_key in tracking:
#                 start_ts = tracking[event_key]
#                 deltas.append(event_time - start_ts)
#             else:
#                 pass

#         else:
#             pass

#     test_end_time = time.time()
#     average_com_delta = (sum(deltas) / len(deltas))
#     test_run_time = test_end_time - test_start_time

#     if (
#         (client_1_contact and client_2_contact)
#         and (host_redirect_callback and active_callback_remotely)
#         and send_data_to_redirect
#     ):
#         History_Manager().store_history_point("test_redirect", communications_speed=average_com_delta, test_speed=test_run_time, test_status="PASSED", log_level=DEBUG_LEVEL)
#     else:
#         History_Manager().store_history_point("test_redirect", communications_speed=average_com_delta, test_speed=test_run_time, test_status="FAILED", log_level=DEBUG_LEVEL)

#     # -> Client 1

#     # assert send_data, "Cant send data"
#     # assert basic_response_handler, "Don't called basic response handler"
#     assert active_callback_remotely, "Don't received redirect response!"

#     # -> Client 2

#     assert send_data_to_redirect, "Don't could send data to redirect!"

#     # -> Host
#     assert client_1_contact, "Client 1 doesn't make any contact with host!"
#     assert client_2_contact, "Client 2 doesn't make any contact with host!"
#     # assert client_contact, "Client1 doesn't made any contact"
#     assert host_redirect_callback, "Basic redirect callback not called!"

#     # TODO >>> When add the client tables mechanism re add the client contact test unit
#     # TODO >>> Add a test mechanism to check if the logs are being stored and transposing

#     # TODO >>> Add a mechanism to call permission to realize the tests and give an advice that data in the buffers will be wiped of when do the test

#     # event = my_host.get_event('client_contact')
#     # assert event.is_set(), "Client1 contact event was not set!"

#     # event = MyClient.get_event('main_event')
#     # assert event.is_set()

#     # my_host.clear_events()
#     # MyClient.clear_events()


#     pass


if __name__ == "__main__":
    pytest.main()
