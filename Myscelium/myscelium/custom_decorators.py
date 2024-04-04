import functools
import warnings

def experimental(func):
    @functools.wraps(func)
    def wrapper(*args, **kwargs):
        warnings.warn(f"{func.__name__} is experimental and may change in the future.", category=FutureWarning)
        return func(*args, **kwargs)
    return wrapper

def instable(func):
    @functools.wraps(func)
    def wrapper(*args, **kwargs):
        warnings.warn(f"{func.__name__} is instable and may cause unexpected behavior.", category=FutureWarning)
        return func(*args, **kwargs)
    return wrapper

# Example usage
# @experimental
# def some_experimental_function():
#     print("This function is experimental.")

# def todo(func):
#     @functools.wraps(func)
#     def wrapper(*args, **kwargs):
#         raise NotImplementedError(f"{func.__name__} is marked as TODO and has not been implemented yet.")
#     return wrapper

# Example usage
# @todo
# def some_function_to_implement():
#     pass

def stable(func):
    @functools.wraps(func)
    def wrapper(*args, **kwargs):
        # logging.info(f"{func.__name__} is stable and safe to use.")
        return func(*args, **kwargs)
    return wrapper

# # Example usage
# @stable
# def some_stable_function():
#     print("This function is stable.")