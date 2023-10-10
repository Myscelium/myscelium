import pandas as pd
import seaborn as sns
import matplotlib.pyplot as plt
import streamlit as st
import datetime
from history_controler import History_Mannanger

# Set the page config to use the full width
st.set_page_config(layout='wide')

pd_dict_df = History_Mannanger().list_history()
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

# Filter the data based on the selected log level
filtered_df = df[df['LogLevel'] == selected_log_level]

# Create columns
col1, col2 = st.columns([1,1])


# Use the left column for the first plot
with col1:
    st.write('Test Speed over Time for Log Level:', selected_log_level)
    fig, ax = plt.subplots(figsize=(8, 5))  # Adjust width and height as needed
    
    # Line plot for 'TestSpeed' over 'Time', separated by 'TestName'
    lineplot = sns.lineplot(x='Time', y='TestSpeed', hue='TestName', data=filtered_df, ci=None, marker="o", palette="tab10")

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
    lineplot = sns.lineplot(x='Time', y='CommunicationSpeed', hue='TestName', data=avg_comm_speed, ci=None, marker="o", palette="tab10")

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

