# Changelog

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

## v0.2.0-PreAlpha (11/07/2023)

### Updates & New Features
- Now the transposer has a dynamically set number of workers. This allows setting more or fewer workers to simultaneously process commands received, calling Python callbacks at once, and speeding up the processing in the host.
- Find ways to acquire GIL in a more efficient way and with lifetimes throughout all the Python dependencies & compiler.

## v0.1.0-PreAlpha (05/07/2023)

### Updates
- Initial pre-alpha release.

## v1.0.0 - ReliseCandidate

### Updates
- Basic and fundamental functionalities are working!


## v1.1.0 - ReliseCandidate

### Updates
- Add a automatica pytest to test all the lib making connections from client to host and testisng remote function callbacks activationa and client response handlers
- Improve lib code
- Improve wrappers
- Add better exemples of usage
- Create a way to use classes to run host and client all using Classes