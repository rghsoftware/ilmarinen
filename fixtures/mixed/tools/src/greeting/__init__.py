"""Fixture: the smallest Python package that exercises the Ilmarinen gates."""


# specscore:implements feature/greeting#req:greet-by-name
def greet(name: str) -> str:
    return f"Hello, {name}!"
