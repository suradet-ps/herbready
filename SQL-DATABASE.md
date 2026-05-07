# SQL Database Documentation

This document describes all SQL queries used by HerbReady to interact with the hospital's HIS database.

---

## Database Connection

- **Database Type**: MySQL 8.0+
- **Connection Method**: sqlx (Rust async database library)

```sql
-- Test connection
SELECT VERSION();
```

---

## Database Tables & Columns Used

### Core Tables

| Table | Description | Key Columns Used |
|-------|-------------|------------------|
| `patient` | Patient demographics | `hn`, `cid`, `pname`, `fname`, `lname`, `pttype` |
| `ovst` | Outpatient visits | `vn`, `hn`, `vstdate`, `cur_dep`, `pttype` |
| `opdscreen` | Vital signs / screening | `vn`, `bw`, `bps`, `bpd`, `pulse` |
| `opitemrece` | Dispensed items | `vn`, `hn`, `icode`, `qty` |
| `drugitems` | Drug master | `icode`, `name`, `units` |
| `pttype` | Patient type | `pttype`, `name` |
| `kskdepartment` | Department master | `depcode`, `department` |
| `lab_head` | Lab order header | `lab_order_number`, `hn`, `order_date` |
| `lab_order` | Lab order details | `lab_order_number`, `lab_items_code`, `lab_order_result` |
| `lab_items` | Lab item master | `lab_items_code`, `lab_items_name` |

---

## SQL Queries by Feature

---

### 1. Daily Queue Processing (Tab 1)

**Purpose**: Load patients for a specific date and department, with drug eligibility calculation.

**Query Function**: `build_daily_query()`

**Tables Used**:
- `ovst` - Visit records
- `patient` - Patient info
- `pttype` - Patient type
- `kskdepartment` - Department
- `opdscreen` - Vital signs
- `drugitems` - Drug master
- `opitemrece` - Dispensing history

**Key Columns**:
```sql
SELECT
    v.vn,                    -- ovst.vn: Visit number
    v.hn,                    -- ovst.hn: Hospital number
    p.cid,                   -- patient.cid: Citizen ID
    p.pname, p.fname, p.lname,  -- patient name
    k.department,            -- kskdepartment.department
    pt.name,                 -- pttype.name
    lv.vstdate,              -- Last visit date
    vitals.bw,               -- opdscreen.bw: Body weight
    vitals.bps,              -- opdscreen.bps: Blood pressure systolic
    vitals.bpd,              -- opdscreen.bpd: Blood pressure diastolic
    vitals.pulse,            -- opdscreen.pulse
    m.drug_name,             -- drugitems.name
    m.course_days             -- Configured course days
```

**Logic**:
1. Filter visits by date and department
2. Join patient demographics
3. Calculate last visit date before process date
4. Get latest vitals (either from same date or most recent prior)
5. Check dispensing history for each configured herb drug
6. Categorize drugs into:
   - **Eligible**: Last dispense + course_days <= today
   - **Never dispensed**: No history for this drug
   - **Not yet eligible**: Last dispense + course_days > today (show days remaining)

---

### 2. Individual Patient Search (Tab 2)

**Purpose**: Search for a patient by HN, CID, or name to view their drug eligibility.

**Query Function**: `build_individual_search_query()`

**Tables Used**: Same as daily query, but filtered by patient identifier

**Search Modes**:
```sql
-- By HN
AND p.hn = ?

-- By CID
AND p.cid = ?

-- By Name (LIKE search)
AND CONCAT(p.pname, p.fname, ' ', p.lname) LIKE ?
```

---

### 3. Dispensing History (Tab 3)

**Purpose**: View all herbal medicine dispensing records within a date range.

**Query Function**: `build_dispensing_history_query()`

**Query**:
```sql
SELECT
    o.vstdate,               -- ovst.vstdate
    p.hn,                    -- patient.hn
    p.cid,                   -- patient.cid
    CONCAT(p.pname, p.fname, ' ', p.lname) AS pt_name,
    di.name,                 -- drugitems.name
    oi.qty,                  -- opitemrece.qty
    di.units                 -- drugitems.units
FROM opitemrece oi
JOIN ovst o ON o.vn = oi.vn
JOIN patient p ON p.hn = oi.hn
JOIN drugitems di ON di.icode = oi.icode
WHERE oi.icode IN (...)       -- Configured herb drug icodes
  AND o.vstdate BETWEEN ? AND ?
  -- Optional: HN/CID/Name filter
```

---

### 4. Patient Herb History (Tree View)

**Purpose**: View detailed dispensing history for a specific patient.

**Query Function**: `build_patient_herb_history_query()`

