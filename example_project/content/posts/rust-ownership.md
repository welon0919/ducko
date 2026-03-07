---
title: "深入淺出 Rust 所有權"
date: "2026-03-05"
---

**所有權 (Ownership)** 是 Rust 最獨特的特性，它讓 Rust 無需垃圾回收 (GC) 就能保證記憶體安全。

### 所有權的三大規則

1. Rust 中的每一個值都由一個變數擁有 (Owner)。
2. 同一時間，一個值只能有一個 Owner。
3. 當 Owner 離開作用域，該值會被丟棄 (Drop)。

### 程式碼範例

讓我們來看看這個經典的例子：

```rust
fn main() {
    let s1 = String::from("hello");

    // s1 的所有權移動 (Move) 給了 s2
    let s2 = s1;

    // println!("{}, world!", s1); // 這行會報錯！因為 s1 已經失效了
    println!("{}, world!", s2);
}
```

引用與借用 (References and Borrowing)
如果我們不想轉移所有權，可以使用 & 符號來「借用」：

```rust
fn calculate_length(s: &String) -> usize {
    s.len()
} // s 離開作用域，但因為它是借來的，所以不會把 String 清除
```

"Rust 的學習曲線雖然陡峭，但風景是值得的。"

希望這篇簡短的筆記能幫助你理解 Rust 的核心概念！