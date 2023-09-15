### HostPatterns Class

The `HostPatterns` class provides patterns for the host.

#### Methods

- `client_pattern(client_type:str, client_id:str) -> dict`: Returns a client pattern.
- `response_pattern(response:any, response_mode:str, response_activation_function:str = None,  redirect_to_client_id:str=None) -> dict`: Returns a response pattern.
- `callback_pattern(callback, args) -> dict`: Returns a callback pattern.


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
    update_host_configs (self, 
                        activation_function="add_client", 
                        new_client=[client_patter])

    # - new_client:list[client_pattern] -> This is a list that contains the new client to add!
    ```

- update_client, needed kwargs:

    ```python
    update_host_configs (self, 
                        activation_function="update_client", 
                        actual_client_key="xMsndkdlenfjedLj", 
                        updated_client=[client_patter])

    # - actual_client_key:str
    # - updated_client:list[client_pattern] -> This is a list that contains the new client updated!    
    ```

- remove_client, need kwargs:

    ```python
    update_host_configs (self, 
                        activation_function="remove_client", 
                        actual_client_key="xMsndkdlenfjedLj")

    # - client_key:str -> The client key of the client that you want to remove.
    ```
