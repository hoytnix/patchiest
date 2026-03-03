from setuptools import setup, find_packages

setup(
    name="patchiest",
    version="0.1.0",
    packages=find_packages(),
    install_requires=[
        "pydantic>=2.0.0",
        "wasmtime",
        "esbuild-py",
        "httpx",
        "uvicorn",
        "starlette",
    ],
    description="The Patchiest Protocol: Surgical AST mutations for agentic code evolution.",
    author="Master Alchemist",
)
