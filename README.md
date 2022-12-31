# 🚚 Dray 🚚
A cloud native SFTP server designed to support multiple data storage backends, starting with S3.

## Why?
Many enterprise applications, such as ADP, SAP, and Workday, are used within companies as a source of truth for their data. These applications offer automated SFTP uploads to synchronize data with other applications. In short, applications that want to interface with enterprise data need to support SFTP.

Dray aims to tackle the undifferentiated heavy lifting of handling SFTP integrations, so developers can focus on differentiating their product.

## What's With the Name
A dray is a cart used to transport heavy cargo short distances. Dray transports files of any size to the storage backend.

## Demo
[![Demo](https://asciinema.org/a/Zjdni4AYhpX65GwhXBxUeaEcE.svg)](https://asciinema.org/a/Zjdni4AYhpX65GwhXBxUeaEcE)

## Hacking
### Prerequisites
- [Install Rust](https://rustup.rs)
- Docker
- cd to the `dray` repository
- Generate an SSH keypair in the `dray` directory (`mkdir -p .ssh && ssh-keygen -t ed25519 -f .ssh/id_ed25519`)

### Test, Build, and Run
- Start Containers (`docker-compose up`)
- Run Tests (`cargo test`)
- Run Dray (`cargo run`)

## 🚧 Work in Progress 🚧
Dray fully implements SSH File Transfer Protocol Version 3 Draft 2; however, it has only been tested against a limited number of SFTP clients. 
Production usage is not recommended.

## Minimum Viable Product (MVP) Roadmap
- [x] Deserialize and Serialize SSH File Transfer Protocol Version 3 Draft 2
- [x] Accept SSH connections
- [x] Verify SSH keys against authorized keys stored in S3
- [x] SFTP subsystem initialization
- [x] List directory (S3-Compatible Storage Only)
- [x] Create directory (S3-Compatible Storage Only)
- [x] Rename directory (S3-Compatible Storage Only)
- [x] Remove directory (S3-Compatible Storage Only)
- [x] Read file (S3-Compatible Storage Only)
- [x] Write file (S3-Compatible Storage Only)
- [x] Rename file (S3-Compatible Storage Only)
- [x] Remove file (S3-Compatible Storage Only)
- [x] Stat commands (S3-Compatible Storage Only)
- [x] Permission checks
- [x] Set Stat commands return "Not Implemented"
