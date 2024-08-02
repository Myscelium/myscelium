import pandas as pd
import seaborn as sns
import matplotlib.pyplot as plt
import streamlit as st
import datetime
from History.history_controller import History_Manager
from Logs.test_logs_manager import Events_Manager, System_Status

import pytest
import xml.etree.ElementTree as ET
import os

import json

# Define the root directory where the Temp folders are located
TEMP_DIR = os.path.join(os.path.dirname(__file__), "Temp")

# Set the page config to use the full width
st.set_page_config(layout='wide')

pd_dict_df = History_Manager().list_history()
df = pd.DataFrame.from_dict(pd_dict_df)

# Convert 'Time' to datetime if it's not
df['Time'] = df['Time'].apply(datetime.datetime.fromtimestamp)

# Handle missing values (optional based on your requirement)
df = df.dropna(subset=['Time'])

# Sidebar menu
st.sidebar.title("Menu")
option = st.sidebar.selectbox('Choose an Option', ['Test Results Visualization', 'Test Interface', 'Logs Navigator'])

def test_fn():
    pass

if option == 'Test Results Visualization':

    pd_dict_df = History_Manager().list_history()
    df = pd.DataFrame.from_dict(pd_dict_df)

    # Convert 'Time' to datetime if it's not
    df['Time'] = df['Time'].apply(datetime.datetime.fromtimestamp)

    # Handle missing values (optional based on your requirement)
    df = df.dropna(subset=['Time'])

    # Streamlit UI
    st.title('Test Results Visualization')
    st.write('Displaying DataFrame:')
    st.dataframe(df)  # Displaying original df for reference

    # Add a selection box at the top for log level
    selected_log_level = st.selectbox('Select Log Level', options=df['LogLevel'].unique())
    
    df = df.dropna()
        
    selected_test_node_name = st.selectbox('Select Test Node', options=df['TestNodeName'].unique())
    
    # Filter the data based on the selected log level
    filtered_df = df[df['TestNodeName'] ==  selected_test_node_name]
    filtered_df = filtered_df[filtered_df['LogLevel'] == selected_log_level]

    # Create columns
    col1, col2 = st.columns([1,1])

    # Use the left column for the first plot
    with col1:
        st.write('Test Speed over Time for Log Level:', selected_log_level)
        fig, ax = plt.subplots(figsize=(8, 5))  # Adjust width and height as needed
        
        # Line plot for 'TestSpeed' over 'Time', separated by 'TestName'
        lineplot = sns.lineplot(x='Time', y='TestSpeed', hue='TestName', data=filtered_df, errorbar=None, marker="o", palette="tab10")

        # Get unique test names
        test_names = filtered_df['TestName'].unique()
        
        # Loop through each test name to add a shaded region and mark outliers for each line
        for i, test_name in enumerate(test_names):
            # Filter data for the current test name
            test_data = filtered_df[filtered_df['TestName'] == test_name]

            # Calculate the median and tolerance
            median_test_speed = test_data['TestSpeed'].median()
            upper_tolerance = median_test_speed * 1.05  # +5%
            lower_tolerance = median_test_speed * 0.95  # -5%
            
            # Get the color of the current line
            line_color = lineplot.get_lines()[i].get_color()
            
            # Add shaded region
            ax.fill_between(test_data['Time'], lower_tolerance, upper_tolerance, color=line_color, alpha=0.2)  # Adjust alpha as needed
            
            # Identify outliers
            outliers = test_data[(test_data['TestSpeed'] > upper_tolerance) | (test_data['TestSpeed'] < lower_tolerance)]
            
            # Mark outliers with a triangle
            sns.scatterplot(x='Time', y='TestSpeed', data=outliers, marker="^", color=line_color, s=100, ax=ax, zorder=3)
        
        plt.title('Test Speed over Time')
        plt.xticks(rotation=45)
        plt.tight_layout()
        st.pyplot(fig)

    # Filter the DataFrame for the relevant tests and selected log level
    filtered_df_tests = filtered_df[filtered_df['TestName'].isin(['test_communication', 'test_redirect'])]

    # Group by 'Time' and 'TestName' and calculate the mean of 'CommunicationSpeed'
    avg_comm_speed = filtered_df_tests.groupby(['Time', 'TestName'])['CommunicationSpeed'].mean().reset_index()

    # Streamlit UI
    st.title('Average Communication Speed Visualization')
    st.write('Displaying DataFrame:')
    st.dataframe(avg_comm_speed)

    # Use the right column for the second plot
    with col2:
        st.write('Average Communication Speed over Time for Log Level:', selected_log_level)
        fig, ax = plt.subplots(figsize=(8, 5))  # Adjust width and height as needed
        
        # Line plot for average 'CommunicationSpeed' over 'Time', separated by 'TestName'
        lineplot = sns.lineplot(x='Time', y='CommunicationSpeed', hue='TestName', data=avg_comm_speed, errorbar=None, marker="o", palette="tab10")

        # Get unique test names
        test_names = avg_comm_speed['TestName'].unique()
        
        # Define a multiplier for the standard deviation
        std_multiplier = 1  # Adjust as needed
        
        # Loop through each test name to add a shaded region and mark outliers for each line
        for i, test_name in enumerate(test_names):
            # Filter data for the current test name
            test_data = avg_comm_speed[avg_comm_speed['TestName'] == test_name]

            # Calculate the mean and standard deviation
            mean_comm_speed = test_data['CommunicationSpeed'].mean()
            std_comm_speed = test_data['CommunicationSpeed'].std()
            
            # Get the color of the current line
            line_color = lineplot.get_lines()[i].get_color()
            
            # Add shaded region
            ax.fill_between(test_data['Time'], mean_comm_speed - std_multiplier * std_comm_speed, mean_comm_speed + std_multiplier * std_comm_speed, color=line_color, alpha=0.2)  # Adjust alpha as needed
            
            # Identify outliers
            outliers = test_data[(test_data['CommunicationSpeed'] > mean_comm_speed + std_multiplier * std_comm_speed) | (test_data['CommunicationSpeed'] < mean_comm_speed - std_multiplier * std_comm_speed)]
            
            # Mark outliers with a triangle
            sns.scatterplot(x='Time', y='CommunicationSpeed', data=outliers, marker="^", color=line_color, s=100, ax=ax, zorder=3)
        
        plt.title('Average Communication Speed over Time')
        plt.xticks(rotation=45)
        plt.tight_layout()
        st.pyplot(fig)

