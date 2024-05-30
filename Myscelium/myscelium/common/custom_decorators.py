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

def stable(func):
    @functools.wraps(func)
    def wrapper(*args, **kwargs):
        # logging.info(f"{func.__name__} is stable and safe to use.")
        return func(*args, **kwargs)
    return wrapper
