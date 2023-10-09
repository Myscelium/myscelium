### HostPatterns Class

The `HostPatterns` class provides patterns for the host.

#### Methods

- `client_pattern(client_type:str, client_id:str) -> dict`: Returns a client pattern.
- `response_pattern(response:any, response_mode:str, response_activation_function:str = None,  redirect_to_client_id:str=None) -> dict`: Returns a response pattern.
- `callback_pattern(callback) -> dict`: Returns a callback pattern.


### Response_Pattern

#### `response_pattern(response:any, response_mode:str, response_activation_function:str = None,  redirect_to_client_id:str=None) -> dict`


```python

 def update_host_configs (self, activation_function:str, **kwargs): # TODO >>> Need rust backend implementation!

        if activation_function == "add_client":

            if "new_client" in kwargs:
                pass
            else:
                raise "new client isn't in kwargs, so can't add client!"

            new_client = kwargs["new_client"]

            response = {'new_client':new_client}

            return {'response_mode':'InternalMannangement', 'activation_function':'add_client', 'kwargs':response}

        elif activation_function == "update_client":

            if "actual_client_key" in kwargs:
                pass
            else:
                raise "actual_client_key isn't in kwargs, so can't update client!"
            
            actual_client_key = kwargs["actual_client_key"]

            if "updated_client" in kwargs:
                pass
            else:
                raise "new client isn't in kwargs, so can't edit client!"
            
            updated_client = ["updated_client"]

            response = {'actual_client_key':actual_client_key, 'updated_client':updated_client}

            return {'response_mode':'InternalMannangement', 'activation_function':'update_client', 'kwargs':response}
        
        elif activation_function == "remove_client":

            if "client_key" in kwargs:
                pass
            else:
                raise "client_key isn't in kwargs, so can't remove client!"
            
            client_key = kwargs["client_key"]

            response = {'client_key':client_key}

            return {'response_mode':'InternalMannangement', 'activation_function':'remove_client', 'kwargs':response}

        else:
            raise f"activation_function: {activation_function} doesn't registred in the avalaible host internal mannangement commands!"

```

### Create a response pattern.


Parameters:

- add_client, needed kwrags: 

    ```python
    update_host_configs (
        self, 
        activation_function="add_client", 
        new_client=[client_pattern]
    )

    # - new_client:list[client_pattern] -> This is a list that contains the new client to add!
    ```

- update_client, needed kwargs:

    ```python
    update_host_configs (
        self, 
        activation_function="update_client", 
        actual_client_key="xMsndkdlenfjedLj", 
        updated_client=[client_pattern]
    )

    # - actual_client_key:str
    # - updated_client:list[client_pattern] -> This is a list that contains the new client updated!    
    ```

- remove_client, need kwargs:

    ```python
    update_host_configs (
        self, 
        activation_function="remove_client", 
        actual_client_key="xMsndkdlenfjedLj"
    )

    # - client_key:str -> The client key of the client that you want to remove.
    ```

### Host Setup:

##### Required Randlers


```python
class Handlers:

    @staticmethod
    def test_add_client (
	    client_name:str, 
	    client_key:str, 
	    client_type:str, 
	    permission_group:str, 
	    is_super_user:bool, 
	    max_sub_channels:int, 
	    owned_sub_channels_keys:list):
    
        new_client = [
            HostPatterns.client_pattern(
                client_name=client_name,
                client_key=client_key,
                client_type=client_type,
                client_permission_group=permission_group,
                client_is_super_user=is_super_user,
                max_sub_channels=max_sub_channels,
                owned_sub_channels_keys=owned_sub_channels_keys
            )
        ]

        return HostPatterns.update_host_configs(
	        activation_function="add_client", 
	        new_client=new_client
	    )

    @staticmethod
    def test_update_client (
	    actual_client_key:str,
	    client_name:str, 
	    client_key:str, 
	    client_type:str, 
	    permission_group:str, 
	    is_super_user:bool, 
	    max_sub_channels:int, 
	    owned_sub_channels_keys:list):

        updated_client = [
            HostPatterns.client_pattern(
                client_name=client_name,
                client_key=client_key,
                client_type=client_type,
                client_permission_group=permission_group,
                client_is_super_user=is_super_user,
                max_sub_channels=max_sub_channels,
                owned_sub_channels_keys=owned_sub_channels_keys
            )
        ]

        return HostPatterns.update_host_configs(
	        activation_function="update_client", 
	        actual_client_key=actual_client_key, 
	        updated_client=updated_client
	    )

    @staticmethod
    def test_remove_client (client_key:str):
        return HostPatterns.update_host_configs(
	        activation_function="remove_client", 
	        actual_client_key=client_key
	    )
```

