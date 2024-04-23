
from myscelium import MysceliumClient, ClientPatterns, callback_pattern
import os
import time
import signal

client_patterns = ClientPatterns()

CLIENT_KEY = "some_client_id"

command = client_patterns.command_pattern(
    origin_key=CLIENT_KEY,
    command_function="python_function",
    target_key="",  # Empty is default
    kwargs={"age": 10, "birth": 8, "name": "cristian"},
    message="",
    response_type="ExternalFunction",
    response_target="Origin",
    response_actf="test_handler",
    auto_collect_response=True,
)

# What the base struct should looks like:
base = {
    'mode': 'Function', 
    'type': 'ExternalFunction', 
    'target': 'Host', 
    'status': 'Success', 
    'origin': 'ClientKey(some_client_id)', 
    'actf': 'python_function', 
    'kwargs': {
        'age': 10, 
        'birth': 8, 
        'name': 'cristian'
    }, 
    'message': '', 
    'response_type': 'ExternalFunction', 
    'response_target': 'Origin', 
    'response_actf': 'test_handler', 
    'collect_response': True
}

print(command)