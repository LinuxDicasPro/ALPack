<p align="center">
  <img src="logo.png" alt="ALPack" width="320"/>
</p>

<h1 align="center"><strong>ALPack - Alpine Linux SandBox Packager</strong></h1>

**ALPack** is a **portable** tool written in **Rust** for creating and
managing **Alpine Linux rootfs**. It uses **proot** and **bubblewrap (bwrap)**
to provide **isolated environments** without requiring superuser privileges.

✨ With ALPack, you can:

* 📦 Easily create and manage **portable Alpine rootfs**.
* 🛠️ Work directly with **APKBUILDs**, simplifying the packaging process.
* ⚙️ Build **static binaries**, ideal for distribution across different systems.
* 💼 Run anywhere without complex installation, thanks to its fully **portable design**.

Lightweight, fast, and productivity-focused, ALPack bridges the gap between Alpine
Linux flexibility and secure isolated environments.

---

## 🚀 Usage

Creating an Alpine rootfs:

```bash
alpack init --release latest-stable
```

Building a static binary with APKBUILD:

```bash
alpack build ./APKBUILD --static
```

Running in an isolated environment with proot or bwrap:

```bash
alpack run --tool proot
# or
alpack run --tool bwrap
```

---

## 🧪 Why AlpineBox for Static Binaries?

Alpine Linux uses the **musl libc** and provides toolchains that are
naturally geared toward **static compilation**. Combined with the
lightweight nature of AlpineBox:

* You can quickly set up isolated environments for building static binaries with `musl-gcc`;
* Perfect for creating portable binaries that run across different Linux systems;
* Avoids linking with host system libraries;
* Small footprint and fast setup, ideal for CI/CD pipelines and embedded builds.

---

## 📄 License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.