elif option == 'Test Interface':

    # selected_tests = st.selectbox('Choose an Option', ['test_myscelium.py::test_communication', 'test_myscelium.py::test_error_returns', 'test_myscelium.py::test_mannangement', 'test_myscelium.py::test_messages', 'test_myscelium.py::test_redirect', 'all'])
    selected_tests = st.selectbox('Choose an Option', ['test_myscelium.py::test_communication', 'test_myscelium.py::test_mannangement', 'test_myscelium.py::test_redirect', 'all'])
    select_debug_level = st.selectbox('Choose an DEBUG level', ["DEBUG", "INFO", "WARN", "EXCEPTION",])

    def get_test_results(tests_to_run):

         # Set DEBUG_LEVEL as an environment variable
        os.environ['DEBUG_LEVEL'] = select_debug_level

        if tests_to_run == "all":
            tests_to_run = "test_myscelium.py"

        if not isinstance(tests_to_run, list):
            tests_to_run = [tests_to_run]
        pytest.main(tests_to_run + ["-v", "-s", "--junitxml=result.xml"])

        tree = ET.parse('result.xml')
        root = tree.getroot()

        results = {
            "errors": int(root.attrib.get('errors', 0)),
            "failures": int(root.attrib.get('failures', 0)),
            "skipped": int(root.attrib.get('skipped', 0)),
            "tests": int(root.attrib.get('tests', 0)),
        }
        return results

    if st.button("Run Selected Tests"):
        if selected_tests:
            results = get_test_results(selected_tests)
            st.write(f"Total Tests: {results.get('tests', 'N/A')}")
            st.write(f"Errors: {results.get('errors', 'N/A')}")
            st.write(f"Failures: {results.get('failures', 'N/A')}")
            st.write(f"Skipped: {results.get('skipped', 'N/A')}")
        else:
            st.warning("Please select at least one test to run.")
        
    # Optionally, delete the 'result.xml' file at the end if you don't need it anymore
    if os.path.exists("result.xml"):
        os.remove("result.xml")



    # Button 1 in the first column of the row
    # if cols[0].button('Button 1'):
    #     st.write('Button 1 was clicked!')

    # # Button 2 in the second column of the row
    # if cols[1].button('Button 2'):
    #     st.write('Button 2 was clicked!')
        
    pass

