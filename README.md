# s33d

a fully offline, zero-dependency cli tool that generates **bip-39 compatible** seed phrases using strong entropy and secure randomness.  
just you, your machine, and your seed.

## why

generating seed phrases should be:

- 🔒 secure (using os-level entropy)
- 🎯 simple (just works™)
- 🛠 reliable (follows bip39 standard)
- ⚡️ fast (< 1ms generation time)
- 🌍 multilingual (10 languages)
- 📦 tiny (~1.2mb binary)

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
s33d -w 12
```

generate 24-word phrase (extra security):

```bash
s33d
```

generate in different language:

```bash
s33d -l japanese
s33d -l french
```

advanced - custom entropy bits (128-256):

```bash
s33d -s 192
```

show technical details:

```bash
s33d -e
```

quiet mode (just the phrase):

```bash
s33d -q
```

list all supported languages:

```bash
s33d --list
```

## languages

supports 10 languages with perfect compatibility:

- english (widely supported by all wallets)
- japanese, korean, chinese (simplified/traditional)
- french, italian, spanish, portuguese, czech

**note**: english is recommended for maximum wallet compatibility

## security

⚠️ important:

- write phrases on paper, never digitally
- store in secure location
- verify words before final storage
- never share your phrase
- consider hardware wallets

## entropy source

uses your operating system's cryptographically secure random number generator:

- unix/linux: `/dev/urandom`
- windows: `CryptGenRandom`
- macos: `SecRandomCopyBytes`

this ensures truly random, unpredictable seed generation with proper entropy.

## license

mit
