from setuptools import setup

setup(
    name="code-scratchpads",
    py_modules=["code_scratchpads"],
    version="0.0.1",
    install_requires=[
        "uvicorn",
        "fastapi",
    ],
)
