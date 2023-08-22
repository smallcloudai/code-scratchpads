from setuptools import setup

setup(
    name="code-scratchpads",
    packages=["code_scratchpads", "code_scratchpads.scratchpads_code_completion"],
    version="0.0.1",
    install_requires=[
        "aiohttp",
        "uvicorn",
        "fastapi",
        "termcolor",
        "transformers"
    ],
)
