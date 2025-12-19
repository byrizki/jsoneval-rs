---
layout: default
title: Date Functions
---

# Date Functions

Date and time manipulation operators.

## Overview

Date functions provide comprehensive capabilities for working with dates, times, and intervals. These operators handle date parsing, formatting, arithmetic, and comparisons, making it easy to implement age calculations, date validation, and time-based logic.

### Common Use Cases

- **Age Calculation**: Determine years, months, or days since a date with `DATEDIF`
- **Date Validation**: Check if dates fall within ranges or meet criteria
- **Date Arithmetic**: Calculate days between dates, add/subtract intervals
- **Date Formatting**: Display dates in user-friendly formats with `DATEFORMAT`
- **Business Logic**: Implement expiry checks, subscription renewals, eligibility rules
- **Financial Calculations**: Calculate interest periods with `YEARFRAC`

### Date Function Categories

1. **Current Date/Time**: `today`, `now` - Get current timestamps
2. **Extraction**: `year`, `month`, `day` - Extract date components
3. **Construction**: `date` - Build dates from components
4. **Arithmetic**: `days`, `DATEDIF` - Calculate differences
5. **Formatting**: `DATEFORMAT` - Display dates in various formats
6. **Financial**: `YEARFRAC` - Calculate fractional years for interest

### Date Format Standards

JSON-Eval-RS uses **ISO 8601** format for dates:
- **Date only**: `"2024-01-15"`
- **Datetime**: `"2024-01-15T14:30:00Z"`
- **With milliseconds**: `"2024-01-15T14:30:00.123Z"`

**Why ISO 8601?**
- Unambiguous (no confusion between MM/DD vs DD/MM)
- Sortable alphabetically
- Widely supported across systems
- Timezone-aware with optional offset

### Timezone Awareness

All date/time operators respect the configured timezone offset:
- Default: UTC (Coordinated Universal Time)
- Configurable via `set_timezone_offset(minutes)`
- When set, all operations adjust to local timezone
- See **Timezone Configuration** section for details

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

## Troubleshooting

### Issue: Date parsing returns null

**Problem:** Date extraction operators (`year`, `month`, `day`) return null.

**Common causes:**
1. **Invalid date format** - Non-ISO format provided
2. **Malformed date string** - Syntax errors in date
3. **Null or undefined input** - Missing date value

**Solutions:**
```json
// ❌ Non-ISO formats might fail
{"year": "01/15/2024"}  // Ambiguous format
{"year": "15-01-2024"}  // Wrong separator

// ✅ Use ISO 8601 format
{"year": "2024-01-15"}  // → 2024

// ✅ Validate before extracting
{"if": [
  {"!==": [{"var": "date"}, null]},
  {"year": {"var": "date"}},
  null
]}

// ✅ Provide default value
{"ifnull": [{"year": {"var": "date"}}, 2024]}
```

### Issue: Age calculation off by one

**Problem:** Age calculation shows 32 years when should be 33, or vice versa.

**Common causes:**
1. **Birthday hasn't occurred yet this year** - Need to check month/day
2. **Simple year subtraction** - Doesn't account for birthday

**Solutions:**
```json
// ❌ Wrong - doesn't account for birthday
{"-": [
  {"year": {"today": null}},
  {"year": {"var": "birthdate"}}
]}

// ✅ Correct - use DATEDIF
{"DATEDIF": [{"var": "birthdate"}, {"today": null}, "Y"]}

// ✅ Manual calculation with month/day check
{"if": [
  {"or": [
    {">": [
      {"month": {"today": null}},
      {"month": {"var": "birthdate"}}
    ]},
    {"and": [
      {"==": [
        {"month": {"today": null}},
        {"month": {"var": "birthdate"}}
      ]},
      {">=": [
        {"day": {"today": null}},
        {"day": {"var": "birthdate"}}
      ]}
    ]}
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
]}
```

### Issue: Days between dates is negative

**Problem:** `days` operator returns negative number.

**Explanation:** Order matters! `days([end, start])` returns positive if end > start.

```json
// Data: {"start": "2024-01-01", "end": "2024-01-15"}

// ❌ Wrong order - returns negative
{"days": [{"var": "start"}, {"var": "end"}]}  // → -14

// ✅ Correct order
{"days": [{"var": "end"}, {"var": "start"}]}  // → 14

// ✅ Use abs if order uncertain
{"abs": [{"days": [{"var": "end"}, {"var": "start"}]}]}
```

### Issue: Date arithmetic creates invalid dates

**Problem:** Date construction with overflow creates unexpected dates.

**Solution:** This is actually a feature! Date overflow normalizes automatically:

```json
// Month 13 wraps to next year
{"date": [2023, 13, 1]}  // → "2024-01-01" ✓

// Day 32 wraps to next month
{"date": [2024, 1, 32]}  // → "2024-02-01" ✓

// Can be used for date arithmetic:
{"date": [
  {"year": {"var": "someDate"}},
  {"+": [{"month": {"var": "someDate"}}, 3]},  // Add 3 months
  {"day": {"var": "someDate"}}
]}

// ✅ For stricter validation, check explicitly
{"if": [
  {"and": [
    {">=": [month, 1]},
    {"<=": [month, 12]},
    {">=": [day, 1]},
    {"<=": [day, 31]}
  ]},
  {"date": [year, month, day]},
  {"return": "Invalid date"}
]}
```

