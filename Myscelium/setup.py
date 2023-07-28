from setuptools import setup
from setuptools_rust import Binding, RustExtension

setup(
    name="myscelium",
    version="1.0",
    rust_extensions=[RustExtension("myscelium.myscelium_engine", path="rust/Cargo.toml", binding=Binding.PyO3)],
    packages=["myscelium"],
    zip_safe=False,
)