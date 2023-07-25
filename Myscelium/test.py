

import json

json_dict = {"response":"{\"response_mode\":{\"Str\":\"to_origin\"},\"response\":{\"Map\":{\"test_redirect\":{\"Map\":{\"client_id\":{\"Str\":\"str\"},\"data\":{\"Str\":\"dict\"}}},\"get_registred_commands\":{\"List\":[]},\"python_function\":{\"Map\":{\"name\":{\"Str\":\"str\"},\"age\":{\"Str\":\"int\"},\"birth\":{\"Str\":\"str\"}}}}}}"}

print(json.loads(json_dict["response"]))