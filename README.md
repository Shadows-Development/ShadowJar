# **ShadowJar - Minecraft Server Jar Builder**  
ShadowJar is an **open-source Minecraft server jar builder** that automates fetching and compiling different Minecraft server types like **Spigot, Paper, Forge, and Fabric**. It includes a versioning API, automatic updates, and customizable build storage.

## **🚀 Features**
- ✅ **Automatic Spigot Builds** – Fetches and compiles Spigot versions using `BuildTools.jar`.
- ✅ **Structured Storage** – Stores builds in `Builds/{ServerType}/{Version}/`.
- ✅ **Build Cleanup** – Keeps only the final `.jar` file to save disk space.
- ✅ **SQLite Database** – Tracks built versions for easy retrieval.
- ✅ **Self-Hosting Support** – Users can run their own API instance.
- ✅ **Parallel Builds** (Planned) – Optimize compilation by running builds concurrently.

---

## **📦 Installation**
### **1️⃣ Prerequisites**
- [Git](https://git-scm.com/downloads)
- [Rust](https://www.rust-lang.org/)
- [Java 17+](https://adoptium.net/)

### **2️⃣ Clone the Repository**
```sh
git clone https://github.com/Shadows-Development/ShadowJar.git
cd ShadowJar
```

### **3️⃣ Build & Run**
```sh
cargo build --release
cargo run
```

---

## **⚙️ How It Works**
1. **Checks for `BuildTools.jar`** and downloads it if missing.
2. **Runs `BuildTools.jar` in Git Bash** to compile Spigot.
3. **Stores builds in structured folders**:  
   ```
   Builds/
   ├── Spigot/
   │   ├── 1.21.4/
   │   │   ├── spigot-1.21.4.jar
   ```
4. **Deletes temporary build files** and keeps only the final `.jar`.

---

## **🛠️ Roadmap**
### **🚀 Short-Term Goals**
- [ ] **Support Additional Server Types** (Paper, Forge, Fabric)
- [ ] **Implement API for Build Version Retrieval**
- [ ] **Better Logging & Error Handling**
- [ ] **Parallel Builds** to speed up compilation

### **🌍 Long-Term Goals**
- [ ] **Webhook Notifications for New Minecraft Versions**
- [ ] **Custom Build Flags for `BuildTools.jar`**
- [ ] **Cloud Hosting Support (AWS, DigitalOcean, etc.)**
- [ ] **GUI for Managing Builds**

---

## **👨‍💻 Contributing**
We welcome contributions! To contribute:
1. Fork the repo.
2. Create a new branch.
3. Submit a pull request.

---

## **📄 License**
This project is licensed under the **MIT License**.

---

## **📢 Need Help?**
- Open an **issue** on GitHub.

---

## **✨ Star the Repo & Follow for Updates!**
If you like this project, consider starring ⭐ it on GitHub!
