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

# Create columns
col1, col2 = st.columns([1,1])

# Use the center column for the first plot
with col1:
    st.write('Line Plot: Test Speed over Time')
    fig, ax = plt.subplots(figsize=(8, 5))  # Adjust width and height as needed
    
    # Line plot for 'TestSpeed' over 'Time', separated by 'TestName'
    sns.lineplot(x='Time', y='TestSpeed', hue='TestName', data=df, ci=None, marker="o")

    # Scatter plot on top of the line plot to color points based on 'TestStatus'
    sns.scatterplot(x='Time', y='TestSpeed', hue='TestStatus', style='TestName', data=df, palette={'FAILED':'red', 'PASSED':'green'}, s=100, legend=False)

    plt.title('Test Speed over Time')
    plt.xticks(rotation=45)
    plt.tight_layout()
    st.pyplot(fig)

# Filter the DataFrame for the relevant tests
filtered_df = df[df['TestName'].isin(['test_communication', 'test_redirect'])]

# Group by 'Time' and 'TestName' and calculate the mean of 'CommunicationSpeed'
avg_comm_speed = filtered_df.groupby(['Time', 'TestName'])['CommunicationSpeed'].mean().reset_index()

# Streamlit UI
st.title('Average Communication Speed Visualization')
st.write('Displaying DataFrame:')
st.dataframe(avg_comm_speed)

# Use the center column for the second plot
with col2:
    st.write('Line Plot: Average Communication Speed over Time')
    fig, ax = plt.subplots(figsize=(8, 5))  # Adjust width and height as needed
    
    # Line plot for average 'CommunicationSpeed' over 'Time', separated by 'TestName'
    sns.lineplot(x='Time', y='CommunicationSpeed', hue='TestName', data=avg_comm_speed, ci=None, marker="o")

    plt.title('Average Communication Speed over Time')
    plt.xticks(rotation=45)
    plt.tight_layout()
    st.pyplot(fig)

