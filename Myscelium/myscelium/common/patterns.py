from pydantic import BaseModel, Field
from pydantic import BaseModel, field_validator, model_validator
from typing import List, Dict, Any, Optional

class ClientPattern(BaseModel):
    client_name: str = Field(..., description="Name of the client (user).")
    client_key: str = Field(..., description="Unique Key of the client.")
    client_type: str = Field(..., description="Client purpose.")
    client_permission_group: str = Field(..., description="Group that client inherits permission.")
    client_is_super_user: bool = Field(..., description="If client has root privileges on myscelium.")
    max_sub_channels: int = Field(..., description="Max sub-channels of stream that client are allowed to create and manage.")
    owned_sub_channels_keys: List[str] = Field(default=[], description="Optional parameter to preinitialize host with client sub-channels keys allowed.")

    def to_dict(self) -> dict:
        """
        Returns a dictionary representing the client pattern.
        """
        return self.dict()

    def format(self) -> dict:
        """
        Returns a dictionary with modified key names to match a different format.
        """
        formatted_dict = {
            "client_name": self.client_name,
            "client_key": self.client_key,
            "client_type": self.client_type,
            "permission_group": self.client_permission_group,
            "is_super_user": self.client_is_super_user,
            "max_sub_channels": self.max_sub_channels,
            "owned_sub_channels_keys": self.owned_sub_channels_keys
        }
        return formatted_dict
    
