# s33d

a fully offline, paranoid-friendly cli tool that generates **bip-39 compatible** seed phrases using strong entropy and secure randomness.  
just you, your machine, and your seed.

![s33d terminal output](https://ik.imagekit.io/zwcfsadeijm/s33d_dfJp4GJ5e.png)

## what is this?

a **seed phrase** (also called a mnemonic) is a human-readable backup of your cryptocurrency wallet. instead of remembering complex private keys like `0x3a4b5c6d...`, you get 12 simple words like `apple tree moon river`.

same security, but you can actually write it down and remember it. lose your phone? use the seed phrase to restore your wallet. it's like a master key that recreates all your crypto addresses.

## why?

because generating seed phrases should be:

- 🔒 secure (using os-level entropy)
- 🎯 simple (just works™)
- 🛠 reliable (follows bip39 standard)
- ⚡️ fast (< 4ms generation time)
- 📦 tiny (~1mb binary)
- 🌍 multilingual (10 languages)
- 📱 mobile-ready (qr code support)

## install

### homebrew (macos & linux)

```bash
brew install rittikbasu/s33d/s33d
```

### cargo (cross-platform)

```bash
cargo install --git https://github.com/rittikbasu/s33d
```

## usage

generate 12-word phrase (default):

```bash
s33d -w 12
```

generate 24-word phrase (extra security):

```bash
s33d -w 24
```

generate with qr code for mobile wallets:

```bash
s33d -q
```

generate in different language:

```bash
s33d -l japanese
```

advanced - custom entropy bits (128-256):

```bash
s33d -s 192
```

show technical details:

```bash
s33d -e
```

show entropy as hexadecimal:

```bash
s33d -x
```

clean mode (just the phrase):

```bash
s33d -c
```

list all supported languages:

```bash
s33d --list
```

## updating

### homebrew

```bash
brew update && brew upgrade s33d
```

### cargo

```bash
cargo install --git https://github.com/rittikbasu/s33d --force
```

## languages

supports 10 languages with perfect compatibility:

- english (widely supported by all wallets)
- japanese, korean, chinese (simplified/traditional)
- french, italian, spanish, portuguese, czech

**note**: english is recommended for maximum wallet compatibility

## security

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

## development

### manual build

```bash
git clone https://github.com/rittikbasu/s33d.git
cd s33d
cargo build --release
```

### releases

pre-built binaries available at [releases](https://github.com/rittikbasu/s33d/releases)

## license

mit