elif option == 'Logs Navigator':
    
    # Create an expander
    with st.expander("AI Analise", expanded=True):
        # Generate HTML with different colors for each log entry
        log_html = ""
        
        try:
            with open(os.path.join(TEMP_DIR, "auto_analise.txt"), "r") as file:
                content = file.read()
                log_html += f'<p style="color: #ffffff;">{content}</p>'
                file.close()
                
            st.markdown(log_html, unsafe_allow_html=True)      
        except:
            pass
    
    #> ------------------------------------------------------------------------------------------------------------------
    #> Events:
    
    with st.expander("Events", expanded=True):
        # Generate HTML with different colors for each log entry
        log_html = ""
        
        
        # columns=['ID', 'Unit', 'StepCompleted', 'EventType', 'EventKey', 'Time']
        events = pd.DataFrame.from_dict(Events_Manager(Unit="*", path="Logs").List_Events()) # Get All Events
        
        for index, row in events.iterrows():
            log_html += f'<p style="color: #ffffff; margin: 0px;">[{row["Unit"]}][{row["EventType"]}] - {row["StepCompleted"]}</p>'
        
        log_html += f'<p style="color: Transparent; margin: 15px;">  </p>'
        
        try:
            file.close()
            st.markdown(log_html, unsafe_allow_html=True)      
        except:
            pass
        
    #> ------------------------------------------------------------------------------------------------------------------
    #> Logs:
    
    # Example list of items
    # items = [f"Item {i}" for i in range(100)]  # Adjust the range for more or fewer items
    
    # Add a selection box at the top for log level
    selected_exibition_preset = st.selectbox('Show', options=["Filtered", "Full"])
    
    def get_last_directory(path):
        # Strip the trailing slash if it exists
        path = path.rstrip(os.path.sep)

        # Check if the last component of the path is a directory
        if os.path.isdir(path):
            return os.path.basename(path)
        else:
            # If the last component is a file, get the directory name
            return os.path.basename(os.path.dirname(path))
    
    all_logs_dict = {}
    
    for root, dirs, files in os.walk(TEMP_DIR):
        for file in files:
            if file == "logs.txt":
                with open(os.path.join(root, file), 'r') as log_file:
                    
                    last_dir_name = get_last_directory(root)
                    
                    logs = []
                    
                    for line in log_file:
                        try:
                            entry = json.loads(line.strip())
                            logs.append(entry)
                        except json.JSONDecodeError as e:
                            print(f"Error parsing JSON: {e}, in file: {file}, line: {line}")
                            
                    if last_dir_name == "Data":    
                        all_logs_dict["HOST"] = logs
                    if last_dir_name == "Client1Data":    
                        all_logs_dict["CLIENT1"] = logs
                    if last_dir_name == "Client2Data":    
                        all_logs_dict["CLIENT2"] = logs
    
    log_lines_dict = {}
    for owner, logs in all_logs_dict.items():
    
        for log in logs:
            
            if selected_exibition_preset == "Filtered":
            
                if log["log_msg"] == "Nothing in the schedule, skipping >>>":
                    continue
            
                if log["log_msg"] == "\nSchedule to process:\n[]\n":
                    continue
                
                # -> Remove C206 Ping and C207 Pong they are not necessary for this task and will overflow the model
                if 'C206' not in log["log_msg"] and 'C207' not in log["log_msg"]:
                    pass
                else:
                    continue
                
                if log["log_msg"] == 'Receive ping response pong conf!':
                    continue
                
                if 'No command received in ping, skipping' in log["log_msg"]:
                    continue
                
                if 'Nothing in schedule to send to host, so sending ping!' in log["log_msg"]:
                    continue
        
                if log["log_time"] == "":
                    continue
                
            else:
                pass
            
            log_lines_dict[log["log_time"]] = f"{owner}: " + log["log_msg"] + "\n"

    # Sorting the dictionary by its keys (timestamps)
    sorted_dict = {k: log_lines_dict[k] for k in sorted(log_lines_dict)}  
                            
                            
    # # Example list of strings
    # log_entries = ["INFO: This is an info message",
    #             "WARNING: This is a warning message",
    #             "ERROR: This is an error message",
    #             "INFO: Another info message"]

    # Color mapping for each label
    # color_map = {
    #     "INFO": "green",
    #     "WARNING": "orange",
    #     "ERROR": "red"
    # }

    # Color mapping for each label
    color_map = {
        "CLIENT1": "green",
        "CLIENT2": "orange",
        "HOST": "lightblue"
    }

    # Create an expander
    with st.expander("Log Entries", expanded=True):
        # Generate HTML with different colors for each log entry
        log_html = ""
        for entry in sorted_dict.values():
            label, content = entry.split(": ", 1)
            color = color_map.get(label, "black")  # Default to black if label is not in color_map
            log_html += f'<p style="color: {color}; margin: 0px;">{label}: {content}</p>'

        st.markdown(log_html, unsafe_allow_html=True)        
                            
    
#    # Create an expander
#     with st.expander("Scrollable List", expanded=True):
#         # Use markdown or HTML for a more compact layout
#         items_str = '\n'.join(f'- {item}' for item in sorted_dict.values())
#         st.markdown(items_str)
        
    pass
