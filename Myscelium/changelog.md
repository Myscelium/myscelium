# Changelog

## v1.3.0 - ReliseCandidate (18/08/2023)

Certainly! Here's how you might integrate the additional updates into the existing ones:

### Updates:

1. **Client and Permissions:**
   - "Modify client allowed by implementing a client table that you can personalize several permissions of client"
   - "Add a mechanism to add new allowed client in flight from mys_host obj"
   - "Add a base to implement permission groups and indexing that to client"

2. **Callbacks and Responses:**
   - "Add a new type of responses to host patterns `ResponseType:InternalMannangement` this allows to change internals of host through callbacks responses"
   - "Why only from callbacks response: ..."
   - "Blocked the reception of InternalMaannangement commands from external sources unless redirected by callbacks."
   - "Updated the response command structure to include a status, message, and kwargs inside a response."
   - "Updated receivers to receive the entire command, including command type, status, response activation function, message, kwargs, and response mode"
   - "Handled the inner management responses in the receiver for both errors and confirmations"
   - Added a response error pattern to host responses `error_response_pattern` patterns.

3. **Improvements and Fixes:**
   - "Improve tests capabilities"
   - "Improve verification of buffer responses"
   - "Make tests of connection more stronger"
   - "Enhance client event transposition threads structure"
   - "Improved debugging in handle internal command functions."
   - "Made improvements to the `new_client.fast_verify_kwargs_and_types` function to check both mismatched types and keys."
   - "Fixed the issue with the Python object extractor not being able to extract lists."
   - "Fixed various errors and issues related to response formulation and command handling."
   - "Fixed buffer bug that caused multiple commands with the same ID to be registered."
   - "Fix verification if client is in whitelist"
   - "Fix removing commands from queue when already receive a response from where we send it to"
   - "Fix wrapper client transposition zombie threads"
   - "Fix client random quitting by rearranging the processes event controllers in python side"
   - "Fix the redirect adding the code to handle this case in host"
   - "Improved logging by changing prints to logging and adding a print in the logging database emitter"

4. **Centralization and Common Functions:**
   - "Centralize common functions reducing code complexity and improve code efficient"
   - "Centralize more python functions in common, like extract_args_type, dict_to tuple and the handle_pyobject that was changed to extract_pyobject"

5. **Tests and Events:**
   - "Start planning a graphical representation of tests"
   - "Implement Test of redirect"
   - "Now test events have 3 categories `Send`, `Receive`, and `Default`"
   - "This config allows to track how much time takes to receive some fn, response or redirect, and consequently allowing to track the performance of the lib in the development process through a permanent database that tracks the event medium time, allowing to do some performance tests."
   - "Added the Historie controller and the history visualizer for host tests"

6. **Internal Management and Commands:**
   - "Added a command type called InternalMaannangement and implemented the necessary changes to handle it."
   - "Blocked the reception of InternalMaannangement commands from external sources unless redirected by callbacks."
   - "Added the InternalMannangement handler in the transposer."
   - "Implemented error patterns to send errors back to the client using the new communication method"
   - "Added confirmation messages and error messages to the inner management"

7. **Type and Verification:**
   - "Add `type_of` and `fast_verify_kwargs_and_types` to `ResultType` enum, this can facilitate the process of checking a type of a `ResultType` and also recursively check if the Result matches with a predefined pattern in a very fast way by providing a reference target to `fast_verify_kwargs_and_types` that can return an empty Ok() when all is fine and return a enum `ExpectationError` when something isn't correct."

8. **Groups Manipulation**:
    - "// TODO - Edit a group from callbacks responses"
    - "// TODO - Add a group from callbacks responses"
    - "// TODO - Remove a group from callbacks response"

### Fixes:

- Now client contact event callback caller doesn't generate random exceptions
- Client permissions doesn't need to be passed every time because now has a permanent database table to hold the client informations
- Tests now are working as intended
- Thread pool has been improved, and now it doesn't bring errors anymore
- Improve several mechanisms, implement new error capabilities, and centralizing for better updates in the future 
- Solve issues in tests of connection


## v1.2.0 - ReliseCandidate (18/08/2023)

### Updates
- Add logs buffer
- Add logs transposition interface attached to wrapper to transpose logs from buffer into some py functions
- Improve Buffers
- Reinforce Code Structure
- Add singleton patterns into wrapper
- Now myscelium host & client can be a class
- Add resistence tests into pytest to test code resistence to multiple runs
- Add Planings to expand
- Centrlaize code to better mainteinability in commom (to ensure fast chanbges)
- Improve code efficiency
- Develop ways to reduce the cicle numbers
- Start preparing to implement multi chanels data stream
- Centralize python callback calling function and python converters
- Centralize type converters
- Create indentificators to diferenciate response callback calling from function callback calling
- Improve the serialization and desserialization mecanisms
- Redirect now is working as intended
- Now pendent comamnds are send to client when client doesn't have nothing to send and send ping, the response will be pendend commands

### Fixes
- Fix random quitting
- Fix py gill aquire
- Fix buffers sql pools
- Fix great part of buffers exception handlers
- Fix wrapper concurrency
- Fix threads
- Fix unity tests
- Fix callbacks calls
- Improve client response handlers callbacks systems
- Fix shutdown
- Fix zombie threds runing without controler when shutdown by exception
- Fix Logger
- Fix buffers random exceptions
- Fix type checking in some key points to ensure the system is working as intended
- Fix callbacks calling to be more correct and detect better the cases were is response handler calling and when is function handler calling
- Fix serialization and desserialization of host responses
- Fix command response decoding
- Fix redirect mecanism
- Fix Test Redirect
- Fix redirect serialization, desserialization and reincoding to redirect


## v1.1.0 - ReliseCandidate (08/08/2023)

### Updates
- Add a automatic pytest to test all the lib making connections from client to host and testisng remote function callbacks activationa and client response handlers
- Improve lib code
- Improve wrappers
- Add better exemples of usage
- Create a way to use classes to run host and client all using Classes


## v1.0.0-Release (27/07/2023)

### Updates & New Features
- Implement several mechanisms to sync client and host.
- Link client to host.
- Create a callback handler for client contact.
- Develop mechanism to schedule sends.
- Implement mechanism to allow client to call callbacks when receiving data.
- Implement redirect mechanisms.
- Implement client directly on the library to facilitate creating clients with patterns.
- Implement a pattern system to facilitate creating command structures.
- Improve loops and pool system for better multi-threading.
- Make the system more robust and add checkpoints for future expansions.

### Error Handling & Fixes
- Add better error messages in the core processes.
- Fix errors related to connection and incorrect function calling.
- Fix issues with argument random mismatching.
- Fix processing error handling.
- Create mechanisms to automatically delete old, forgotten messages.
- Fix mechanism of commands allowed for syncing.

### Safety & Security
- Implement several mechanisms to ensure safety while calling callbacks.

### Optimizations
- Improve loops and pool system for enhanced multi-threading.

## v1.0.0 - ReliseCandidate

### Updates
- Basic and fundamental functionalities are working!

## v0.2.0-PreAlpha (11/07/2023)

### Updates & New Features
- Now the transposer has a dynamically set number of workers. This allows setting more or fewer workers to simultaneously process commands received, calling Python callbacks at once, and speeding up the processing in the host.
- Find ways to acquire GIL in a more efficient way and with lifetimes throughout all the Python dependencies & compiler.

## v0.1.0-PreAlpha (05/07/2023)

### Updates
- Initial pre-alpha release.









