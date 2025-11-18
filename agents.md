# 🧭 Agent Instructions: Client Module Documentation

## Goal
All **public methods** in the `client/` folder **must include complete Rust documentation comments** (`///`) following the **official Rust documentation style guide**.

---

## 📚 Requirements

### 1. Add Documentation to All Public Methods
For every `pub fn` in the `client/` folder:
- Add a **Rust doc comment** (`///`) immediately above the function.
- The documentation should explain:
  - **What** the function does.
  - **How** to use it.
  - Any **parameters**, **return values**, and **error conditions**.
  - If relevant, include a **code example**.

**Example:**
```rust
/// Sends a GET request to the specified endpoint.
///
/// # Arguments
///
/// * `url` - The URL to send the request to.
/// * `headers` - Optional headers to include with the request.
///
/// # Returns
///
/// Returns a `Result` containing the response body as a `String` if successful,
/// or an error if the request fails.
///
/// # Examples
///
/// ```
/// let response = client::get("https://api.example.com", None)?;
/// println!("{}", response);
/// ```
pub fn get(url: &str, headers: Option<HashMap<String, String>>) -> Result<String, ClientError> {
    // implementation
}
```

---

### 2. Follow the Official Rust Documentation Style Guide
Agents **must** conform to the [Rust Documentation Style Guide](https://doc.rust-lang.org/rustdoc/how-to-write-documentation.html).  
Key points:
- Use **third-person descriptive style** (“Returns the result…” not “This function returns…”).
- Use Markdown formatting for sections like:
  - `# Arguments`
  - `# Returns`
  - `# Errors`
  - `# Examples`
- Ensure all examples **compile and run correctly** if copied into a `rustdoc` test.
- Keep tone **concise and consistent** across all files.
- Do not document Arguments if the only argument is "self"
- Do not document Returns if nothing is returned
---

### 3. Scope
Apply these rules to:
- All `.rs` files under `client/`
- Any new or modified public functions
- Structs, enums, and traits with public visibility (document them with `///` too)

---

### 4. Validation
Before submission:
- Run `cargo doc --no-deps` to verify docs build correctly.
- Check for warnings or missing docs using:
  ```bash
  cargo clippy -- -D missing_docs
  ```

---

### ✅ Summary
**In short:**  
> Every public item in the `client` module must be fully documented using `///` comments, in accordance with Rust’s official documentation standards, and include meaningful examples where possible.
