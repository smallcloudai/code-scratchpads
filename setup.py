from setuptools import setup, find_packages


setup(
    name="code-scratchpads",
    packages=find_packages(),
    version="0.0.1",
    install_requires=[
        "aiohttp",
        "uvicorn",
        "fastapi",
        "termcolor",
        "transformers"
    ],
)
