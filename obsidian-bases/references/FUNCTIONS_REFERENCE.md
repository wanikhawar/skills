# Bases Formula Reference

Use this as a compact working reference, not as a substitute for the current [official function documentation](https://obsidian.md/help/bases/functions).

## Global functions

| Function | Purpose |
| --- | --- |
| `date(value)` | Parse a date string. |
| `duration(value)` | Parse a duration for date/duration arithmetic. |
| `now()` | Current date and time. |
| `today()` | Current date at midnight. |
| `if(condition, yes, no?)` | Conditional result. |
| `number(value)` | Convert a compatible value to a number. |
| `list(value)` | Preserve a list or wrap one value as a list. |
| `file(path)` | Resolve a file object. |
| `link(path, display?)` | Render an internal or external link. |
| `image(path)` | Render an image value. |
| `icon(name)` | Render a Lucide icon. |
| `html(value)` | Render trusted HTML. Do not place untrusted extracted HTML here. |
| `escapeHTML(value)` | Escape text before incorporating it into HTML. |
| `min(...)`, `max(...)` | Select numeric minimum or maximum. |
| `random()` | Random number from 0 to 1; refreshes when the view loads. |

## Date arithmetic

```yaml
formulas:
  created_date: 'file.ctime.date()'
  created_label: 'file.ctime.format("YYYY-MM-DD")'
  recent: 'file.mtime > now() - "1 week"'
  age_days: '((now() - file.ctime) / 86400000).floor()'
  due_days: 'if(due, ((date(due) - today()) / 86400000).round(0), "")'
  next_review: 'today() + "7d"'
```

Subtracting two dates yields a millisecond number. Do not use `.days`, `.hours`, `.minutes`, `.seconds`, or `.milliseconds` on the result.

`duration()` is useful when a duration must be multiplied before adding it to a date:

```yaml
formulas:
  next_window: 'now() + (duration("6h") * 2)'
```

Keep the duration on the left of scalar multiplication.

## Common value methods

| Type | Useful fields/functions |
| --- | --- |
| Date | `.year`, `.month`, `.day`, `.hour`, `.date()`, `.format()`, `.time()`, `.relative()` |
| String | `.length`, `.contains()`, `.startsWith()`, `.endsWith()`, `.lower()`, `.replace()`, `.slice()`, `.split()`, `.trim()` |
| Number | `.abs()`, `.ceil()`, `.floor()`, `.round()`, `.toFixed()` |
| List | `.length`, `.contains()`, `.filter()`, `.map()`, `.flat()`, `.join()`, `.slice()`, `.sort()`, `.unique()` |
| File | `.asLink()`, `.hasLink()`, `.hasTag()`, `.hasProperty()`, `.inFolder()` |
| Link | `.asFile()`, `.linksTo()` |
| Object | `.keys()`, `.values()`, `.isEmpty()` |
| RegExp | `.matches(value)` |

Most value types also support `.isEmpty()`, `.isTruthy()`, `.isType()`, or `.toString()` where applicable.

## Safe patterns

Guard an optional property before conversion or arithmetic:

```yaml
formulas:
  finished_year: 'if(finished, date(finished).year, "")'
```

Use `list()` when a property may be either one value or a list:

```yaml
filters:
  and:
    - 'list(exams).contains("GATE")'
```

Use `escapeHTML()` before `html()` when text originated outside the vault or from an uncontrolled property.
