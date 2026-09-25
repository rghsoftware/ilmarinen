from greeting import greet


# specscore:verifies feature/greeting#ac:greets-named-user
def test_greets_named_user() -> None:
    assert greet("Ada") == "Hello, Ada!"
