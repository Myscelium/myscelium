from setuptools import setup
from setuptools_rust import Binding, RustExtension


setup(
    name="myscelium",
    version="1.1",
    rust_extensions=[RustExtension("myscelium.myscelium_engine", path="rust/Cargo.toml", binding=Binding.PyO3)],
    packages=["myscelium"],
    zip_safe=False,
    description="Myscelium is a library designed to simplify the creation of large, interconnected networks of Python scripts through sockets. These scripts harmoniously run on different machines, yet synchronize with one another. Much like the hyphae in mycelium interconnect various mushrooms, the socket bridges in the Myscelium library interlink multiple machines, synchronizing commands and harnessing the power of modularity.",
    long_description=open('README.md').read(),
    long_description_content_type="text/markdown",
    author='Cristian Camargo Filho',
    author_email='ccf@cdone.com.br',
    classifiers=[
        # Classifiers for your package
        "Programming Language :: Python",
        "Programming Language :: Python :: 3",
        "Programming Language :: Python :: 3.6",
        "Programming Language :: Python :: 3.7",
        "Programming Language :: Python :: 3.8",
        "Programming Language :: Python :: 3.9",
    ],
)