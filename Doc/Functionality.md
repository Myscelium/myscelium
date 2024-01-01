
### ResultType functionality

```mermaid
graph TD
  A["convert_to_value_map"] --> B["dict.iter()"]
  B --> C["map"]
  C --> D["(k, v)"]
  D --> E["k.clone()"]
  D --> F1["resulttype_to_value"]
  F1 --> F2["match result"]
  F2 --> G1["ResultType::Str"]
  G1 --> R[Return]
  F2 --> G2["ResultType::Int"]
  G2 --> R
  F2 --> G3["ResultType::Float"]
  G3 --> R
  F2 --> G4["ResultType::Bool"]
  G4 --> R
  F2 --> G5["ResultType::Map"]
  G5 --> H1["map.iter()"]
  H1 --> H2["resulttype_to_value"]
  H2 --> F2["Recursive Call"]
  H2 --> R
  F2 --> G6["ResultType::List"]
  G6 --> H3["list.iter()"]
  H3 --> H4["resulttype_to_value"]
  H4 --> F2["Recursive Call"]
  H4 --> R
  F2 --> G7["ResultType::Empty"]
  G7 --> R
  F2 --> G8["ResultType::Error"]
  G8 --> R
  C --> I["collect"]
  I --> R
```

---
### Extract Py Object

```mermaid
graph TD
A[extract_pyobject] --> B[Check PyDict]
B --> C[Iterate and Convert]
B --> D[Check PyTuple]
D --> E[Convert Tuple]
B --> F[Check PyList]
F --> G[Convert List]
B --> H[Check PyInt]
H --> I[Return Int]
B --> J[Check PyFloat]
J --> K[Return Float]
B --> L[Check PyString]
L --> M[Return String]
B --> N[Check PyBool]
N --> O[Return Bool]
B --> P[Check None]
P --> Q[Return Empty]
B --> Q

```
---

### Call Python Function

```mermaid
graph TD

R[call_callback] --> S[Get Function Name]
S --> T[Get Function and Args]
S --> U[Convert Command to Kwargs]
U --> V[Convert to PyDict]
V --> W[Call Python Function]
W --> X[Return PyObject]

```


---
### Process

```mermaid  
	graph TD
	A[Start process]
	A --> B[Initialize logger]
	B --> C[Check if parity_id is registered]
	C --> D{Command not registered?}
	D -->|No| E[Remove command]
	D -->|Yes| F[Translate command]
	F --> G{Function present?}
	G -->|No| H[Remove command]
	G -->|Yes| I[Function in patterns?]
	I -->|No| J[Remove command]
	I -->|Yes| K[Acquire GIL]
	K --> L{Response mode?}
	L -->|No| M[Error: No response mode]
	L -->|Yes, to_origin| N[Convert to value]
	L -->|Yes, redirect| O[Handle redirect]
```
---
-> Process call a python function that returns a python value, then this python object response are converted into ResultType using the Extract py object, then this ResultType is converted into json Value using the convert_to_value_map function the response should have this format:

```json
{"response_activation_function": String("update_avaliable_host_commands"), "kwargs": Object {"get_registred_commands": Array [], "python_function": Object {"age": String("int"), "birth": String("str"), "event_key": String("str"), "name": String("str")}, "test_redirect": Object {"client_id": String("str"), "data": String("int")}}, "response_mode": String("to_origin")}
```

Then it will be convert into a string using: 

```rust
response = Ok(serde_json::to_string(&converted_to_value).unwrap());
```

After this, the process encode the response into a UpCommand:

```rust
let up_command = UpCommand::new(
                    client_id, 
                    down_command.parity_id.clone(), 
                    down_command.priority.clone(), 
                    response.unwrap()
                );
```

And save into the schedule.


---


```mermaid
sequenceDiagram


Client 1 ->> Host: Connect
Host ->> Host: Client isn't sync 
Host ->> Client 1: C: 01
Client 1 ->> Client 1: update available commands
Client 1 ->> Host: C: 02
Host ->> Host: Update client handlers

```

### indice:

C: 01 - `send_network_available_commands`:
```Rust
{
	"command_type": "direct_function",
	"response_mode": "to_origin",
	"status": "success",
	"function": "update_available_host_commands",
	"kwargs": {<netwrok av commands>}
	"origin": "host",
}
```

C: 02 - `update_client_commands_ref`:
```Rust
	"command_type": "direct_function_response",
	"status": "success",
	"function": "update_client_commands_ref",
	"kwargs": {<client handlers>}
	"origin": "<Client_Key>",
	"response_mode": "to_host",
```


C: 03 - `update_available_host_commands`:
```Rust
	"command_type": "direct_function_response",
	"status": "success",
	"function": "update_available_host_commands",
	"kwargs": {<client handlers>}
	"origin": "<Client_Key>",
	"response_mode": "to_host",
```


-> `request_client_available_commands`
```Rust
{
	"command_type": "direct_function",
	"response_mode": "to_origin",
	"status": "success",
	"function": "get_socket_client_available_handlers",
	"origin": "host",
	"kwargs": {<netwrok av commands>}
}
```