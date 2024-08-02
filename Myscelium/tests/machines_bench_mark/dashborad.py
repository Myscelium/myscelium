from dash import Dash, dcc, html
import dash_bootstrap_components as dbc
from dash.dependencies import Input, Output
import plotly.express as px
import pandas as pd
import sqlite3
import numpy as np
from scipy.stats import gaussian_kde

# Normalization functions
def z_score_normalization(times):
    mean = np.mean(times)
    std = np.std(times)
    if std == 0:
        return times - mean  # Avoid division by zero
    return (times - mean) / std

def min_max_normalization(times):
    min_time = np.min(times)
    max_time = np.max(times)
    if max_time == min_time:
        return times - min_time  # Avoid division by zero
    return (times - min_time) / (max_time - min_time)

def exponential_moving_average(times, alpha=0.3):
    ema_times = []
    ema = times[0]  # Start with the first value
    for time in times:
        ema = alpha * time + (1 - alpha) * ema
        ema_times.append(ema)
    return np.array(ema_times)

def log_transformation(times):
    return np.log(times + 1)  # Adding 1 to avoid log(0)

def select_normalization(times, benchmark_name):
    # Apply specific normalization methods based on benchmark type
    if 'Disk' in benchmark_name or 'PeakDiskWrite' in benchmark_name:
        # Use EMA or log normalization for disk benchmarks
        normalized_times = exponential_moving_average(times)
        if np.std(normalized_times) > np.std(log_transformation(times)):
            normalized_times = log_transformation(times)
        best_method = 'EMA or log'
    else:
        # Use z-score normalization for other benchmarks
        normalized_times = z_score_normalization(times)
        best_method = 'z-score'
    return best_method, normalized_times

def fetch_benchmark_data():
    conn = sqlite3.connect('test_results.db')
    df = pd.read_sql_query('SELECT * FROM benchmark_samples', conn)
    conn.close()
    return df

def compute_machine_score():
    df = fetch_benchmark_data()
    benchmarks = df['benchmark_name'].unique()
    
    scores = {}
    for benchmark in benchmarks:
        times = df[df['benchmark_name'] == benchmark]['time'].values
        if len(times) > 0:
            best_method, normalized_times = select_normalization(times, benchmark)
            scores[benchmark] = np.mean(normalized_times)
    
    # Aggregate scores (simple average for now)
    composite_score = np.mean(list(scores.values()))
    
    # Apply normalization to the composite score for stability
    composite_score_normalized = min_max_normalization(np.array([composite_score]))[0]  # Using min-max normalization

    # Debug output
    print("Scores per benchmark:", scores)
    print("Composite score:", composite_score)
    print("Normalized composite score:", composite_score_normalized)

    return composite_score_normalized

def analyze_clusters(times, num_clusters=3):
    if len(times) > 0:
        min_time = np.min(times)
        max_time = np.max(times)
        threshold = (max_time - min_time) / num_clusters
        clusters = []
        for i in range(num_clusters):
            lower_bound = min_time + i * threshold
            upper_bound = min_time + (i + 1) * threshold
            cluster = times[(times >= lower_bound) & (times < upper_bound)]
            clusters.append(cluster)
        return clusters
    return [times]

app = Dash(__name__, external_stylesheets=[dbc.themes.BOOTSTRAP])

app.layout = dbc.Container([
    dbc.Row([
        dbc.Col(html.H1("Benchmark Dashboard"), className="mb-2")
    ]),
    dbc.Row([
        dbc.Col([
            dcc.Dropdown(
                id='benchmark-dropdown',
                options=[
                    {'label': 'CPU', 'value': 'CPU'},
                    {'label': 'Memory', 'value': 'Memory'},
                    {'label': 'Disk', 'value': 'Disk'},
                    {'label': 'IO', 'value': 'IO'},
                    {'label': 'Peak Disk Write', 'value': 'PeakDiskWrite'},
                    {'label': 'Machine Score', 'value': 'MachineScore'}
                ],
                value='CPU',
                clearable=False
            )
        ], width=4),
        dbc.Col([
            dbc.Checkbox(
                id='cluster-checkbox',
                label='Show Clusters',
                value=False
            )
        ], width=2)
    ]),
    dbc.Row([
        dbc.Col([
            dcc.Graph(id='benchmark-graph')
        ])
    ])
])

@app.callback(
    Output('benchmark-graph', 'figure'),
    [Input('benchmark-dropdown', 'value'), Input('cluster-checkbox', 'value')]
)
def update_graph(benchmark_name, show_clusters):
    if benchmark_name == 'MachineScore':
        score = compute_machine_score()
        fig = px.bar(x=['Machine'], y=[score], labels={'x': 'Machine', 'y': 'Normalized Score'})
        fig.update_layout(
            title='Machine Composite Score',
            yaxis_title='Normalized Score',
            xaxis_title='Machine'
        )
    else:
        df = fetch_benchmark_data()
        filtered_df = df[df['benchmark_name'] == benchmark_name]
        times = filtered_df['time'].values

        if len(times) > 0:  # Ensure times is not empty
            if show_clusters:
                clusters = analyze_clusters(times)
                fig = px.histogram(x=times, nbins=30, histnorm='probability density')  # Initialize with full data
                for i, cluster in enumerate(clusters, 1):
                    if len(cluster) > 0:
                        best_method, normalized_times = select_normalization(cluster, benchmark_name)
                        kde = gaussian_kde(normalized_times)
                        x_range = np.linspace(min(normalized_times), max(normalized_times), 1000)
                        kde_values = kde(x_range)
                        
                        # Add histogram and KDE for each cluster
                        fig.add_trace(px.histogram(x=normalized_times, nbins=30, histnorm='probability density').data[0])
                        fig.add_scatter(
                            x=x_range,
                            y=kde_values,
                            mode='lines',
                            name=f'Cluster {i} Density (Normalization: {best_method})'
                        )
            else:
                # Automatically select the best normalization method for the full dataset
                best_method, normalized_times = select_normalization(times, benchmark_name)
                kde = gaussian_kde(normalized_times)
                x_range = np.linspace(min(normalized_times), max(normalized_times), 1000)
                kde_values = kde(x_range)
                
                # Create histogram with full dataset
                fig = px.histogram(x=normalized_times, nbins=30, histnorm='probability density')
                fig.add_scatter(
                    x=x_range,
                    y=kde_values,
                    mode='lines',
                    name='Density'
                )
        else:
            fig = px.histogram()  # Initialize an empty histogram

        fig.update_layout(
            title=f'{benchmark_name} Benchmark Results' + (' with Clusters' if show_clusters else ''),
            xaxis_title='Normalized Time' if len(times) > 0 else 'Time (seconds)',
            yaxis_title='Density'
        )

    return fig

if __name__ == '__main__':
    app.run_server(debug=True)
