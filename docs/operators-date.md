---
layout: page
title: Date Functions
permalink: /operators-date/
---

# Date Functions

Date and time manipulation operators.

## `today` / `TODAY` - Current Date

Returns the current date at midnight (00:00:00).

### Syntax
```json
{"today": null}
{"TODAY": null}
```

### Return Type
String - Current date in ISO 8601 format (YYYY-MM-DDTHH:MM:SS.SSSZ)

### Examples

**Get today:**
```json
{"today": null}  // → "2024-01-15T00:00:00.000Z"
```

**Calculate days since:**
```json
{"days": [{"today": null}, {"var": "startDate"}]}
```

**Age calculation:**
```json
{"-": [
  {"year": {"today": null}},
  {"var": "birthYear"}
]}
```

---

## `now` / `NOW` - Current DateTime

Returns the current date and time with full precision.

### Syntax
```json
{"now": null}
{"NOW": null}
```

### Return Type
String - Current datetime in RFC3339 format

### Examples

**Get current time:**
```json
{"now": null}  // → "2024-01-15T14:30:45.123Z"
```

**Timestamp logging:**
```json
{"cat": ["Last updated: ", {"now": null}]}
```

---

## `year` / `YEAR` - Extract Year

Extracts the year component from a date.

### Syntax
```json
{"year": date}
{"YEAR": date}
```

### Parameters
- **date** (string): Date string in ISO format

### Return Type
Number - Year (e.g., 2024)

### Examples

**Extract year:**
```json
{"year": "2024-06-15"}                    // → 2024
{"YEAR": "1990-03-20T10:30:00Z"}          // → 1990
```

**With variable:**
```json
// Data: {"birthdate": "1990-05-15"}
{"year": {"var": "birthdate"}}            // → 1990
```

**Calculate age:**
```json
{"-": [
  {"year": {"today": null}},
  {"year": {"var": "birthdate"}}
]}
```

**Invalid date:**
```json
{"year": "not-a-date"}                    // → null
{"year": null}                            // → null
```

---

## `month` / `MONTH` - Extract Month

Extracts the month component from a date (1-12).

### Syntax
```json
{"month": date}
{"MONTH": date}
```

### Parameters
- **date** (string): Date string

### Return Type
Number - Month (1 = January, 12 = December)

### Examples

**Extract month:**
```json
{"month": "2024-06-15"}                   // → 6
{"MONTH": "1990-12-25T10:30:00Z"}         // → 12
```

**Check if birthday month:**
```json
{"==": [
  {"month": {"today": null}},
  {"month": {"var": "birthdate"}}
]}
```

---

## `day` / `DAY` - Extract Day

Extracts the day component from a date (1-31).

### Syntax
```json
{"day": date}
{"DAY": date}
```

### Parameters
- **date** (string): Date string

### Return Type
Number - Day of month (1-31)

### Examples

**Extract day:**
```json
{"day": "2024-06-15"}                     // → 15
{"DAY": "1990-12-25T10:30:00Z"}           // → 25
```

**Check if birthday:**
```json
{"and": [
  {"==": [{"month": {"today": null}}, {"month": {"var": "birthdate"}}]},
  {"==": [{"day": {"today": null}}, {"day": {"var": "birthdate"}}]}
]}
```

---

## `date` / `DATE` - Construct Date

Creates a date from year, month, and day components.

### Syntax
```json
{"date": [year, month, day]}
{"DATE": [year, month, day]}
```

### Parameters
- **year** (number): Year (e.g., 2024)
- **month** (number): Month (1-12)
- **day** (number): Day (1-31)

### Return Type
String - ISO 8601 date string

### Examples

**Create date:**
```json
{"date": [2024, 12, 25]}                  // → "2024-12-25T00:00:00.000Z"
{"DATE": [1990, 5, 15]}                   // → "1990-05-15T00:00:00.000Z"
```

**From variables:**
```json
// Data: {"y": 2024, "m": 6, "d": 15}
{"date": [
  {"var": "y"},
  {"var": "m"},
  {"var": "d"}
]}
```

**Date normalization:**
```json
// Month 13 wraps to next year
{"date": [2023, 13, 1]}                   // → "2024-01-01T00:00:00.000Z"

// Day 32 wraps to next month
{"date": [2024, 1, 32]}                   // → "2024-02-01T00:00:00.000Z"
```

