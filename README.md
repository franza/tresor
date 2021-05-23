Command line application to securely store sensitive data

# Installation

Tresor is written in Rust and will be published soon in crates.io.
Currently, you can clone the sources from the GitHub and compile it manually with `cargo build`.

```shell
git clone https://github.com/franza/tresor.git
cd tresor
cargo build --release
```

# Usage

Tresor encrypts each key with individually specified password. 
Keys are grouped into buckets which are used for namespacing.

Basic usage:

```shell
# Initializing storage
$ tresor init

# Storing secrets

$ tresor store browserpasswords facebook supersecretpassword
Enter password: 
# Saved!

# Accessing the secret key will ask you for password
$ tresor get browserpasswords facebook 
Enter password: 
supersecretpassword

# Overwriting existing value will require you to enter old password
# Entering incorrect password will cause an error!

$ tresor store browserpasswords facebook completelydifferentpassword
Bucket browserpasswords already contains key facebook with a value. Enter the password which was used to encrypt it in order to overwrite: 
Failed to decrypt the data

# Let's try again but with correct password
$ tresor store browserpasswords facebook completelydifferentpassword
Bucket browserpasswords already contains key facebook with a value. Enter the password which was used to encrypt it in order to overwrite: 
Enter new password: 

# Deleting values will require you yo enter old password as well
$ tresor delete browserpasswords facebook 
Enter the password which was used to encrypt this value: 
```