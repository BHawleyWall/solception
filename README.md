# Solana Inception Command-Line Tool
A command-line tool for discovery of the origin timestamp for any Solana program deployed to the blockchain.

`solc` is a simple tool that can be used to verify the authenticity of a Solana program by checking the timestamp of its deployment on the ledger. This is useful for verifying the origin of a program, especially when the program is not open-source or when the program is not deployed by the original author.

## Installation
Currently, the tool is available only through this repository.

### Local Build
You can clone the repository and build the tool using the Rust toolchain.
If you don't have Rust installed, you can install it using `rustup`. You can find the installation instructions [here](https://rustup.rs/).

```bash
$ git clone BHawleyWall/solception
$ cd solception
$ cargo build --release
```

### Docker
You can also build the tool using Docker. The Dockerfile is included in the repository.

```bash
$ git clone BHawleyWall/solception
$ cd solception
$ docker build -t solc .
```

## Usage
```bash
$ solc <program_id>
```

## Documentation
The tool uses the Solana RPC API to query the ledger for the timestamp of the program deployment. The tool sends a request to the `getProgramAccounts` endpoint and filters the response to find the account with the specified program ID. The timestamp of the account creation is then extracted and displayed to the user.

## License
This project is licensed under the  GNU General Public License - see the [LICENSE](LICENSE) file for details.

## Contributing
Currently, the project is in its early stages and is not accepting contributions. However, feel free to open an issue if you have any suggestions or feedback.
