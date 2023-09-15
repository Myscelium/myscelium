# Changelog

## v1.3.0 - ReliseCandidate (18/08/2023)

### Updates:

- Modify client allowed by implementing a client table that you can personalize several permissions of client
- Now is possible to use again the client contact event watcher without having exceptions by multiple python gill aquire
- Improve tests capabilities
- Add base to start the binary sub channels links
- Add a base to implement permission groups and indexing that to client
- Thread pools was being update in both client and host, this provides more precise executions
- Centralize commom functions reducing code complexity and improve code efficient
- Improve verification of buffer responses
- Make tests of connection more stroger
- Enhance client event transposition threads structure
- Start planing a graphical representation of tests
- Implement Test of redirect
- Centrilize morre python functions in commom, like extract_args_type, dict_to tuple and the handle_pyobject that was changed to extract_pyobject
- Add a mecanism to add new allowed client in flight from mys_host obj
- Add direct conversion from `ResultType` to rust types
- Add a new type of responses to host patters `ResponseType:InternalMannangement` this allows to change internals of host throught callbacks responses
   
    - This allows:
        - User Manipulation:
            - Add users from callbacks responses
            - Remove users from callback responses
            - Update users Throught callbacks Responses
        - Groups manipulation
            - // TODO - Edit a group from callbacks responses
            - // TODO - Add a group from callbacks responses
            - // TODO - Remove a group from calbacks response
   
    - Why only from callbacks response:
        - Whel if this is only possible throught callbacks response you will need to config it, so it will not be possible if you don't add a callback to it, what gives more sucurity to you dont have to mannage all the permissions manually at all, so even that a user has permission to do so but you dont have this callback registred the user will not be able to do so what gives another layer of security

- Add `type_of` and `fast_verify_kwargs_and_types` to `ResultType` enum, this can facilitate the process of checking a type of a `ResultType` and also recursively check if the Result matches with a predefined pattern in a very fast way by providing a reference target to `fast_verify_kwargs_and_types` that cant return an empy Ok() when all is fine and return a enum `ExpectationError` when something isn't correct. 

### Fixes:

- Now client contact event callback caller doesn't generate random exceptions
- Client permissions doesn't need to be passed every time because now has a permanent database table to hold the client informations
- Tests now are working as intended
- Thread pool has been improved, and now it doesn't bring errors anymore
- Improve several mecanism implement new error capabilities and centralizing to better updates in future 
- Fix verification if client is in whitelist
- Solve issues in tests of connection
- Fix removing commands from queue when alwready receive a response from were we send it to
- Fix wrapper client transposition zombiee threads
- Fix client random quiting by rearagin the processes event controlers in python side
- Fix the redirect adding the code to handle this case in host


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









