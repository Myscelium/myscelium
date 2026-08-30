# SPDX-License-Identifier: MPL-2.0
# Copyright © 2021-2026 Cristian Camargo Filho


import inspect
from .functions import callback_pattern

class CallbackCollector:
    """
    Extract from a list of classes (Handlers, Receivers and Retransmiters) the methods and callbacks in it
    And automatically creates the callback list, simplifiing even more the process of create new callbacks and
    Methods for you host or to you client.

    IMPORTANT!: This only extract static methods toa llow you to use other methods inside your class

    Usage:

    ```
    callbacks_list = CallbackCollector([Handlers, Receivers, Retransmiters]).get_callbacks()
    ```
    """

    def __init__(self, callback_containers):
        self.callbacks = []
        for container in callback_containers:
            self._get_methods(container)

    def _get_methods(self, callback_class):
        # Get all attributes of Class
        for name, obj in inspect.getmembers(callback_class):
            # Check if it is a function/method
            if (
                inspect.isfunction(obj)
                or inspect.ismethod(obj)
                or isinstance(obj, staticmethod)
            ):
                # If it's a static method, get the underlying function
                if isinstance(obj, staticmethod):
                    obj = obj.__get__(None, None)

                # Check if obj is not None before proceeding
                if obj is not None:
                    callback_pattern_result = callback_pattern(callback=obj)
                    # Check if callback_pattern_result is not None before appending
                    if callback_pattern_result is not None:
                        self.callbacks.append(callback_pattern_result)

    def get_callbacks(self):
        """
        Extract from a list of classes (Handlers, Receivers and Retransmiters) the methods and callbacks in it
        And automatically creates the callback list, simplifiing even more the process of create new callbacks and
        Methods for you host or to you client.

        Usage:

        ```
        callbacks_list = CallbackCollector([Handlers, Receivers, Retransmiters]).get_callbacks()
        ```
        """
        return self.callbacks