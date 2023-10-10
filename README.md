## Requirements
- setuptools-rust 
- wheel

## How to compile:
-> go to E:\Zarpyon\Git\Repos\Myscelium\Myscelium
-> activate .venv
-> run: python setup.py bdist_wheel

## New way to install
cd \Myscelium
. /.venv/bin/activate
cd \Myscelium
then run `pip install -e .` (This guaranties that the binnaries will be installed too)

## To test:
cd \Myscelium
. /.venv/bin/activate
cd \Myscelium\tests
then:
-> run: `pytest -v -s ./test_myscelium.py`


## To see tests results 
cd \Myscelium\tests\History
then:
-> run: `streamlit run .\history_visualizer.py`