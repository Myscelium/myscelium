def cast_response_command_instruction(
    command_mode: str,
    command_type: str,
    command_target: str,
    command_status: str,
    command_origin: str,
    command_actf: str,
    command_kwargs: dict,
    command_message: str,
    auto_collect: bool = True,
) -> dict:
    """
    Constructs a command instruction dictionary from the given parameters.

    Validates the provided arguments against specific criteria for each command aspect
    (mode, type, target, status, origin, activation function). Raises an exception if
    any of the provided arguments do not meet the expected values or format.

    Parameters:
    - command_mode (str): Mode of the command, must be one of ['Function', 'Response'].
    - command_type (str): Type of the command, must be one of ['SpecialFunction', 'DirectFunction',
                      'InternalManagement', 'ExternalFunction'].
    - command_target (str): Target of the command, should follow the format 'Origin',
                            'ClientKey(String)', or 'Host'.
    - command_status (str): Status of the command, must be one of ['Success', 'Failure'].
    - command_origin (str): Origin of the command, must be 'Host' or 'ClientKey(String)'.
    - command_actf (str): Activation function for the command. Cannot be empty.
    - command_kwargs (dict): Additional keyword arguments for the command.
    - command_message (str): Message associated with the command.

    Returns:
    dict: A dictionary representing the constructed command instruction.

    Raises:
    Exception: If any parameter does not conform to its expected format or allowable values.

    Example:
    command_dict = cast_command_instruction("Function", "DirectFunction", "Host", "Success",
                                            "ClientKey(123)", "activate", {}, "Execute action")
    """

    if command_mode not in ["Function", "Response"]:
        raise ValueError(
            "Command mode needs to be one of those: ['Function', 'Response']"
        )

    if command_type not in [
        "SpecialFunction",
        "DirectFunction",
        "InternalManagement",
        "ExternalFunction",
    ]:
        raise ValueError(
            "Command type needs to be one of those: ['SpecialFunction', 'DirectFunction', 'InternalManagement', 'ExternalFunction',]"
        )

    if command_target in ["Origin", "Host"]:
        pass

    # -> Validate the redirect cases:
    elif command_target.startswith("ClientKey(") and command_target.endswith(")"):
        # Extracting the part inside 'ClientKey()'
        content = command_target[len("ClientKey(") : -1].strip()
        
        # Validate the content inside the parentheses
        if content == "":
            raise ValueError("Command target ClientKey needs a valid ClientKey!")
        
        command_target = content
        

    else:
        raise ValueError(
            "Command target must be either 'Origin', 'Host', or 'ClientKey(some_value)'"
        )

    if command_status not in ["Success", "Failure"]:
        raise ValueError(
            "Command status can only be one of those: ['Success', 'Failure']"
        )

    if command_origin == "Host":
        pass

    # -> Validate the client cases:
    elif command_origin.startswith("ClientKey(") and command_origin.endswith(")"):
        # Extracting the part inside 'ClientKey()'
        content = command_origin[len("ClientKey(") : -1].strip()
        
        # Validate the content inside the parentheses
        if content == "":
            raise ValueError("Command target ClientKey needs a valid ClientKey!")
        
        command_origin = content

    else:
        raise ValueError(
            "Command origin must be either 'Host' or 'ClientKey(some_value)'"
        )

    if (
        auto_collect
    ):  # command_actf definition is only required in cases where auto_collect is on
        if command_actf == "" or command_actf == None:
            raise ValueError("Command activation function can't be empty")
    else:
        pass

    command_instruction = {
        "mode": command_mode,
        "type": command_type,
        "target": command_target,
        "status": command_status,
        "origin": command_origin,
        "actf": command_actf,
        "kwargs": command_kwargs,
        "message": command_message,
        "response_type": command_type,  # This is a duplication due to a temporary change in the Option downcast
        "response_target": command_origin,  # This is a duplication due to a temporary change in the Option downcast
        "response_actf": command_actf,  # This is a duplication due to a temporary change in the Option downcast
        "collect_response": auto_collect,  # Default here is true, but if can be changed in the command that trigger this handler that send this response, if False it will not be automatically Transposed.
    }

    # TODO >>> Find a sulution to use None in the above struct casting in the `response_type`, `response_target`, `response_actf` and not a repetition of the act and other fields used to replace None

    return command_instruction


# > ----------------------------------------------------------------------------------------------------------------------------------------------
# > Callbacks

import inspect

def callback_pattern(callback) -> dict:
    """
    Create a callback pattern.

    Parameters:
    - callback: The callback function.

    args and kwargs: Will be auto inferred by the wrapper, just add the notes to your functions.

    Returns:
    - Dictionary representing the callback pattern.
    """

    # > The idea of this pattern
    # >
    # > (Client 1)         [Host]
    # >    |                 |
    # >    | --------------> |
    # >    |                [|]
    # >   (|) <------------- |
    # >    |                 |
    # > (Client 1)         [Host]
    # >
    # > (|) This is this callback
    # > [|] This is a host process
    # >
    # > Basically this creates a callable, that host can execute remotely by redirecting some command or sending some command

    sig = inspect.signature(callback)
    params = sig.parameters

    args = {}

    for name, param in params.items():
        if param.annotation is inspect._empty:
            function_name = callback.__name__
            raise f"function: {function_name} has args or kwargs without the required notes!"
        else:
            pass

        args[name] = str(param.annotation.__name__)

    else:
        pass

    if "info" not in args:
        function_name = callback.__name__
        raise ValueError(
            f"Fist argument must be info, ifnor is a carrier argument designed to send crutial info about then engine and general staus, you must define it inside function: {function_name}"
        )
    else:
        pass

    args.pop(
        "info", None
    )  # Remove the info carrier arg in the start of the callback argument

    # Info carrier argument is only defined in the function as a delimiter to the
    # information that will be sended by the engine to the callback, it isn't a
    # requirement for remote call this argument, however this is necessary to delimit
    # that space and say that the first arg is the info carrier. The engine know that
    # too so it isn't required to be sended to inside the engine.

    callback_pattern = {
        "function": callback,
        "args": args,
    }

    return callback_pattern


def split_dataframe(df, num_chunks):
    """
    Split a DataFrame into num_chunks parts.

    If the DataFrame cannot be split exactly into num_chunks, the remainder will be
    distributed among the chunks.

    Parameters:
    - df: DataFrame to be split
    - num_chunks: Number of chunks

    Returns:
    - List of DataFrames
    """

    n = len(df)
    chunk_size = n // num_chunks
    remainder = n % num_chunks

    chunks = []
    start = 0

    for i in range(num_chunks):
        end = start + chunk_size

        if remainder:  # Distribute the remainder across the initial chunks
            end += 1
            remainder -= 1

        chunks.append(df.iloc[start:end])
        start = end

    return chunks