---

## `days` / `DAYS` - Days Between Dates

Calculates the number of days between two dates.

### Syntax
```json
{"days": [end_date, start_date]}
{"DAYS": [end_date, start_date]}
```

### Parameters
- **end_date** (string): End date
- **start_date** (string): Start date

### Return Type
Number - Number of days (can be negative if end < start)

### Examples

**Days between dates:**
```json
{"days": ["2024-01-20", "2024-01-15"]}    // → 5
{"DAYS": ["2024-01-15", "2024-01-20"]}    // → -5
```

**Days until event:**
```json
{"days": [{"var": "eventDate"}, {"today": null}]}
```

**Days since birth:**
```json
{"days": [{"today": null}, {"var": "birthdate"}]}
```

**Age in days:**
```json
// Data: {"birth": "1990-03-15"}
{"DAYS": [{"today": null}, {"var": "birth"}]}  // → ~12,000+ days
```

---

## `dateformat` / `DATEFORMAT` - Format Date

Formats a date string using predefined or custom formats.

### Syntax
```json
{"dateformat": [date, format]}
{"DATEFORMAT": [date]}
```

### Parameters
- **date** (string): Date to format
- **format** (string, optional): Format specifier (default: ISO)

### Return Type
String - Formatted date

### Predefined Formats

- **`"iso"`** or default - `"2024-01-15"`
- **`"short"`** - `"01/15/2024"` (US format)
- **`"long"`** - `"January 15, 2024"`
- **`"eu"`** - `"15/01/2024"` (European format)

### Custom Format (strftime)

Use strftime format codes:
- **%Y** - Year (4 digits)
- **%m** - Month (01-12)
- **%d** - Day (01-31)
- **%B** - Month name (January)
- **%b** - Month abbr (Jan)
- **%A** - Weekday (Monday)
- **%a** - Weekday abbr (Mon)

### Examples

**Predefined formats:**
```json
// Data: {"date": "2024-01-15"}
{"DATEFORMAT": [{"var": "date"}]}              // → "2024-01-15"
{"dateformat": [{"var": "date"}, "short"]}     // → "01/15/2024"
{"dateformat": [{"var": "date"}, "long"]}      // → "January 15, 2024"
{"DATEFORMAT": [{"var": "date"}, "eu"]}        // → "15/01/2024"
```

**Custom formats:**
```json
{"dateformat": ["2024-01-15", "%Y/%m/%d"]}     // → "2024/01/15"
{"dateformat": ["2024-01-15", "%B %d, %Y"]}    // → "January 15, 2024"
{"dateformat": ["2024-01-15", "%A, %b %d"]}    // → "Monday, Jan 15"
```

**Display date:**
```json
{"cat": [
  "Date: ",
  {"DATEFORMAT": [{"today": null}, "long"]}
]}
```

---

## `yearfrac` / `YEARFRAC` - Year Fraction

Calculates the fraction of a year between two dates.

### Syntax
```json
{"yearfrac": [start_date, end_date, basis]}
{"YEARFRAC": [start_date, end_date]}
```

### Parameters
- **start_date** (string): Start date
- **end_date** (string): End date
- **basis** (number, optional): Day count basis (default: 0)
  - 0: US (NASD) 30/360
  - 1: Actual/actual
  - 2: Actual/360
  - 3: Actual/365
  - 4: European 30/360

### Return Type
Number - Fractional years

### Examples

**Basic yearfrac:**
```json
{"YEARFRAC": ["1990-03-15", "2023-07-10"]}     // → ~33.32
```

**Age in years (decimal):**
```json
{"yearfrac": [{"var": "birthdate"}, {"today": null}]}
```

**Calculate interest:**
```json
{"*": [
  {"var": "principal"},
  {"var": "rate"},
  {"yearfrac": [{"var": "startDate"}, {"var": "endDate"}]}
]}
```

---

## `datedif` / `DATEDIF` - Date Difference

Calculates the difference between two dates in specified units.

### Syntax
```json
{"datedif": [start_date, end_date, unit]}
{"DATEDIF": [start_date, end_date, unit]}
```

### Parameters
- **start_date** (string): Start date
- **end_date** (string): End date
- **unit** (string): Unit of measurement
  - **"Y"** - Complete years
  - **"M"** - Complete months
  - **"D"** - Days
  - **"YM"** - Months ignoring years
  - **"YD"** - Days ignoring years
  - **"MD"** - Days ignoring months and years

