from myscelium import MysceliumHost, HostPatterns

class MyHost:
    def __init__(self):
        self.host_patterns = HostPatterns()
        self.event = None

    @staticmethod # Decorator to convert instace method into static method
    def python_function(self, age, birth, name):
        print("Access python function")
        print(birth)
        print(name)
        print(age)

        response = MyHost.host_patterns.response_pattern(
            response_mode='to_origin',
            response_activation_function="test_handler",
            response={"data": 'hello!'}
        )

        if self.event:
            print("Python function is setting the event!")
            self.event.set()

        return response

    @staticmethod
    def test_redirect(self, client_id, data):
        if isinstance(client_id, str):
            print(f"Redirecting data: {data} to client: {client_id}")
            response = MyHost.host_patterns.response_pattern(
                response=data,
                response_mode='redirect',
                redirect_to_client_id=client_id
            )
            return response
        else:
            print("Client id isn't a string, failed to redirect data!")
            return None

    @staticmethod
    def handle_client_contact(self, client_id):
        print("Access heartbeat handler")
        print(f"Client: {client_id}, made contact")

        if self.event:
            print("Heartbeat handler is setting the event!")
            self.event.set()

        return None

    def run(self, ip="127.0.0.1", port=4444, event=None):
        self.event = event

        callbacks = [
            self.host_patterns.callback_pattern(callback=self.python_function,
                                                args={"birth": "str", "name": "str", "age": "int"}),
            self.host_patterns.callback_pattern(callback=self.test_redirect,
                                                args={"client_id": "str", "data": "dict"}),
        ]

        allowed_clients = [
            self.host_patterns.client_pattern(client_type="Interface", client_id="some_client_id"),
            self.host_patterns.client_pattern(client_type="Interface", client_id="randomsclientids"),
        ]

        mys_host = MysceliumHost(callbacks=callbacks, host_id="xnsmdkeflerpfsa",
                                 allowed_clients=allowed_clients, buffer_path="Data/", n_workers=2)

        client_heart_beat_handler = [self.host_patterns.callback_pattern(callback=self.handle_client_contact,
                                                                         args={"client_id": "str"}), ]

        mys_host.set_client_heartbeat_handler(callback=client_heart_beat_handler)
        mys_host.initialize_host(ip=ip, port=port)

        return mys_host
    

