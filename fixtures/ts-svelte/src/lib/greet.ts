// Fixture: the smallest Svelte + TypeScript project that exercises the Ilmarinen gates.

// specscore:implements feature/greeting#req:greet-by-name
export function greet(name: string): string {
  return `Hello, ${name}!`;
}
