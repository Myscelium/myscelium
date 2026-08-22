# SPDX-License-Identifier: MPL-2.0
# Copyright © 2021-2026 Cristian Camargo Filho

# TODO >>> Create a test that auto create one host and client and test all the available functionalities of host and clients

#> The idea of doing this is auto test everything with one button, and every thing will be tested
#> The successfully ones will be marked with ✅ and the possible bad outcomes if has one will be marked with ❗or ⛔ or 💥

#> We can also made a test mechanism to tes like 1000 runs of something, test redirects, test file transfer, and then made the final build
#> We can also test bad cases where something is to be wrong, to test the lib strength, etc..

#> And if it passes in all test it will compile the build an jenkins will auto delivery the lib built and the things done in it

import json

json_dict = {"response":"{\"response_mode\":{\"Str\":\"to_origin\"},\"response\":{\"Map\":{\"test_redirect\":{\"Map\":{\"client_id\":{\"Str\":\"str\"},\"data\":{\"Str\":\"dict\"}}},\"get_registred_commands\":{\"List\":[]},\"python_function\":{\"Map\":{\"name\":{\"Str\":\"str\"},\"age\":{\"Str\":\"int\"},\"birth\":{\"Str\":\"str\"}}}}}}"}

print(json.loads(json_dict["response"]))