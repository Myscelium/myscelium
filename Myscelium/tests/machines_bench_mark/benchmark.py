# SPDX-License-Identifier: MPL-2.0
# Copyright © 2021-2026 Cristian Camargo Filho

import time
import numpy as np
import os
import sqlite3
import tempfile
from tqdm import tqdm
from concurrent.futures import ThreadPoolExecutor, as_completed


def benchmark_results_path():
    configured = os.environ.get("MYSCELIUM_BENCHMARK_DB")
    if configured:
        path = os.path.expanduser(configured)
        if not os.path.isabs(path):
            raise ValueError("MYSCELIUM_BENCHMARK_DB must be absolute")
        return os.path.abspath(path)
    return os.path.join(
        os.path.expanduser("~"), ".cache", "myscelium", "benchmarks", "test_results.db"
    )


def temporary_benchmark_file(prefix, suffix):
    descriptor, path = tempfile.mkstemp(prefix=prefix, suffix=suffix)
    os.close(descriptor)
    return path

def run_benchmark_multiple_times(benchmark_func, iterations=10, warmup=True):
    if warmup:
        benchmark_func()
    
    times = []
    for _ in tqdm(range(iterations), desc=benchmark_func.__name__, leave=False):
        start_time = time.time()
        benchmark_func()
        end_time = time.time()
        times.append(end_time - start_time)
    
    return times  # Return all times for visualization

def cpu_benchmark_single_core():
    result = 0
    for i in range(1, 1000000):
        result += (i ** 0.5) ** 2

def cpu_benchmark_multicore(workers=4):
    def worker_task(start, end):
        result = 0
        for i in range(start, end):
            result += (i ** 0.5) ** 2
        return result

    chunk_size = 1000000 // workers
    with ThreadPoolExecutor(max_workers=workers) as executor:
        futures = [executor.submit(worker_task, i * chunk_size, (i + 1) * chunk_size) for i in range(workers)]
        results = [future.result() for future in as_completed(futures)]
    return sum(results)

def memory_benchmark():
    array_size = 10000000
    array = np.arange(array_size, dtype=np.float64)
    array *= 2

def disk_benchmark():
    file_path = temporary_benchmark_file('myscelium-disk-', '.tmp')
    try:
        with open(file_path, 'wb') as f:
            f.write(os.urandom(500000000))  # Write 500MB
        with open(file_path, 'rb') as f:
            f.read()
    finally:
        os.remove(file_path)

def io_benchmark():
    file_path = temporary_benchmark_file('myscelium-io-', '.tmp')
    try:
        with open(file_path, 'w') as f:
            for _ in range(100000):
                f.write('This is a test.\n')
    finally:
        os.remove(file_path)

def peak_disk_write_benchmark(file_size_mb=500):
    data = os.urandom(file_size_mb * 1024 * 1024)  # Generate large data block
    file_path = temporary_benchmark_file('myscelium-peak-disk-', '.tmp')
    
    start_time = time.time()
    with open(file_path, 'wb') as f:
        f.write(data)
        f.flush()  # Ensure all data is written
        os.fsync(f.fileno())  # Flush OS buffer to disk
    end_time = time.time()
    
    os.remove(file_path)  # Clean up
    
    elapsed_time = end_time - start_time
    throughput = file_size_mb / elapsed_time  # MB/s
    return throughput

def run_peak_disk_write_benchmark(iterations=5, file_size_mb=500):
    throughputs = []
    for _ in tqdm(range(iterations), desc="Peak Disk Write Benchmark", leave=False):
        throughput = peak_disk_write_benchmark(file_size_mb)
        throughputs.append(throughput)
    return throughputs

def save_benchmark_results(benchmark_name, times):
    results_path = benchmark_results_path()
    os.makedirs(os.path.dirname(results_path), exist_ok=True)
    conn = sqlite3.connect(results_path)
    cursor = conn.cursor()
    cursor.execute('''
        CREATE TABLE IF NOT EXISTS benchmark_samples (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            benchmark_name TEXT,
            time REAL
        )
    ''')
    for time in times:
        cursor.execute('''
            INSERT INTO benchmark_samples (benchmark_name, time)
            VALUES (?, ?)
        ''', (benchmark_name, time))
    conn.commit()
    conn.close()
    
ITERATIONS = 100000

# Example usage
cpu_times = run_benchmark_multiple_times(lambda: cpu_benchmark_multicore(workers=os.cpu_count()), ITERATIONS)
memory_times = run_benchmark_multiple_times(memory_benchmark, ITERATIONS)
disk_times = run_benchmark_multiple_times(disk_benchmark, ITERATIONS)
io_times = run_benchmark_multiple_times(io_benchmark, ITERATIONS)
peak_disk_write_throughputs = run_peak_disk_write_benchmark(ITERATIONS)


save_benchmark_results('CPU', cpu_times)
save_benchmark_results('Memory', memory_times)
save_benchmark_results('Disk', disk_times)
save_benchmark_results('IO', io_times)
save_benchmark_results('PeakDiskWrite', peak_disk_write_throughputs)

print("Benchmarking complete. Results saved to the database.")
