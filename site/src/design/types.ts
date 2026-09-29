// The token names component props accept, so the type checker enforces the
// scales: `gap={5}` compiles, `gap="13px"` does not.
export type Space = 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10;
export type TextSize = 'xs' | 'sm' | 'base' | 'md' | 'lg' | 'xl' | '2xl' | '3xl';
export type Weight = 'regular' | 'medium' | 'bold';
export type MarkKey = 'todo' | 'doing' | 'done' | 'dropped' | 'later' | 'ask' | 'answer' | 'group' | 'graft';
export type Columns = 1 | 2 | 3 | 4;
