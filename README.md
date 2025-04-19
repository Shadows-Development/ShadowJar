# **ShadowJar - Minecraft Server Jar Builder**  

> ⚠️ **Notice:** Development of ShadowJar is currently progressing at a slower pace due to the author's limited experience with Rust.  
> While the core ideas and goals for the project are well-defined, implementation is taking longer as foundational Rust knowledge is being actively developed.  
>  
> This project is **not abandoned** — continued development is expected over time, and major milestones will still be pursued.  
>  
> Contributions from the community are also welcome to help accelerate progress, especially from those experienced in Rust or Minecraft server tooling.

ShadowJar is an **open-source Minecraft server jar builder** designed for **developers, server administrators, and companies** who need a **self-hosted, reliable solution** for fetching and compiling different Minecraft server versions without relying on external APIs.  

With ShadowJar, you can **automate server builds**, maintain a **local versioning API**, and reduce dependency on third-party services like SpigotMC and PaperMC for fetching and managing versions. While ShadowJar still utilizes their official tools (such as `BuildTools.jar`), it allows users to **self-host their own build system**, cache versions locally, and automate updates—eliminating the need to manually query external sources.

---

## **🚀 Features**
- ✅ **Automatic Spigot Builds** – Fetches and compiles Spigot versions using `BuildTools.jar`.
- ✅ **Structured Storage** – Stores builds in `Builds/{ServerType}/{Version}/`.
- ✅ **Build Cleanup** – Keeps only the final `.jar` file to save disk space.
- ✅ **SQLite Database** – Tracks built versions for easy retrieval.
- ✅ **Self-Hosting Support** – Users can run their own API instance.
- ✅ **API for Build Version Retrieval** – Query available versions programmatically.
- ✅ **Better Logging & Error Handling** – Improved error messages and structured logging.
- 🛠️ **Parallel Builds** (In Progress) – Optimize compilation by running builds concurrently.
- 🛠️ **Rework of the Build System** (In Progress) – Improve efficiency and modularity.
- 🛠️ **Support for Additional Server Types** (Planned) – Paper, Forge, Fabric support.

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
5. **Exposes an API** that allows querying available versions.

---

## **🛠️ Roadmap**
### **🚀 Short-Term Goals**
- [ ] **Rework of the Build System** (Optimizing compilation & modularizing)
- [ ] **Support Additional Server Types** (Paper, Forge, Fabric)
- [ ] **Support for Multiple Operating Systems**
- [X] **Implement API for Build Version Retrieval**
- [X] **Better Logging & Error Handling**
- [🛠] **Parallel Builds** (Currently in progress)

### **🌍 Long-Term Goals**
- [ ] **Webhook Notifications for New Minecraft Versions**
- [ ] **Custom Build Flags for `BuildTools.jar`**
- [ ] **Cloud Hosting Support (AWS, DigitalOcean, etc.)**
- [?] **GUI for Managing Builds**
- [ ] **API Authentication & Rate Limiting**

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
