# SPDX-License-Identifier: MPL-2.0
# Copyright © 2021-2026 Cristian Camargo Filho

# conftest.py

import json
import pytest
import os
# from ollama import Client

# This fixture will be automatically used by all tests.
@pytest.fixture(autouse=True)
def run_around_tests():
    # Code before each test (optional)
    yield
    # Code after each test (optional)

# Define the root directory where the Temp folders are located
TEMP_DIR = os.path.join(os.path.dirname(__file__), "Temp")

# Define a global variable to hold all log entries
ALL_LOG_ENTRIES = {}

@pytest.fixture(scope="session", autouse=True)
def collect_logs():
    global ALL_LOG_ENTRIES
    
    def get_last_directory(path):
        # Strip the trailing slash if it exists
        path = path.rstrip(os.path.sep)

        # Check if the last component of the path is a directory
        if os.path.isdir(path):
            return os.path.basename(path)
        else:
            # If the last component is a file, get the directory name
            return os.path.basename(os.path.dirname(path))
    
    all_logs_dict = {}
    
    for root, dirs, files in os.walk(TEMP_DIR):
        for file in files:
            if file == "logs.txt":
                with open(os.path.join(root, file), 'r') as log_file:
                    
                    last_dir_name = get_last_directory(root)
                    
                    logs = []
                    
                    for line in log_file:
                        try:
                            entry = json.loads(line.strip())
                            logs.append(entry)
                        except json.JSONDecodeError as e:
                            print(f"Error parsing JSON: {e}, in file: {file}, line: {line}")
                            
                    if last_dir_name == "Data":    
                        all_logs_dict["HOST"] = logs
                    if last_dir_name == "Client1Data":    
                        all_logs_dict["CLIENT1"] = logs
                    if last_dir_name == "Client2Data":    
                        all_logs_dict["CLIENT2"] = logs
    
    ALL_LOG_ENTRIES = all_logs_dict                
                            
    # Yield the collected log entries for tests, if needed
    yield ALL_LOG_ENTRIES

# This hook is called after all tests have been executed.
def pytest_sessionfinish(session, exitstatus):
    global ALL_LOG_ENTRIES
    
    warnings = []
    exceptions = []
    
    for logs in ALL_LOG_ENTRIES.values():
        warnings + [entry for entry in logs if entry["log_level"] == "WARNING"]
        exceptions + [entry for entry in logs if entry["log_level"] == "EXCEPTION"]

    print("\nPytest Summary:")
    print(f"✅ pytest {session.testscollected} collected")
    print(f"✅ No Warnings" if not warnings else f"⚠️ Warnings: {len(warnings)}")
    print(f"✅ No Exceptions" if not exceptions else f"🚫 Exceptions: {len(exceptions)}")
    
    print("\n")
    
    # Received: Command { client_key: \"some_client_id\", parity_id: \"itisaspecialcase\", priority: 11, command: CommandInstructions { mode: Response, command_type: SpecialFunction, target: Origin, status: Success, origin: Host, actf: \"C207\", kwargs: {}, message: \"\", response_type: None, response_target: None, response_actf: None, collect_response: true } }
    
    # try:
    #     client = Client(host='http://127.0.0.1:11434')
        
    #     log_lines_dict = {}
    #     for owner, logs in ALL_LOG_ENTRIES.items():
        
    #         for log in logs:
                
    #             if log["log_msg"] == "Nothing in the schedule, skipping >>>":
    #                 continue
            
    #             if log["log_msg"] == "\nSchedule to process:\n[]\n":
    #                 continue
                
    #             # -> Remove C206 Ping and C207 Pong they are not necessary for this task and will overflow the model
    #             if 'C206' not in log["log_msg"] and 'C207' not in log["log_msg"]:
    #                 pass
    #             else:
    #                 continue
     
    #             if log["log_time"] == "":
    #                 continue

    #             log_lines_dict[log["log_time"]] = f"{owner}: " + log["log_msg"] + "\n"

    #     # Sorting the dictionary by its keys (timestamps)
    #     sorted_dict = {k: log_lines_dict[k] for k in sorted(log_lines_dict)}

    #     log_lines = ""
    
    #     for val in sorted_dict.values():
    #         log_lines += val
            
    #     # TODO >>> Maybe implement Crewai here to better analyse the logs

    #     # print(log_lines)
    
    #     print("\n")

    #     response = client.chat(model='mistral', messages=[
    #         {
    #             'role': 'user',
    #             'content': f"Please review the following log messages from a test between a Central Node and adjacent nodes in a distributed system. Your task is to identify any anomalies, errors, or unusual patterns in these log lines, you will encounter seval commands, like C206 and C207 that are ping and pong, and other CXXX commands that are especial commands, also another type of commands and responses too, in middle of that will have some other kind of log messages, the order here matters to understand it! Note that you do not need to provide explanations or solutions for these issues. Simply identify and list any irregularities or notable errors you find in the log data. The log lines are as follows: \n{log_lines} \n Focus on discrepancies, things that looks like errors, or any pattern breaks in these log messages and do not foucus inconsistencies because the logs logs various behaviors that do diferent things and also, don't focus in the timestamp order because they are already are in order! Good job.",
    #         },
    #     ])

    #     message = response["message"]["content"]
        
    #     analise_file = os.path.join(TEMP_DIR, "auto_analise.txt")
        
    #     # Remove old analise file
    #     os.remove(analise_file)
        
    #     with open (analise_file , "w") as file:
    #         file.write(message + "\n")
    #         file.close()
            
    #     print("-="*50)
    #     print(f"AI: {message}")

    # except BaseException as e:
    #     print(f"Error calling the llm for custom analisis, the error is: \n{e}")
    #     pass
    
    print("\n")

    behavior_ok = (exitstatus == 0) and (len(exceptions) == 0)
    print(f"{'✅' if behavior_ok else '🚫'} Behavior {'GOOD' if behavior_ok else 'BAD'}\n")

    # if warnings or exceptions or not behavior_ok:
    #     session.exitstatus = 1  # non-zero exit status indicates failure
        
    # if not behavior_ok:
    #     session.exitstatus = ExitCode.TESTS_FAILED
