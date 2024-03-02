- [Setting Up the Client](#setting-up-the-client)
- [MysceliumClient Class](#mysceliumclient-class)
- [ClientPatterns Class](#clientpatterns-class)
- [Non Bloking Client Usage Guide](#myscelium-client-multithreading-usage-guide)

## Myscelium Client

### Setting Up the Client

##### You will need to define your receivers class, in this case is a class that will contain your functions

```python
class Receivers:
    @staticmethod
    def test_handler(info: dict):

        if "status" in info:
            pass
        else:
            return None

        if info["status"] == "success":
            pass
        else:
            return None

        # Do something with the data, lets say, save it into a database

        print("Received data: ", info)

        time.sleep(5)
```

##### Define the main function to the things that this clien node will do:

- Here first you will need to define your senders:

```python
class Senders:
    @staticmethod
    def send_some_data():
        time.sleep(25)

        mys_client = MysceliumClient(
            name="TestClient1",
            client_uid="some_client_id",
            buffer_path="Temp/Client1Data/",
            is_main_process = False
        )

        mys_client.running = True

        max_attempts = 10
        attemtps = 0
        while not mys_client.is_client_ready():
            time.sleep(1)
            attemtps += 1
            if attemtps >= max_attempts:
                assert False, "Take too long to client be ready"
            continue

        # origin_key:str, command_function:str, target_key:str="", kwargs:dict={}, message:str=""
        command = client_patterns.command_pattern(
            CLIENT_KEY,
            "python_function",
            "",  # Empty is default
            {"age": 10, "birth": 8, "name": "cristian"},
        )

        result = mys_client.send(command, priority=10)

        Events_Manager(Unit="Client1", path="Logs").Set_Event(
            step="Data Sended", event_type="Send", event_key="088p72pbv9Ozj7T1"
        )

        print(result)
```

Then you will need to define you main workflow, this is the root of your client processing structure, the part that will do things with the commands received,here because of the way myscelium was developed to work you will need a queue because the commands that client will receive in response will be received using a async structure that can be called at any time to pass through limitaitons related to timeout, so, you willl need to have a queue that allows you to store the responses while a worker check if the response for some command was received

Something like this:

<img src="./Resources/ClientCommandsFlowExample.png" alt="Description of Image">

Above is a example of how the Commands are sended and how they are received, you can see that the Senders send a order to inside the myscelium engine to prepare a command to send, then the receivers receives the commands in a async way, then this commands received can save some instruction into a db or a global of you program. Then a loop thread in your program can look into this storage and check for the response and when find it return to the main flow and do stuff with it, dis is a aproach that is more like a await system but without timeout and other limitations.

Isn't the only way to do it by the way, however is a important technique too.

```python

def wait_response(parity_id):
  while True:

    # Check for the response inside the global or the database

    if find:
      return resp
    else:
      pass

    time.sleep(1)

    continue

def my_workflow ():

  time.sleep(20) # waith the crate to initialize

  senders = Senders()

  while True:
    parity_id = senders.test_handler()
    response = wait_response (parity_id)

    # Do something with the response

    continue

```

##### Define your client main class

```python

from mysclium import MysceliumClient, CallbackCollector

class MyClient:
    def __init__(self, debug_level):
        self.debug_level = debug_level

    def initializer(self):

        # Define the client
        mys_client = MysceliumClient(
            name="TestClien1",
            client_uid=CLIENT_KEY,
            buffer_path="Temp/Client1Data/",
            log_level=self.debug_level,
            is_main_process = True
        )

        self.mys_client = mys_client

        # This will collect the callbacks from the classes
        callbacks = CallbackCollector([
          Receivers,
        ]).get_callbacks()

        # You need to set your callbacks and workes before initialize client
        mys_client.set_callbacks(callbacks=callbacks)
        mys_client.set_workers_num(n_workers=2)

        mys_client.initialize_client("127.0.0.1", 4444)

        return

    def run(self):
        senders = Senders()

        t1 = Process(target=self.initializer, args=())

        # If you want to start a thread to schedule commands to the client send
        # then you should use the following t2 setup:

        t2 = Process(target=wait_response, args=())

        t1.start()
        time.sleep(5)

        #! It is important that t2 be initialized after 15 seconds latter after the initialization
        #! of t1, this is required because the client needs to initialize before send any commands,
        #! and this 15s delay is necessary to client initializate

        t2.start() # start t2 seutp (if you want the workflow to send things actively)

        time.sleep(5)

        # PID is the process ID of the process you want to send the signal to.
        # You would typically get this from the 'pid' attribute of a process.
        os.kill(t1.pid, signal.SIGINT)

        t1.join()  # Wait for the process to finish
        t2.join()


        return
```

This will create a client, when you run this client it will try to connect in: `127.0.0.1:4444` where it will try to find a compatible host,
also it's important that you client key be correctly setuped, when you client enters in contact to the host, host will verify if the client is a valid client, if it is in the white list, if the client isn't in the white list then client will be droped and it will cause a disconnect event, client also will receive a error saying that the cause for the exception was that host detects a unautrized key for connection. to be able to connect with a valid one will be necessary to change the key to a valid one or create this key in host.

Now it's only run it and your client will start, based on the above class, to run it is simple, just do:

```python
MyClient(debug_level="INFO").run()
```

---

## Senders:

What are senders? Well senders are classes that allows to define groups of functions to send data to another client connected in the myscelium network. They are more for grouping purposes since they don't make into the myscelium core anyway,they work like a module of senders, bellow there is a elaborated example of how this works

```python
class Senders:
    @staticmethod
    def send_some_data():
        time.sleep(25)

        mys_client = MysceliumClient(
            name="TestClient1",
            client_uid="some_client_id",
            buffer_path="Temp/Client1Data/",
            is_main_process = False
        )

        mys_client.running = True

        max_attempts = 10
        attemtps = 0
        while not mys_client.is_client_ready():
            time.sleep(1)
            attemtps += 1
            if attemtps >= max_attempts:
                assert False, "Take too long to client be ready"
            continue

        # origin_key:str, command_function:str, target_key:str="", kwargs:dict={}, message:str=""
        command = client_patterns.command_pattern(
            CLIENT_KEY,
            "python_function",
            "",  # Empty is default
            {"age": 10, "birth": 8, "name": "cristian"},
        )

        result = mys_client.send(command, priority=10)

        Events_Manager(Unit="Client1", path="Logs").Set_Event(
            step="Data Sended", event_type="Send", event_key="088p72pbv9Ozj7T1"
        )

        print(result)
```

Above we have a example of a Sender class, we can see the basic definition of a sender that consists in a class with at least one function decorated as Static Method, this function needs to be a static method because if doesn't need the self parameter, the basic structure of a sender function consists in a function that uses a MysClient instance configured as secondary process that allows to schedule things inside myscelium buffer up, this processes includes:

1. Make a instance of myscelium client
2. Define running to true
3. Define a Ready State whatcher that is this block bellow:

```python
max_attempts = 10
attemtps = 0
while not mys_client.is_client_ready():
   time.sleep(1)
   attemtps += 1
   if attemtps >= max_attempts:
       assert False, "Take too long to client be ready"
   continue
```

This uses the `mys_client.is_client_ready()` method to check if the client is ready using relative states, this states are shared through db, when client becomes ready it stores the state into this table, this means tht client is ready to send commands, this status checking is required because client has to have numerous process before it's become ready to send information beack and fourth in a secure way .

After verify that the client is ready you will need to cast a command and to cast a command is necessary to use client patterns, that is a class containing several patterns that you can use to cast important patterns used inside client side of myscelium network, for example:

1. Command patterns
2. Command to redirect patterns
3. Response patterns
4. Response to redirect patterns
5. Inner management command patterns

This is some examples of what can be used, but there is other combinations possible too that will be explained in the wright section realted to it. Here however to cast a basic command you will need to do the following:

```python
from myscelium import ClientPatterns

command = client_patterns.command_pattern(
   CLIENT_KEY,        # origin_key:str
   "python_function", # command_function:str
   "",                # target_key:str - Empty is default means send to host
   {"age": 10, "birth": 8, "name": "cristian"}, # Kwargs
   "" # Optinal messages
)
```

Above we can see a basic example of how the command patters works, it takes some kwargs like:

##### client key:

- The key of the client that is sending it
- This will be auto infered in future for security reasons

##### command function:

- This is the function that will be activated in the target when this command will be executed

##### target_key:

- is the final destination that this command needs to reach. The way that the myscelium network works is by using a mechanism that redirect the commands untill them arrive into the correct place, for example if our target is host it will be sended to host and them will stop there. However if we want to send it to other place then it will be redirected throught the myscelium network untill it arrives in the target, the same works for the responses too

# TODO >>>

### MysceliumClient Class

---

### Myscelium Client Multithreading Usage Guide

#### 1. **Import Necessary Modules:**

```python
from myscelium import MysceliumClient, ClientPatterns
from multiprocessing import Process
import time
```

#### 2. **Initialize Client Patterns:**

```python
client_patterns = ClientPatterns()
```

#### 3. **Define Callback Functions:**

This function will be triggered when the client receives a response.

```python
def test_handler(data):
    print("Receive data:", data)
    return None
```

#### 4. **Setup Callbacks:**

```python
callbacks = [
    client_patterns.callback_pattern(
        callback=test_handler,
        args={"data": "dict"}
    ),
]
```

Or you can use the callbacks collector to automate this by do the following:

```python

class Receivers:

    def __init__ (self):
        pass

    @staticmethod
    def example_receiver (data:dict):
        pass

class Retransmiters:

    def __init__ (self):
        pass

    @staticmethod
    def example_retransmiter (data:dict):
        pass

    @staticmethod
    def example_retransmiter (data:dict):
        pass

callbacks = CallbackCollector([Receivers, Retransmiters]).get_callbacks()
```

With this CallbackCollector we can extract all the callbacks of these call and also the types that these callbacks takes,
imediatly automate the callbacks of these receivers.

** IMPORTANT! ** Take in consideration that now all client functions require data:dict arg, this is a thing to allow the following:

```python
"data": {
  "command_type":"response",
  "status": "success"
  "response_activation_function":"",
  "message":"",
  "kwargs":{"arg1": [], "arg2": "", "arg3": {}}
  "response_mode":"",
}
```

So now client have the entire control to status, activation function to allow create advanced activation switches,
also you have the access to the entire kwargs, a message field and a response mode, the response mode indicates if it is:

- `redirect`
- `to_send`

And also, now you can retransmit messages direct adding a possibility to return errors using the `error_pattern` introduced in v1.3 to host be able to send error messages to client:

TODO >>> See to add a mecanism to retrasnmit from client to host to client without complications

```python
client_patterns.redirect_error_pattern (self, error_message:str, expected_remote_error_handler:str, redirect_to:str)
```

#### 5. **Function to Send Data to the Host:**

This function initializes a client, sets its UID, and sends a command to the host after a delay.

```python
def send_some_data():

    mys_client = MysceliumClient(
        client_uid="some_client_id",
        buffer_path="ClientData/"
    )

    mys_client.runing = True
    time.sleep(10)

    command = client_patterns.command_pattern(
        "python_function",
        args={"age":10, "birth":8, "name":"cristian"}
    )

    result = mys_client.send(command, priority=10)
    print(result)
```

#### 6. **Function to Initialize and Start the Client:**

This function initializes the client, sets its callbacks, worker number, and starts it.

```python
def initialize_client():

    mys_client = MysceliumClient(
        client_uid="some_client_id",
        buffer_path="ClientData/"
    )

    mys_client.set_callbacks(callbacks=callbacks)
    mys_client.set_workers_num(n_workers=2)
    mys_client.initialize_client("127.0.0.1", 4444)
```

#### 7. **Main Execution:**

Here, we use Python's multiprocessing module to run the client continuously in one process and send commands in another process. This allows the client to run independently and interact with it by sending commands.

```python
if __name__ == '__main__':
    p1 = Process(target=initialize_client)
    p2 = Process(target=send_some_data)

    p1.start()
    p2.start()

    p1.join()
    p2.join()
```

---

This setup ensures that the client runs continuously in one process, while another process can interact with it by sending commands. Callbacks are activated when there's a response. This approach provides concurrency, allowing the client to handle responses while still being able to send new commands.

-
