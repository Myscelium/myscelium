# Instructions

## Requirements

- setuptools-rust
- wheel

## How to compiled

-> go to E:\Zarpyon\Git\Repos\Myscelium\Myscelium
-> activate .venv
-> run: python setup.py bdist_wheel

## New way to installed

cd \Myscelium
. /.venv/bin/activate

cd \Myscelium
then run `pip install -e .`
-> (This guaranties that the binnaries will be installed too).

## Installation

-> run `py -m pip install --force-reinstall myscelium-1.3-cp310-cp310-win_amd64.whl`

## To test

cd \Myscelium
. /.venv/bin/activate
cd \Myscelium\tests
then:
-> run: `pytest -v -s ./test_myscelium.py`

---

## To see tests results

cd \Myscelium\tests\History
then:
-> run: `streamlit run .\interface.py`
