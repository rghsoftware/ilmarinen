import { expect, test } from "vitest";
import { greet } from "./greet";

// specscore:verifies feature/greeting#ac:greets-named-user
test("greets a named user", () => {
  expect(greet("Ada")).toBe("Hello, Ada!");
});
