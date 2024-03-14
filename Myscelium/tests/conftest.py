# conftest.py

import json
import pytest
import os

# This fixture will be automatically used by all tests.
@pytest.fixture(autouse=True)
def run_around_tests():
    # Code before each test (optional)
    yield
    # Code after each test (optional)

# Define the root directory where the Temp folders are located
TEMP_DIR = os.path.join(os.path.dirname(__file__), "Temp")

# Define a global variable to hold all log entries
ALL_LOG_ENTRIES = []

@pytest.fixture(scope="session", autouse=True)
def collect_logs():
    global ALL_LOG_ENTRIES
    for root, dirs, files in os.walk(TEMP_DIR):
        for file in files:
            if file == "logs.txt":
                with open(os.path.join(root, file), 'r') as log_file:
                    for line in log_file:
                        try:
                            entry = json.loads(line.strip())
                            ALL_LOG_ENTRIES.append(entry)
                        except json.JSONDecodeError as e:
                            print(f"Error parsing JSON: {e}, in file: {file}, line: {line}")
    # Yield the collected log entries for tests, if needed
    yield ALL_LOG_ENTRIES


# This hook is called after all tests have been executed.
def pytest_sessionfinish(session, exitstatus):
    global ALL_LOG_ENTRIES
    warnings = [entry for entry in ALL_LOG_ENTRIES if entry["log_level"] == "WARNING"]
    exceptions = [entry for entry in ALL_LOG_ENTRIES if entry["log_level"] == "EXCEPTION"]

    print("\nPytest Summary:")
    print(f"✅ pytest {session.testscollected} collected")
    print(f"✅ No Warnings" if not warnings else f"⚠️ Warnings: {len(warnings)}")
    print(f"✅ No Exceptions" if not exceptions else f"🚫 Exceptions: {len(exceptions)}")

    behavior_ok = (exitstatus == 0) and (len(exceptions) == 0)
    print(f"✅ Behavior {'GOOD' if behavior_ok else 'BAD'}\n")

    if warnings or exceptions or not behavior_ok:
        session.exitstatus = 1  # non-zero exit status indicates failure
