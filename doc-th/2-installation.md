# 2. การติดตั้งโปรแกรม

## ข้อกำหนดเบื้องต้น (Prerequisites)

ก่อนติดตั้ง HerbReady คุณต้องมีซอฟต์แวร์ต่อไปนี้ติดตั้งอยู่ในเครื่อง:

| ซอฟต์แวร์ | เวอร์ชันขั้นต่ำ | หมายเหตุ |
|-----------|----------------|----------|
| **Node.js** | 18+ | สำหรับ build frontend (Vue) |
| **Rust** | 1.70+ | สำหรับ build backend (Tauri) |
| **MySQL** | 8.0+ | ฐานข้อมูล HIS ของโรงพยาบาล |
| **npm** | มาพร้อมกับ Node.js | จัดการ dependencies |
| **Git** | ทุกเวอร์ชัน | สำหรับ clone repository |

**ระบบปฏิบัติการที่รองรับ:**
- Windows 10/11 (แนะนำ)
- macOS 11+ (Big Sur ขึ้นไป)
- Linux (Ubuntu 20.04+, Fedora 35+)

---

## ขั้นตอนการติดตั้ง

### ขั้นตอนที่ 1: ดาวน์โหลดซอร์สโค้ด

เปิด Terminal (macOS/Linux) หรือ Command Prompt/PowerShell (Windows) แล้วรันคำสั่ง:

```bash
# Clone repository
git clone https://github.com/suradet-ps/herbready.git

# เข้าไปในโฟล์เดอร์โปรแกรม
cd herbready
```

### ขั้นตอนที่ 2: ติดตั้ง Dependencies

รันคำสั่งติดตั้ง package ที่จำเป็น:

```bash
npm install
```

คำสั่งนี้จะติดตั้ง:
- Vue 3 — Framework สำหรับสร้าง UI
- TypeScript — ภาษาสำหรับเขียนโค้ดที่มี Type safety
- Vite — Development server และ build tool
- Tauri CLI — เครื่องมือสำหรับ build desktop app
- Lucide Vue Icons — ไอคอนสำหรับ UI

### ขั้นตอนที่ 3: ติดตั้ง Rust (ถ้ายังไม่มี)

ถ้ายังไม่มี Rust ในเครื่อง ให้ติดตั้งโดยรัน:

```bash
# สำหรับ macOS, Linux, WSL
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# สำหรับ Windows
# ดาวน์โหลดจาก https://rustup.rs และรัน installer
```

> **หมายเหตุ:** หลังติดตั้ง Rust ให้ restart Terminal ใหม่

### ขั้นตอนที่ 4: รันโปรแกรมในโหมด Development

หลังติดตั้ง dependencies เสร็จ ให้รันคำสั่ง:

```bash
npm run tauri dev
```

คำสั่งนี้จะ:
1. Compile Rust backend
2. Start Vue dev server
3. เปิดหน้าต่างโปรแกรม HerbReady ขึ้นมา

**ระยะเวลา:** ครั้งแรกอาจใช้เวลาประมาณ 5-15 นาที (ขึ้นกับสเปคเครื่อง)

### ขั้นตอนที่ 5: Build เป็นไฟล์ติดตั้ง (Production)

เมื่อต้องการสร้างไฟล์ติดตั้งที่พร้อมใช้งานจริง:

```bash
npm run tauri build
```

ผลลัพธ์จะอยู่ที่:
- **Windows:** `src-tauri/target/release/bundle/msi/` หรือ `src-tauri/target/release/bundle/nsis/`
- **macOS:** `src-tauri/target/release/bundle/dmg/`
- **Linux:** `src-tauri/target/release/bundle/deb/` หรือ `AppImage/`

---

## การติดตั้งบน Windows โดยละเอียด

### ติดตั้ง Node.js

1. ดาวน์โหลด Node.js จาก https://nodejs.org/ (เลือก LTS version)
2. Run installer และทำตามขั้นตอน
3. ตรวจสอบการติดตั้งโดยเปิด PowerShell แล้วพิมพ์:
   ```powershell
   node --version
   npm --version
   ```

### ติดตั้ง Rust

1. ดาวน์โหลด rustup-init.exe จาก https://rustup.rs/
2. Run installer
3. เลือก **1) Default installation**
4. ตรวจสอบการติดตั้งโดยพิมพ์:
   ```powershell
   rustc --version
   cargo --version
   ```

### ติดตั้ง Visual Studio Build Tools (Windows เท่านั้น)

สำหรับ build Tauri บน Windows ต้องติดตั้ง Visual Studio Build Tools:

1. ดาวน์โหลดจาก https://visualstudio.microsoft.com/visual-cpp-build-tools/
2. เลือก **"Desktop development with C++"**
3. รอให้ติดตั้งเสร็จ

---

## การติดตั้งบน macOS โดยละเอียด

### ติดตั้ง Homebrew (ถ้ายังไม่มี)

```bash
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
```

### ติดตั้ง Node.js ผ่าน Homebrew

```bash
brew install node
```

### ติดตั้ง Rust ผ่าน Homebrew

```bash
brew install rust
```

หรือใช้ rustup ตามขั้นตอนด้านบน

---

## การติดตั้งบน Linux (Ubuntu) โดยละเอียด

### ติดตั้ง Node.js

```bash
# เพิ่ม NodeSource repository
curl -fsSL https://deb.nodesource.com/setup_18.x | sudo -E bash -

# ติดตั้ง Node.js
sudo apt-get install -y nodejs
```

### ติดตั้ง Rust

```bash
sudo apt-get install -y build-essential pkg-config libssl-dev
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

### ติดตั้ง dependencies อื่นๆ

```bash
sudo apt-get install -y libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf
```

---

## ปัญหาที่อาจพบในการติดตั้ง

| ปัญหา | วิธีแก้ไข |
|-------|----------|
| `npm install` ล้มเหลว | ลบ `node_modules` และ `package-lock.json` แล้วลองใหม่ `rm -rf node_modules package-lock.json && npm install` |
| Rust ไม่พบ | Restart Terminal หรือรัน `source ~/.cargo/env` |
| Build ช้ามาก | ตรวจสอบว่า antivirus ไม่ได้ scan โฟล์เดอร์ project |
| Error: "Could not find Visual Studio" | ติดตั้ง Visual Studio Build Tools (Windows) |

หากพบปัญหาอื่นๆ ให้ดูที่ [การแก้ปัญหา](./7-troubleshooting.md)

---

## การรันโปรแกรมหลังติดตั้ง

หลังจากติดตั้งเสร็จแล้ว ทุกครั้งที่ต้องการรันโปรแกรม:

```bash
# โหมด Development
npm run tauri dev

# หรือถ้าต้องการแค่ Frontend
npm run dev
```

---

## ขั้นตอนถัดไป

เมื่อติดตั้งโปรแกรมเสร็จแล้ว ให้ไปที่:

- **[การเชื่อมต่อฐานข้อมูล](./3-database-connection.md)** — เชื่อมต่อกับฐานข้อมูลของโรงพยาบาล