**Query**:
```sql
SELECT
    o.vstdate,               -- ovst.vstdate
    di.name,                -- drugitems.name
    oi.qty,                 -- opitemrece.qty
    di.units                -- drugitems.units
FROM opitemrece oi
JOIN ovst o ON o.vn = oi.vn
JOIN drugitems di ON di.icode = oi.icode
WHERE oi.hn = ?              -- Patient HN
  AND oi.icode IN (...)     -- Configured herb drug icodes
  AND o.vstdate >= DATE_SUB(CURDATE(), INTERVAL ? YEAR)
ORDER BY o.vstdate DESC
```

---

### 5. Patient Lookup

**Purpose**: Quick search for patients by name, HN, or CID.

**Query Function**: `build_patient_lookup_by_name()`, `build_patient_lookup_by_hn_or_cid()`

**By Name**:
```sql
SELECT
    p.hn,
    p.cid,
    CONCAT(p.pname, p.fname, ' ', p.lname) AS pt_name,
    pt.name AS pttype_name
FROM patient p
LEFT JOIN pttype pt ON pt.pttype = p.pttype
WHERE CONCAT(p.pname, p.fname, ' ', p.lname) LIKE ?
```

**By HN or CID**:
```sql
SELECT
    p.hn,
    p.cid,
    CONCAT(p.pname, p.fname, ' ', p.lname) AS pt_name,
    pt.name AS pttype_name
FROM patient p
LEFT JOIN pttype pt ON pt.pttype = p.pttype
WHERE p.hn = ?   -- or p.cid = ?
```

---

### 6. Lab Results

**Purpose**: Fetch latest lab results for patients to check against threshold rules.

**Query Function**: `build_latest_lab_results_query()`, `build_latest_abnormal_lab_results_query()`

**Tables Used**:
- `lab_head` - Lab order header
- `lab_order` - Lab order details
- `lab_items` - Lab item master

**Key Columns**:
```sql
SELECT
    h.hn,                    -- lab_head.hn
    o.lab_items_code,        -- lab_order.lab_items_code
    i.lab_items_name,        -- lab_items.lab_items_name
    o.lab_order_result,      -- lab_order.lab_order_result
    DATE(h.order_date)       -- lab_head.order_date
FROM lab_order o
JOIN lab_head h ON h.lab_order_number = o.lab_order_number
JOIN lab_items i ON o.lab_items_code = i.lab_items_code
WHERE h.hn IN (...)
  AND o.lab_items_code IN (...)
  AND DATE(h.order_date) <= ?
  AND o.lab_order_result IS NOT NULL
  AND o.lab_order_result <> ''
  AND o.lab_order_result REGEXP '^[[:space:]]*[0-9]'  -- Numeric only
```

**Abnormal Results Filter**:
```sql
-- Applied after getting latest results per (hn, lab_items_code)
WHERE
    (lab_items_code = 'X' AND CAST(result AS DECIMAL) > threshold)
    OR (lab_items_code = 'Y' AND CAST(result AS DECIMAL) < threshold)
    OR (lab_items_code = 'Z' AND ABS(CAST(result AS DECIMAL) - threshold) < 0.001)
```

---

### 7. Herb-Drug Interaction Check

**Purpose**: Detect if patients have active modern drug prescriptions that interact with herb drugs.

**Query Function**: `build_check_modern_drug_query()`

**Query**:
```sql
SELECT DISTINCT oi.hn, oi.icode AS modern_icode
FROM opitemrece oi
JOIN ovst o ON o.vn = oi.vn
WHERE oi.hn IN (...)
  AND oi.icode IN (...)        -- Modern drug icodes from interaction rules
  AND DATE(o.vstdate) >= DATE_SUB(?, INTERVAL 1 YEAR)
  AND DATE(o.vstdate) <= ?     -- process_date
```

---

### 8. Drug & Department Name Lookup

**Purpose**: Look up display names for drug codes and department codes in settings.

**Query - Drug Name**:
```sql
SELECT name FROM drugitems WHERE icode = ? LIMIT 1
```

**Query - Department Name**:
```sql
SELECT department FROM kskdepartment WHERE depcode = ? LIMIT 1
```

---

### 9. Lab Item Name Lookup

**Purpose**: Get lab item display name from code.

**Query**:
```sql
SELECT lab_items_name FROM lab_items WHERE lab_items_code = ? LIMIT 1
```

---

## Configuration Tables (Local)

HerbReady uses local JSON configuration files for:

| Config | Description |
|--------|-------------|
| `drugs` | List of herb drugs (icode, abbr, course_days, capsules) |
| `departments` | Department codes to include in daily queue |
| `lab_rules` | Lab items to monitor with threshold values |
| `herb_drug_interactions` | Rules linking modern drugs to herb drugs |

These are NOT stored in the database but passed to SQL queries as parameters.

---

## Parameter Binding

All user input and dates are passed as parameterized values (`?`) to prevent SQL injection:

```rust
sqlx::query("SELECT ... WHERE v.vstdate = ?")
    .bind(&process_date)
    .fetch_all(&pool)
    .await
```