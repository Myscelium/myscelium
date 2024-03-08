The command patterns after the Myscelium 1.3 receives in the version 1.3.1 a significant update, a mechanism that allows responses to be sended back to a specified handler in a specified target with a specified command type, so it can be automatically redirected if the client that send the command that this response have the permission to it, but not only that, you can also determine the type of this response, so if you want to send the response to a `InternalFunction` no problem, if you want to send it to a `ExternalFunction` no problem too. The structure to you cast a command in client side is like that:

```Rust
command = client_patterns.command_pattern(
    origin_key=CLIENT_KEY,
    command_function="python_function",
    target_key="",  # Empty is default
    kwargs={"age": 10, "birth": 8, "name": "cristian"},
    message="",
    response_type="ExternalFunction", # Type of the handler that this response will trigger
    response_target="Origin", # To Where the response will be sended
    response_actf="test_handler", # the handler that will be activated
)
```

This tree fields in the end are transmited to the info carrier argument in the Remote Handler, this way we can use it inside the handler to do things like the following ones:

```Rust
class MyHostHandlers:

    @staticmethod
    def python_function(info:dict, age:int, birth:int, name:str):
        print("Access python function")
        print(birth)
        print(name)
        print(age)

        print(f"info is this: {info}")

        if "response_actf" in info:
            pass
        else:
            print("info don't have the response_actf, sending none")
            return None

        response_actf = info["response_actf"]

        host_patterns = HostPatterns()

        response = host_patterns.response_pattern(
            activation_function=response_actf,
            kwargs={"data": 'hello!'}
        )

        # (callback name) - Receive Data: [Data received list for comparison]

        return response
```

Above we can see a example of the `dinamic response activation function` in action, we use the info response_actf info parameter as a response activation_function parameter to send the response back to the handler defined in the client that call it, and with this method we can do the same to the response command type to define if the command triggered will be a `InternalFunction` or a `ExternalFunction` for example, but not only that we can also define a target to the response, like send the response to another client and trigger a handler in this client for example.

> IMPORTANT: Its nice to remember that this parameters execute some rules and verifications inside the crate that sees if the target exist's and is sync, if the response handler exist in the response target, if the client has permission to access this client, etc.. so this isn't the same of sending this parameters via handler argument, that theoretically can do the same thing, because a loot of hard verifications are done inside the client and the host oxidized core to check for violations in the rules and in the parameters

Also you can still use the traditional method of calling remote handlers if you want, that doesn't covers the new `dinamic responses` as show above, this way of doing it is more restrictive and not too recommended because can make things like the interface remote testing functionality not work as intended, however if you want to do so is just define the responses of the handlers to a defined things like this:

```Rust
class MyHostHandlers:

    @staticmethod
    def python_function(info:dict, age:int, birth:int, name:str):
        print("Access python function")
        print(birth)
        print(name)
        print(age)

        print(f"info is this: {info}")

        // if "response_actf" in info:
        //     pass
        // else:
        //     print("info don't have the response_actf, sending none")
        //     return None

        //response_actf = info["response_actf"]

        host_patterns = HostPatterns()

        response = host_patterns.response_pattern(
            activation_function="my_response_actf",
            kwargs={"data": 'hello!'}
        )

        # (callback name) - Receive Data: [Data received list for comparison]

        return response
```

This way you will not use the `Dinamic Responses` for this handler, and this change the way that you will call this handler too, you will need to do something like that:

```Rust
command = client_patterns.command_pattern(
    origin_key=CLIENT_KEY,
    command_function="python_function",
    target_key="",  # Empty is default
    kwargs={"age": 10, "birth": 8, "name": "cristian"},
    message="",
)
```

That is the same of the older versions, however take into consideration that eveen that you don't define the response type, response actf and other response ifnormation they will still be defined as default because the lib requires them to do some internal checking and this is important to ensure safety for example. But you can do this that way, if the requirements of the lib was supplyed then you can do what you want, the idea is that the Myscelium lib was designed to be flexible to the majority of the cases, giving power to the developed do things from simple to complex with easy.
