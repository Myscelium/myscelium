import toml
import os

def validate_config(config_data, default_config):
    for section, fields in default_config.items():
        if section not in config_data:
            raise ValueError(f"Missing section: {section}")
        for key, default_value in fields.items():
            if key not in config_data[section]:
                raise ValueError(f"Missing key: {key} in section: {section}")
            if not isinstance(config_data[section][key], type(default_value)):
                raise TypeError(f"Incorrect type for key: {key} in section: {section}. Expected {type(default_value).__name__}, got {type(config_data[section][key]).__name__}")
            if key == 'test_node_name' and config_data[section][key] == "":
                raise ValueError(f"'test_node_name' in section: {section} cannot be an empty string.")
            if key == 'node_disk_name' and config_data[section][key] == "":
                raise ValueError(f"'node_disk_name' in section: {section} cannot be an empty string.")

# TODO >>> Add one more field to the disk, so you can set what disk you is using since disk impacts this project

def load_configs (config_path='config.toml'):
    # Define default configuration
    default_config = {
        'configs': {
            'test_node_name': '',
            'node_disk_name': '',
        }
    }

    # Check if the configuration file exists
    if os.path.exists(config_path):
        # Load the existing configuration
        with open(config_path, 'r') as config_file:
            config_data = toml.load(config_file)
    else:
        # Create the configuration file with default values
        config_data = default_config
        with open(config_path, 'w') as config_file:
            toml.dump(config_data, config_file)
        print(f"Configuration file '{config_path}' created with default values.")
        raise ("Your config is new, you must configure it before run the tests again!")

    # Validate the loaded or default configuration
    validate_config(config_data, default_config)
    
    return config_data