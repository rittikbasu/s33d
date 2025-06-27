# s33d

a fully offline, zero-dependency cli tool that generates **bip-39 compatible** seed phrases using strong entropy and secure randomness.  
just you, your machine, and your seed.

## why

generating seed phrases should be:

- 🔒 secure (using os-level entropy)
- 🎯 simple (just works™)
- 🛠 reliable (follows bip39 standard)
- ⚡️ fast (< 1ms generation time)
- 📦 tiny (< 700kb binary)

## install

- quick install (recommended):

  ```bash
  cargo install --git https://github.com/rittikbasu/s33d
  ```

- manual build (for development):

  clone repo:

  ```bash
  git clone https://github.com/rittikbasu/s33d.git
  ```

  navigate to project directory:

  ```bash
  cd s33d
  ```

  build optimized binary (will be in target/release/s33d):

  ```bash
  cargo build --release
  ```

## usage

generate 12-word phrase (standard):

```bash
s33d --words 12
```

generate 24-word phrase (extra security):

```bash
s33d
```

advanced - custom entropy bits (128-256):

```bash
s33d --strength 192
```

show technical details:

```bash
s33d --entropy
```

quiet mode (just the phrase):

```bash
s33d --quiet
```

## security

⚠️ important:

- write phrases on paper, never digitally
- store in secure location
- verify words before final storage
- never share your phrase
- consider hardware wallets

## license

mit
