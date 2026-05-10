# 8. ภาคผนวก

ส่วนนี้รวบรวมข้อมูลทางเทคนิคและรายละเอียดเพิ่มเติมสำหรับผู้ดูแลระบบและนักพัฒนา

---

## ภาคผนวก ก: โครงสร้างฐานข้อมูล

### ตารางหลักที่ใช้

| ตาราง | คำอธิบาย | คอลัมน์ที่ใช้ |
|-------|----------|---------------|
| `patient` | ข้อมูลผู้ป่วย | hn, cid, pname, fname, lname, pttype |
| `ovst` | การมาพบแพทย์ | vn, hn, vstdate, cur_dep, pttype |
| `opdscreen` | การตรวจร่างกาย | vn, bw, bps, bpd, pulse |
| `opitemrece` | รายการยาที่จ่าย | vn, hn, icode, qty |
| `drugitems` | ข้อมูลยา | icode, name, units |
| `pttype` | ประเภทสิทธิ์ | pttype, name |
| `kskdepartment` | ห้องตรวจ | depcode, department |
| `lab_head` | ใบสั่งแลป | lab_order_number, hn, order_date |
| `lab_order` | ผลแลป | lab_order_number, lab_items_code, lab_order_result |
| `lab_items` | รายการแลป | lab_items_code, lab_items_name |

### หมายเหตุ

ชื่อตารางและคอลัมน์อาจแตกต่างกันในแต่ละระบบ HIS:
- **HosXP:** ใช้ชื่อตารางดังตารางด้านบน
- **PMQA:** อาจมีชื่อตารางคล้ายกัน
- **ระบบอื่น:** อาจต้องปรับแต่ง Query ใน source code

---

## ภาคผนวก ข: ไฟล์การตั้งค่า

### config.ini

ไฟล์เก็บข้อมูลการเชื่อมต่อฐานข้อมูล (เข้ารหัสแล้ว)

**ที่อยู่:**
- Windows: `%LOCALAPPDATA%\HerbReady\config.ini`
- macOS: `~/Library/Application Support/HerbReady/config.ini`
- Linux: `~/.local/share/HerbReady/config.ini`

### app_config.json

ไฟล์เก็บการตั้งค่ายา ห้องตรวจ กฎแลป และปฏิกิริยาระหว่างยา

**โครงสร้าง:**
```json
{
  "drugs": [
    {
      "icode": "1510001",
      "abbr": "ฟ้าทะลายโจร",
      "course_days": 30,
      "capsules": 30,
      "drug_name": "ฟ้าทะลายโจร 500 mg.",
      "enabled": true
    }
  ],
  "departments": [
    {
      "code": "011",
      "name": "แพทย์แผนไทย"
    }
  ],
  "lab_rules": [
    {
      "lab_items_code": "001",
      "lab_items_name": "น้ำตาลในเลือด (FBS)",
      "threshold": 126,
      "compare_gt": true,
      "compare_eq": false,
      "compare_lt": false
    }
  ],
  "herb_drug_interactions": [
    {
      "modern_drug_icode": "1010001",
      "modern_drug_name": "Warfarin",
      "herb_drugs": [
        { "icode": "1510003", "name": "ขิง" },
        { "icode": "1510004", "name": "กระเพรา" }
      ],
      "reason": "อาจเพิ่มความเสี่ยงเลือดออก"
    }
  ]
}
```

---

## ภาคผนวก ค: คีย์ลัดและแป้นพิมพ์

| ปุ่ม | การทำงาน |
|------|----------|
| **F5** | รีเฟรชข้อมูล |
| **Ctrl + F** | ค้นหาในตาราง |
| **Ctrl + P** | พิมพ์ |
| **Ctrl + S** | บันทึกการตั้งค่า |
| **Ctrl + +/-** | ซูมเข้า/ออก |
| **Esc** | ปิดหน้าต่าง |

---

## ภาคผนวก ง: ข้อกำหนดระบบ (System Requirements)

### ขั้นต่ำ

| องค์ประกอบ | ความต้องการ |
|-----------|-------------|
| **CPU** | Intel Core i3 หรือเทียบเท่า |
| **RAM** | 4 GB |
| **พื้นที่ว่าง** | 500 MB |
| **จอภาพ** | ความละเอียด 1280x720 ขึ้นไป |
| **OS** | Windows 10, macOS 11, Ubuntu 20.04 |

### แนะนำ

| องค์ประกอบ | ความแนะนำ |
|-----------|------------|
| **CPU** | Intel Core i5 หรือเทียบเท่า |
| **RAM** | 8 GB |
| **พื้นที่ว่าง** | 1 GB |
| **จอภาพ** | ความละเอียด 1920x1080 |
| **Network** | การเชื่อมต่อฐานข้อมูล 100 Mbps ขึ้นไป |

---

## ภาคผนวก จ: การแก้ไข Source Code

### โครงสร้างโปรแกรม

```
herbready/
├── src/                      # Vue 3 Frontend
│   ├── components/           # Vue Components
│   │   ├── dialogs/          # Dialog components
│   │   ├── DailyTab.vue      # แท็บคิววันนี้
│   │   ├── SearchTab.vue     # แท็บค้นหาผู้ป่วย
│   │   ├── HistoryTab.vue    # แท็บประวัติการจ่ายยา
│   │   └── DrugPanel.vue     # แผงเลือกยา
│   ├── stores/               # Pinia Stores
│   ├── api/                  # Tauri API wrappers
│   ├── types/               # TypeScript types
│   └── utils/               # Utilities
├── src-tauri/                # Rust Backend
│   ├── src/
│   │   ├── commands.rs       # Tauri commands
│   │   ├── db.rs            # Database connection
│   │   ├── models.rs        # Data structures
│   │   ├── queries.rs       # SQL queries
│   │   ├── config.rs       # Config management
│   │   └── crypto.rs       # Encryption
│   └── tauri.conf.json     # Tauri config
└── package.json             # Node dependencies
```

### การแก้ไขการ Query

หากโครงสร้างตารางในฐานข้อมูลต่างจากค่าเริ่มต้น สามารถแก้ไข SQL Query ได้ที่:

- `src-tauri/src/queries.rs` — สำหรับ Query หลัก
- `src-tauri/src/db.rs` — สำหรับ Database connection

---

## ภาคผนวก ฉ: การอัปเกรดโปรแกรม

เมื่อมีเวอร์ชันใหม่:

1. **ดาวน์โหลดเวอร์ชันใหม่**
   ```bash
   git pull origin main
   ```

2. **อัปเดต Dependencies**
   ```bash
   npm install
   ```

3. **Build โปรแกรม**
   ```bash
   npm run tauri build
   ```

4. **สำรองการตั้งค่าเดิม** (แนะนำ)
   - ส่งออกไฟล์ app_config.json ก่อนอัปเกรด

---

## ภาคผนวก ช: License

**© 2026 rxdevman. All rights reserved.**

HerbReady เผยแพร่ภายใต้ **MIT License**

---

## สรุปเอกสาร

เอกสารคู่มือการใช้งานนี้ครอบคลุม:

1. ✅ ภาพรวมของโปรแกรม
2. ✅ การติดตั้ง
3. ✅ การเชื่อมต่อฐานข้อมูล
4. ✅ การตั้งค่าโปรแกรม
5. ✅ คู่มือการใช้งานแต่ละส่วน
6. ✅ การส่งออกข้อมูลและพิมพ์
7. ✅ การแก้ปัญหาและ FAQ
8. ✅ ภาคผนวก

---

**ขอบคุณที่ใช้งาน HerbReady!**