---

## Client setup
### Senders:

```python
class Senders:

    @staticmethod
    def test_add_client ():

        time.sleep(10)

        mys_client = MysceliumClient(client_uid="some_client_id", buffer_path="Temp/Client1Data/")
        mys_client.runing = True
        mys_client.set_client_uid(client_uid="some_client_id")

        command = client_patterns.command_pattern(
                        "test_add_client",
                        args={
                            "client_name":"test_client",
                            "client_key":"xMndjslwpedcnfe",
                            "client_type":"Test",
                            "permission_group":"",
                            "is_super_user":1,
                            "max_sub_channels":5,
                            "owned_sub_channels_keys":[],
                        }
                    )

        result = mys_client.send(command, priority=10)
        
        Events_Mananger(Unit="Client1", path="Logs").Set_Event(
            "Data Sended",
            event_type="Send test add client",
            event_key=""
        ) # TODO >>> Add the event key


    @staticmethod
    def test_update_client ():
    
        time.sleep(10)

        mys_client = MysceliumClient(client_uid="some_client_id", buffer_path="Temp/Client1Data/")
        mys_client.runing = True
        mys_client.set_client_uid(client_uid="some_client_id")

        command = client_patterns.command_pattern(
                        "test_update_client",
                        args={
                            "client_name":"test_client",
                            "client_key":"xMndjslwpedcnfe",
                            "client_type":"Test",
                            "permission_group":"",
                            "is_super_user":1,
                            "max_sub_channels":10,
                            "owned_sub_channels_keys":[]
                        }
                    )

        result = mys_client.send(command, priority=10)
        
        Events_Mananger(Unit="Client1", path="Logs").Set_Event(
            "Data Sended",
            event_type="Send test update a client",
            event_key=""
        ) # TODO >>> Add the event key

    @staticmethod
    def test_remove_client ():
    
        time.sleep(10)
        
        mys_client = MysceliumClient(client_uid="some_client_id", buffer_path="Temp/Client1Data/")
        mys_client.runing = True
        mys_client.set_client_uid(client_uid="some_client_id")

        command = client_patterns.command_pattern(
                        "test_remove_client",
                        args={
                            "client_key": "xMndjslwpedcnfe"
                        }
                    )

        result = mys_client.send(command, priority=10)
        
        Events_Mananger(Unit="Client1", path="Logs").Set_Event(
            "Data Sended",
            event_type="Send",
            event_key=""
        ) # TODO >>> Add the event key
```

#### Receivers:

```python
class Receivers:

    @staticmethod
    def test_update_client (data):
        Events_Mananger(Unit="Client1", path="Logs").Set_Event(
            "Activate Basic Response Test callback handler"
        )
        
        print("Received data: ", data)
        time.sleep(5)
        # System_Status(path="Logs").change_unit_status(Unit="Client1", Status=False)

    @staticmethod
    def test_remove_client (data): # TODO
        Events_Mananger(Unit="Client1", path="Logs").Set_Event(
            "Activate Basic Response Test callback handler"
        )

        print("Received data: ", data)
        
        time.sleep(5)
        System_Status(path="Logs").change_unit_status(Unit="Client1", Status=False)

    @staticmethod
    def test_add_client (data): # TODO
    
        Events_Mananger(Unit="Client1", path="Logs").Set_Event(
            "Activate Basic Response Test callback handler"
        )
        
        print("Received data: ", data)
        
        time.sleep(5)
        System_Status(path="Logs").change_unit_status(Unit="Client1", Status=False)
```