class CommandInstruction(BaseModel):
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
    - command_actf (str): Activation function for the command. Cannot be empty.
    - command_kwargs (dict): Additional keyword arguments for the command.
    - command_message (str): Message associated with the command.
    - response_type (str): The type of the response command that will be sended back to this node that is casting this command,
    - response_target (str): The target of the response that this command will produce,
    - response_actf (str): The handler that response will triger when arrive in the target

    Returns:
    dict: A dictionary representing the constructed command instruction.

    Raises:
    Exception: If any parameter does not conform to its expected format or allowable values.

    Example:
    command_dict = cast_command_instruction("Function", "DirectFunction", "Host", "Success",
                                            "ClientKey(123)", "activate", {}, "Execute action")
    """

    command_mode: str
    command_type: str
    command_target: str
    command_status: str
    command_actf: str
    command_kwargs: Dict[str, Any]
    command_message: str
    response_type: str
    response_target: str
    response_actf: str
    auto_collect_response: bool

    def format(self) -> Dict[str, Any]:
        """
        Converts the model instance into a customized dictionary format,
        based on specific attribute mappings.

        Returns:
            Dict[str, Any]: The dictionary representation of the model instance with custom keys.
        """
        return {
            'mode': self.command_mode,
            'type': self.command_type,
            'target': self.command_target,
            'status': self.command_status,
            'actf': self.command_actf,
            'kwargs': self.command_kwargs,
            'message': self.command_message,
            'response_type': self.response_type,
            'response_target': self.response_target,
            'response_actf': self.response_actf,
            'collect_response': self.auto_collect_response
        }

    def to_dict(self) -> Dict[str, Any]:
        """
        Converts the model instance into a dictionary, with the ability to add custom
        modifications or additional key-value pairs.

        Returns:
            Dict[str, Any]: The dictionary representation of the model instance.
        """
        # Convert the model to a dictionary using Pydantic's built-in method
        model_dict = self.dict()

        # Example of adding a custom key-value pair or modifying the dict
        model_dict['custom_key'] = 'custom_value'
        
        # You can also modify existing data if necessary
        # For instance, transforming nested dictionaries or lists if needed
        # model_dict['command_kwargs'] = custom_transform(model_dict['command_kwargs'])

        return model_dict

    @field_validator('command_mode', 'command_status')
    def check_fixed_choices(cls, v, field):
        if field.field_name == 'command_mode' and v not in ['Function', 'Response']:
            raise ValueError("Command mode must be one of ['Function', 'Response']")
        if field.field_name == 'command_status' and v not in ['Success', 'Failure']:
            raise ValueError("Command status must be one of ['Success', 'Failure']")
        return v

    @field_validator('command_type', 'response_type')
    def check_type_choices(cls, v, field):
        allowed_types = ['SpecialFunction', 'DirectFunction', 'InternalManagement', 'ExternalFunction']
        if field.field_name == 'response_type':
            allowed_types = ['DirectFunction', 'InternalManagement', 'ExternalFunction']
        if v not in allowed_types:
            raise ValueError(f"{field.field_name} must be one of {allowed_types}")
        return v

    @model_validator(mode="after")
    def check_keys_and_origin(cls, values):
        print(values)
        # for attribute_name in targets:  

        command_target = getattr(values, 'command_target', 'Attribute not found')
        response_target = getattr(values, 'response_target', 'Attribute not found')
        collect_response = getattr(values, 'collect_response', 'Attribute not found')

        mode = getattr(values, 'command_mode', 'Attribute not found')
        
        match mode:

            case "Function":

                #-> Command:
                #   > command_target
                #         * ClientKey(string)
                #         * Host
                #   > response_target
                #         * Origin              |
                #         * Host                | Auto collect == false
                #         * ClientKey(String)   | Auto collect == false

                if command_target == response_target:
                    raise ValueError(f"You can't schedule a command that the response points to self triggered node, command_target must be diferente than response_target")

                if command_target != "Host":
                    if command_target.startswith('ClientKey(') and command_target.endswith(')'):
                        content = command_target[len('ClientKey('):-1].strip()
                        if content == "":
                            raise ValueError(f"Command target needs a valid ClientKey not empty!")
                    else:
                        raise ValueError(f"Command target must be either 'Host', or 'ClientKey(some_value)'")
                else:
                    pass

                if response_target != "Origin":
                    if collect_response: # If the collect response is false we can't use Host nor ClientKey(String)             
                        if response_target == "Host":
                            pass
                        elif response_target.startswith('ClientKey(') and response_target.endswith(')'):
                            content = response_target[len('ClientKey('):-1].strip()
                            if content == "":
                                raise ValueError(f"Command target needs a valid ClientKey not empty!")
                        else:
                            raise ValueError(f"Command target must be either 'Origin', 'Host', or 'ClientKey(some_value)'")
                    else:
                        raise ValueError("To use inplace responses (collect_response == false) you must have the response target as Origin")
                else:
                    # If the response is == Origin this is correct!
                    pass
                    
            case "Response":

                #-> Response:
                #   > command_target
                #         * ClientKey(string)   | Auto collect == true
                #         * Host
                #   > response_target
                #         * Empty

                if command_target != "Origin":

                    if not collect_response:
                        raise ValueError(f"Can't send a inplace response to places that isn't the Origin!")

                    if command_target == "Host":
                        pass
                    elif command_target.startswith('ClientKey(') and command_target.endswith(')'):
                        content = command_target[len('ClientKey('):-1].strip()
                        if content == "":
                            raise ValueError(f"Command target needs a valid ClientKey not empty!")
                    else:
                        raise ValueError(f"Command target must be either 'Host', 'Origin', or 'ClientKey(some_value)'")

                else:
                    pass

                # TODO >>> Add the validation of the response target if necessary, but it should be empty here

        if collect_response: # If auto collect == true `command_actf` needs to be defined!
            if not getattr(values, 'command_actf', 'Attribute not found'):
                raise ValueError("Command activation function can't be empty")
            
        return values
        
    

# class ResponseInstruction(BaseModel):
    
#     command_mode: str
#     command_type: str
#     command_target: str
#     command_status: str
#     command_origin: str
#     command_actf: str
#     command_kwargs: dict
#     command_message: str
#     auto_collect_response: bool 
    
#     @field_validator('command_mode', 'command_status')
#     def check_fixed_choices(cls, v, field):
#         if field.field_name == 'command_mode' and v not in ['Function', 'Response']:
#             raise ValueError("Command mode must be one of ['Function', 'Response']")
#         if field.field_name == 'command_status' and v not in ['Success', 'Failure']:
#             raise ValueError("Command status must be one of ['Success', 'Failure']")
#         return v

#     @field_validator('command_type', 'response_type')
#     def check_type_choices(cls, v, field):
#         allowed_types = ['SpecialFunction', 'DirectFunction', 'InternalManagement', 'ExternalFunction']
#         if field.field_name == 'response_type':
#             allowed_types = ['DirectFunction', 'InternalManagement', 'ExternalFunction']
#         if v not in allowed_types:
#             raise ValueError(f"{field.field_name} must be one of {allowed_types}")
#         return v

#     @model_validator(mode="after")
#     def check_keys_and_origin(cls, values):
#         print(values)
#         targets = ['command_target', 'response_target']
#         origins = ['command_origin']
#         for attribute_name in targets + origins:  
#             target = getattr(values, attribute_name, 'Attribute not found')
#             if target not in ['Origin', 'Host']:
#                 if target.startswith('ClientKey(') and target.endswith(')'):
#                     content = target[len('ClientKey('):-1].strip()
#                     if content == "":
#                         raise ValueError(f"Command {attribute_name} needs a valid ClientKey not empty!")
#                 else:
#                     raise ValueError(f"{attribute_name} must be either 'Origin', 'Host', or 'ClientKey(some_value)'")
#             else:
#                 if target != "Origin":
#                     collect_response = getattr(values, "collect_response", 'Attribute not found')  
#                     if not collect_response:
#                         raise ValueError(f"You only can send inplace responses to origin or some to some client!")
            
#         command_target = getattr(values, "command_target", 'Attribute not found')  
#         response_target = getattr(values, "response_target", 'Attribute not found')        
        
#         if command_target == response_target:
#             raise ValueError(f"You can't schedule a command that the response points to self triggered node, command_target must be diferente than response_target")

#         if not getattr(values, 'command_actf', 'Attribute not found'):
#             raise ValueError("Command activation function can't be empty")
        # return values