### Issue: DATEFORMAT returns wrong format

**Problem:** Date formatting doesn't match expected output.

**Common causes:**
1. **Wrong format specifier** - Using incorrect format string
2. **Custom format syntax error** - Typo in strftime codes
3. **Locale differences** - Month/day names depend on system locale

**Solutions:**
```json
// ✅ Use predefined formats for consistency
{"DATEFORMAT": [date, "iso"]}     // "2024-01-15"
{"DATEFORMAT": [date, "short"]}   // "01/15/2024"
{"DATEFORMAT": [date, "long"]}    // "January 15, 2024"
{"DATEFORMAT": [date, "eu"]}      // "15/01/2024"

// ✅ Check strftime syntax for custom formats
{"DATEFORMAT": [date, "%Y-%m-%d"]}  // ✓ Correct
{"DATEFORMAT": [date, "%Y/%m/%d"]}  // ✓ Correct
{"DATEFORMAT": [date, "YYYY-MM-DD"]} // ✗ Wrong (not strftime)

// Common strftime codes:
// %Y = 4-digit year (2024)
// %m = 2-digit month (01-12)
// %d = 2-digit day (01-31)
// %B = Full month name (January)
// %b = Short month name (Jan)
// %A = Full weekday (Monday)
// %a = Short weekday (Mon)
```

### Issue: Timezone offset causing wrong dates

**Problem:** Dates don't match expected values due to timezone differences.

**Solutions:**
```json
// Problem: UTC midnight might be different day in local timezone

// ✅ Check current timezone configuration
// If set_timezone_offset(420) for UTC+7:
{"today": null}  // Returns midnight in UTC+7

// ✅ Be aware of timezone impact
// UTC: 2024-01-15T23:00:00Z
// UTC+7: 2024-01-16T06:00:00+07:00 (next day!)

// ✅ For date-only comparisons, use date components
{"and": [
  {"==": [{"year": date1}, {"year": date2}]},
  {"==": [{"month": date1}, {"month": date2}]},
  {"==": [{"day": date1}, {"day": date2}]}
]}
```

### Issue: DATEDIF with wrong units

**Problem:** `DATEDIF` returns unexpected values.

**Common causes:**
1. **Wrong unit code** - Using lowercase or incorrect code
2. **Confusing similar units** - "M" vs "YM" vs "MD"

**Solutions:**
```json
// Unit codes (case-sensitive):
// "Y"  - Complete years
// "M"  - Complete months (total, not relative to years)
// "D"  - Total days
// "YM" - Months ignoring years (remainder after years)
// "YD" - Days ignoring years (day of year difference)
// "MD" - Days ignoring months and years (day of month difference)

// Example: From 1990-03-15 to 2023-07-10
{"DATEDIF": ["1990-03-15", "2023-07-10", "Y"]}   // → 33 years
{"DATEDIF": ["1990-03-15", "2023-07-10", "M"]}   // → 399 months (total)
{"DATEDIF": ["1990-03-15", "2023-07-10", "YM"]}  // → 3 months (remainder)

// ✅ For "XX years, YY months" display:
{"cat": [
  {"DATEDIF": [start, end, "Y"]}, " years, ",
  {"DATEDIF": [start, end, "YM"]}, " months"
]}
```

### Issue: YEARFRAC basis confusion

**Problem:** Different basis values give different results.

**Explanation:** Basis determines day-count convention:

```json
// 0 = US (NASD) 30/360 - Assumes 30 days/month, 360 days/year
// 1 = Actual/actual - Exact days, exact year length
// 2 = Actual/360 - Exact days, 360-day year
// 3 = Actual/365 - Exact days, 365-day year  
// 4 = European 30/360 - Similar to 0 but European rules

// ✅ For most accurate age calculations:
{"YEARFRAC": [birthdate, today, 1]}  // Actual/actual

// ✅ For financial calculations (bonds, loans):
{"YEARFRAC": [startDate, endDate, 0]}  // US 30/360 (common in US)

// ✅ For simple interest:
{"YEARFRAC": [startDate, endDate, 3]}  // Actual/365
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
- **Timezone handling**: Efficient offset calculation without heavy datetime libraries
- **Year calculations** use chrono library for accuracy

---

## Timezone Configuration

Date and time operators (`today`, `now`, `dateformat`, `year`, `month`, `day`, `date`) are timezone-sensitive.

- **Default Behavior**: UTC (Coordinated Universal Time)
- **Configurable**: You can set a timezone offset in minutes via the API (`set_timezone_offset`).

When a timezone offset is set:
1. **Inputs are shifted**: `today` and `now` return values in the specified timezone.
2. **Operations are shifted**: extraction (`year`, `month`, `day`) and formatting (`dateformat`) respect the offset.
3. **Outputs are shifted**: `date` constructs values relative to the offset.

### Example

With offset `420` (UTC+7, e.g., Bangkok/Jakarta):
```json
{"today": null}  // Returns midnight in UTC+7 (e.g. "2024-01-16T00:00:00.000+07:00" normalized to UTC)
{"hour": {"now": null}} // Returns hour in UTC+7
```
