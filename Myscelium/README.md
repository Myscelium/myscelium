# Setup

## Requirements

- setuptools-rust
- wheel

## How to compile

-> go to E:\Zarpyon\Git\Repos\Myscelium\Myscelium
-> activate .venv
-> run: python setup.py bdist_wheel

## Debugging

-> To make a general tests using pytest got to Mysceliu/tests
-> Then run: `pytest -v -s .\test_myscelium.py` and wait, this is a process that auto gerenciate itself.

## Installation

-> run `py -m pip install --force-reinstall myscelium-1.3-cp310-cp310-win_amd64.whl`


### To calculate total of lines contained in the crate:

Total lines inserted / deleted every time:

```shell
git fame --recurse --excl=".fingerprint/*, .bin, .pyd" --loc ins --bytype
```

Total of lines that have survived:

```shell
git fame --recurse --excl=".fingerprint/*, .bin, .pyd" --loc surv --bytype
```

#### Repo content report 11 - 04 - 2024 : 01:09

Total .db: 6703
Total .db-journal: 83
Total .egg-info/PKG-INFO: 36
Total .egg-info/not-zip-safe: 1
Total .gitignore: 18
Total .lock: 2586
Total .md: 220
Total .py: 11891
Total .pyd: 30014
Total .rs: 2371
Total .toml: 90
Total .txt: 24
Total .whl: 10715
Total .zip: 4535
Total commits: 1001
Total ctimes: 14456
Total files: 85
Total loc: 69287

| Author           |   loc |   coms |   fils |  distribution    |
|:-----------------|------:|-------:|-------:|:-----------------|
| Cristian Camargo | 69287 |    904 |     85 | 100.0/90.3/100.0 |
| Poseidon         |     0 |     97 |      0 | 0.0/ 9.7/ 0.0    |