### Return Type
Number - Difference in specified units

### Examples

**Years between dates:**
```json
{"DATEDIF": ["1990-03-15", "2023-07-10", "Y"]}  // → 33
```

**Months between dates:**
```json
{"datedif": ["1990-03-15", "2023-07-10", "M"]}  // → 399
```

**Days between dates:**
```json
{"DATEDIF": ["2024-01-01", "2024-12-31", "D"]}  // → 365
```

**Age display:**
```json
{"cat": [
  {"DATEDIF": [{"var": "birthdate"}, {"today": null}, "Y"]},
  " years, ",
  {"DATEDIF": [{"var": "birthdate"}, {"today": null}, "YM"]},
  " months"
]}
// → "33 years, 4 months"
```

**Months in current year:**
```json
{"datedif": [{"var": "startDate"}, {"var": "endDate"}, "YM"]}
```

---

## Complex Examples

### Calculate Age
```json
{
  "if": [
    {">=": [
      {"month": {"today": null}},
      {"month": {"var": "birthdate"}}
    ]},
    {"-": [
      {"year": {"today": null}},
      {"year": {"var": "birthdate"}}
    ]},
    {"-": [
      {"-": [
        {"year": {"today": null}},
        {"year": {"var": "birthdate"}}
      ]},
      1
    ]}
  ]
}
```

### Days Until Birthday
```json
{"days": [
  {"date": [
    {"year": {"today": null}},
    {"month": {"var": "birthdate"}},
    {"day": {"var": "birthdate"}}
  ]},
  {"today": null}
]}
```

### Is Date in Range
```json
{"and": [
  {">=": [
    {"days": [{"var": "checkDate"}, {"var": "startDate"}]},
    0
  ]},
  {"<=": [
    {"days": [{"var": "checkDate"}, {"var": "endDate"}]},
    0
  ]}
]}
```

### Format Display Date
```json
{"if": [
  {"<": [
    {"days": [{"today": null}, {"var": "eventDate"}]},
    7
  ]},
  {"cat": [
    {"days": [{"today": null}, {"var": "eventDate"}]},
    " days ago"
  ]},
  {"DATEFORMAT": [{"var": "eventDate"}, "long"]}
]}
```

### Subscription Expiry Check
```json
{"and": [
  {">=": [
    {"days": [{"var": "expiryDate"}, {"today": null}]},
    0
  ]},
  {"<": [
    {"days": [{"var": "expiryDate"}, {"today": null}]},
    30
  ]}
]}
// Returns true if expires in next 30 days
```

### Quarter Calculator
```json
{"ceiling": [
  {"/": [{"month": {"var": "date"}}, 3]},
  1
]}
// Returns quarter number (1-4)
```

---

## Date Parsing

Supported date formats:
- **ISO 8601**: `"2024-01-15"`, `"2024-01-15T10:30:00Z"`
- **RFC 3339**: `"2024-01-15T10:30:00.000Z"`
- **Short form**: `"2024-01-15"`

Invalid dates return `null`:
```json
{"year": "not-a-date"}     // → null
{"month": ""}              // → null
{"day": null}              // → null
```

---

## Best Practices

1. **Always use ISO format** for portability
   ```json
   "2024-01-15"  // ✓ ISO format
   "01/15/2024"  // ✗ Ambiguous
   ```

2. **Handle null dates gracefully**
   ```json
   {"ifnull": [{"year": {"var": "date"}}, 0]}
   ```

3. **Use DATEDIF for age** calculations
   ```json
   {"DATEDIF": [birthdate, today, "Y"]}
   ```

4. **Format dates for display** using DATEFORMAT
   ```json
   {"DATEFORMAT": [date, "long"]}
   ```

5. **Calculate durations** with days or yearfrac
   ```json
   {"days": [endDate, startDate]}
   ```

---

## Related Operators

- **[Arithmetic Operators](operators-arithmetic.md)** - Calculate with dates
- **[Comparison Operators](operators-comparison.md)** - Compare dates
- **[String Operators](operators-string.md)** - Format date strings

---

## Performance Notes

- **Date parsing** cached during compilation where possible
- **Timezone handling** assumes UTC for consistency
- **Year calculations** use chrono library for accuracy
